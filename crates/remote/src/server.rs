use crate::{control::Run, valid_origin};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub(crate) async fn serve(state: Arc<Run>, listener: std::net::TcpListener) {
    let router = Router::new()
        .route("/", get(|| async { asset("text/html; charset=utf-8", include_str!("../web/index.html")) }))
        .route(
            "/client.js",
            get(|| async { asset("text/javascript; charset=utf-8", include_str!("../web/client.js")) }),
        )
        .route(
            "/pairing.js",
            get(|| async { asset("text/javascript; charset=utf-8", include_str!("../web/pairing.js")) }),
        )
        .route(
            "/geometry.js",
            get(|| async { asset("text/javascript; charset=utf-8", include_str!("../web/geometry.js")) }),
        )
        .route("/signals.js", get(|| async { asset("text/javascript; charset=utf-8", include_str!("../web/signals.js")) }))
        .route("/style.css", get(|| async { asset("text/css; charset=utf-8", include_str!("../web/style.css")) }))
        .route(
            "/instruments.js",
            get(|| async { asset("text/javascript; charset=utf-8", include_str!("../web/instruments.js")) }),
        )
        .route("/pair", post(pair))
        .route("/offer", post(offer))
        .route("/dashboard", post(dashboard))
        .route("/convoy", post(convoy))
        .route("/diagnostics", post(diagnostics))
        .route(
            "/dashboard.js",
            get(|| async { asset("text/javascript; charset=utf-8", include_str!("../web/dashboard.js")) }),
        )
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::from_std(listener) {
        Ok(l) => l,
        Err(e) => {
            state.fail(e.to_string());
            return;
        }
    };
    let stop = state.clone();
    let end = async move {
        while !stop.stop.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let peer = stop.peer.lock().await.take();
        if let Some(peer) = peer {
            let _ = peer.close().await;
        }
    };
    // Graceful HTTP shutdown alone waits indefinitely for an unfinished body.
    // Bound it; the owning runtime then aborts any remaining connection tasks.
    let bound = state.clone();
    let deadline = async move {
        while !bound.stop.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    tokio::select! {
        result = axum::serve(listener, router).with_graceful_shutdown(end) => {
            if let Err(e) = result { state.fail(e.to_string()); }
        }
        _ = deadline => {}
    }
}

fn asset(kind: &'static str, text: &'static str) -> Response {
    ([(header::CONTENT_TYPE, kind)], Body::from(text)).into_response()
}
async fn guard(State(state): State<Arc<Run>>, request: Request, next: Next) -> Response {
    let host = request.headers().get(header::HOST).and_then(|v| v.to_str().ok()).unwrap_or_default();
    let origin = request.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if state.stop.load(Ordering::Acquire)
        || host != state.host
        || origin.is_some_and(|o| !valid_origin(host, o))
        || (request.method() == axum::http::Method::POST && origin.is_none())
    {
        return (StatusCode::FORBIDDEN, "Use the local address shown on the PC.").into_response();
    }
    let mut response = next.run(request).await;
    for (key, value) in [
        ("cache-control", "no-store"),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("x-frame-options", "DENY"),
        (
            "content-security-policy",
            "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; media-src 'self' blob:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'",
        ),
    ] {
        response
            .headers_mut()
            .insert(axum::http::HeaderName::from_static(key), axum::http::HeaderValue::from_static(value));
    }
    response
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Code {
    code: String,
}
async fn pair(State(state): State<Arc<Run>>, Json(code): Json<Code>) -> Response {
    if code.code.len() > 64 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match state.pairing.lock().unwrap().pair(&code.code, Instant::now()) {
        Ok(token) => Json(serde_json::json!({ "token": token })).into_response(),
        Err(message) => (StatusCode::UNAUTHORIZED, message).into_response(),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Offer {
    token: String,
    description: RTCSessionDescription,
}
async fn offer(State(state): State<Arc<Run>>, Json(body): Json<Offer>) -> Response {
    if body.token.len() > 128 || !state.authorized(&body.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if state.negotiating.swap(true, Ordering::AcqRel) {
        return (StatusCode::CONFLICT, "This phone already has a connection. Wait for it to close.").into_response();
    }
    match tokio::time::timeout(Duration::from_secs(12), crate::peer::answer(state.clone(), body.description)).await {
        Ok(Ok(description)) => Json(description).into_response(),
        failure => {
            state.negotiating.store(false, Ordering::Release);
            let message = match failure {
                Ok(Err(e)) => e,
                _ => "Connection setup timed out.".into(),
            };
            (StatusCode::BAD_REQUEST, message).into_response()
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Auth {
    token: String,
}
async fn dashboard(State(state): State<Arc<Run>>, Json(auth): Json<Auth>) -> Response {
    if auth.token.len() > 128 || !state.authorized(&auth.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let now = Instant::now();
    let mut snapshot = state.dashboard.lock().unwrap().snapshot(now);
    snapshot["convoy"] = state.observer.data.lock().unwrap().snapshot(now);
    if snapshot["connected"] != true {
        snapshot["convoy"]["players"] = serde_json::json!([]);
    }
    Json(snapshot).into_response()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observe {
    token: String,
    enabled: bool,
}
async fn convoy(State(state): State<Arc<Run>>, Json(auth): Json<Observe>) -> Response {
    if auth.token.len() > 128 || !state.authorized(&auth.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    // Older clients may still send this command. Tracking is now host-owned;
    // neither a mobile connection nor a manual command controls its lifetime.
    let _ = auth.enabled;
    Json(serde_json::json!({"ok":true,"automatic":true})).into_response()
}
async fn diagnostics(State(state): State<Arc<Run>>, Json(auth): Json<Auth>) -> Response {
    if auth.token.len() > 128 || !state.authorized(&auth.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(serde_json::json!({"schema":"westplay-web-diagnostics-v1","convoy":state.observer.data.lock().unwrap().logs(),"capabilities":{"passengerSatisfaction":true,"laneAssist":false,"completeRoster":false}})).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };
    fn request(host: &str, method: &str, path: &str, origin: Option<&str>, body: &str) -> String {
        let mut stream = TcpStream::connect(host).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let origin = origin.map(|v| format!("Origin: {v}\r\n")).unwrap_or_default();
        write!(stream, "{method} {path} HTTP/1.1\r\nHost: {host}\r\n{origin}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actual_http_rejects_foreign_origin_missing_origin_and_second_pairing() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let host = listener.local_addr().unwrap().to_string();
        let state = Arc::new(Run::new(host.clone(), "127.0.0.1".parse().unwrap(), "test".into()));
        let task = tokio::spawn(serve(state.clone(), listener));
        tokio::time::sleep(Duration::from_millis(20)).await;
        let body = serde_json::json!({"code": state.pairing.lock().unwrap().code()}).to_string();
        assert!(request(&host, "POST", "/pair", Some("http://evil.test"), &body).starts_with("HTTP/1.1 403"));
        assert!(request(&host, "POST", "/pair", None, &body).starts_with("HTTP/1.1 403"));
        let origin = format!("http://{host}");
        let response = request(&host, "POST", "/pair", Some(&origin), &body);
        assert!(response.starts_with("HTTP/1.1 200"));
        let data: serde_json::Value = serde_json::from_str(response.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        let token = data["token"].as_str().unwrap();
        assert!(state.authorized(token));
        assert!(
            request(&host, "POST", "/dashboard", Some(&origin), r#"{"token":"invalid"}"#).starts_with("HTTP/1.1 401")
        );
        state.dashboard.lock().unwrap().publish(77, serde_json::json!({"parkingBrake":false}), Instant::now());
        let auth = serde_json::json!({"token":token}).to_string();
        let dashboard = request(&host, "POST", "/dashboard", Some(&origin), &auth);
        assert!(dashboard.starts_with("HTTP/1.1 200"));
        assert!(dashboard.contains("parkingBrake"));
        assert!(!dashboard.contains("pid"));
        assert!(request(&host, "POST", "/diagnostics", Some(&origin), &auth).starts_with("HTTP/1.1 200"));
        assert!(
            request(&host, "POST", "/convoy", Some(&origin), r#"{"token":"invalid","enabled":true}"#)
                .starts_with("HTTP/1.1 401")
        );

        let qr_body = serde_json::json!({"code":state.pairing.lock().unwrap().invitation()}).to_string();
        let qr_response = request(&host, "POST", "/pair", Some(&origin), &qr_body);
        assert!(qr_response.starts_with("HTTP/1.1 200"));
        assert!(qr_response.contains(token), "rescanning QR retains the same session");
        assert!(request(&host, "POST", "/pair", Some(&origin), &body).starts_with("HTTP/1.1 401"));
        assert!(request(&host, "GET", "/", None, "").contains("frame-ancestors 'none'"));
        state.stop.store(true, Ordering::Release);
        assert!(!state.authorized(token));
        tokio::time::timeout(Duration::from_secs(3), task).await.unwrap().unwrap();
        assert!(TcpStream::connect(&host).is_err());
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stopping_does_not_wait_for_an_unfinished_http_body() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let host = listener.local_addr().unwrap().to_string();
        let state = Arc::new(Run::new(host.clone(), "127.0.0.1".parse().unwrap(), "test".into()));
        let task = tokio::spawn(serve(state.clone(), listener));
        let mut slow = TcpStream::connect(&host).unwrap();
        write!(slow,"POST /pair HTTP/1.1\r\nHost: {host}\r\nOrigin: http://{host}\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{").unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        state.stop.store(true, Ordering::Release);
        assert!(tokio::time::timeout(Duration::from_secs(2), task).await.is_ok(), "shutdown must bound slow clients");
        drop(slow);
    }
}
