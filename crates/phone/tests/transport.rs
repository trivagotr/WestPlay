use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use westplay_phone::{IdentityStore, PhoneBridge, PhoneStatus, interfaces};
type Client = WebSocketStream<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>;
async fn started(bridge: &PhoneBridge) -> PhoneStatus {
    let end = Instant::now() + Duration::from_secs(6);
    loop {
        let status = bridge.status();
        if status.port.is_some() {
            return status;
        }
        assert!(Instant::now() < end, "service failed: {:?}", status.error);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
async fn connect(status: &PhoneStatus, cert: rustls::pki_types::CertificateDer<'static>) -> Result<Client, String> {
    let ip = status.address.unwrap();
    let port = status.port.unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert).unwrap();
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let stream = tokio::net::TcpStream::connect((ip, port)).await.map_err(|e| e.to_string())?;
    let tls = tokio_rustls::TlsConnector::from(Arc::new(config))
        .connect(rustls::pki_types::ServerName::IpAddress(ip.into()), stream)
        .await
        .map_err(|e| e.to_string())?;
    tokio_tungstenite::client_async(format!("wss://{ip}:{port}/phone"), tls)
        .await
        .map(|(ws, _)| ws)
        .map_err(|e| e.to_string())
}
async fn receive(client: &mut Client) -> Value {
    let next = tokio::time::timeout(Duration::from_secs(3), client.next()).await.unwrap().unwrap().unwrap();
    serde_json::from_str(next.to_text().unwrap()).unwrap()
}
async fn send(client: &mut Client, value: Value) {
    client.send(Message::Text(value.to_string().into())).await.unwrap();
}
fn auth(kind: &str, credential: &str) -> Value {
    let mut value =
        json!({"version":1,"type":kind,"deviceId":"transport-test","deviceName":"Android test","platform":"android"});
    value[if kind == "pair" { "invite" } else { "token" }] = credential.into();
    value
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pairing_events_revocation_and_port_fallback_use_the_same_identity() {
    let Some(ip) = interfaces().first().map(|i| i.address) else {
        eprintln!("No private interface; transport test skipped");
        return;
    };
    let dir = std::env::temp_dir().join(format!("westplay-transport-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("identity.json");
    let bridge = PhoneBridge::new(path.clone()).unwrap();
    bridge.configure(Some(ip)).unwrap();
    let first = started(&bridge).await;
    let cert = IdentityStore::load(path.clone()).unwrap().certificate_der(ip).unwrap();
    let wrong = IdentityStore::load(dir.join("other.json")).unwrap().certificate_der(ip).unwrap();
    assert!(connect(&first, wrong).await.is_err(), "untrusted PC must fail TLS validation");
    let mut client = connect(&first, cert.clone()).await.unwrap();
    let invite = first.pair_uri.as_ref().unwrap().split("&invite=").nth(1).unwrap();
    send(&mut client, auth("pair", invite)).await;
    let ready = receive(&mut client).await;
    assert_eq!(ready["type"], "ready");
    let token = ready["token"].as_str().unwrap().to_owned();
    send(&mut client,json!({"version":1,"sequence":1,"eventType":"deviceHello","payload":{"capabilities":{"battery":true,"charging":true,"notifications":true,"notificationBody":false,"backgroundRelay":true}}})).await;
    assert_eq!(receive(&mut client).await["sequence"], 1);
    send(
        &mut client,
        json!({"version":1,"sequence":2,"eventType":"deviceState","payload":{"batteryPct":14,"charging":false}}),
    )
    .await;
    assert_eq!(receive(&mut client).await["sequence"], 2);
    assert_eq!(bridge.snapshot().battery_pct, Some(14));
    send(&mut client,json!({"version":1,"sequence":3,"eventType":"notificationPosted","payload":{"id":"msg","appId":"chat","appName":"Chat","title":"Hello","body":"Private"}})).await;
    receive(&mut client).await;
    let update = bridge.updates_after(0).unwrap();
    assert_eq!(update.events.len(), 1);
    assert!(update.events[0].body.is_none(), "body permission must be enforced by host");
    bridge.configure(None).unwrap();
    assert!(!bridge.snapshot().connected);
    let occupied = std::net::TcpListener::bind((ip, first.port.unwrap())).unwrap();
    bridge.configure(Some(ip)).unwrap();
    let changed = started(&bridge).await;
    assert_ne!(first.port, changed.port);
    assert_eq!(first.pc_id, changed.pc_id);
    assert_eq!(
        first.pair_uri.as_ref().unwrap().split("&fp=").nth(1).unwrap().split('&').next(),
        changed.pair_uri.as_ref().unwrap().split("&fp=").nth(1).unwrap().split('&').next()
    );
    let mut reconnected = connect(&changed, cert.clone()).await.unwrap();
    send(&mut reconnected, auth("authenticate", &token)).await;
    let ready = receive(&mut reconnected).await;
    assert_eq!(ready["type"], "ready");
    assert!(ready.get("token").is_none());
    bridge.forget().unwrap();
    assert!(!bridge.snapshot().connected);
    let revoked = receive(&mut reconnected).await;
    assert_eq!(revoked["code"], "revoked");
    let mut rejected = connect(&changed, cert).await.unwrap();
    send(&mut rejected, auth("authenticate", &token)).await;
    assert_eq!(receive(&mut rejected).await["code"], "unauthorized");
    bridge.configure(None).unwrap();
    drop(occupied);
    drop(bridge);
    std::fs::remove_dir_all(dir).unwrap();
}
