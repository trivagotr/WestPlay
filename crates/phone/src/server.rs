use crate::{
    DeviceInfo, Platform,
    bridge::{Control, Endpoint, interfaces},
    crypto,
    discovery::Advertisement,
    parse_event,
    protocol::{MAX_MESSAGE, text_valid},
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::{
    net::TcpListener,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{
        Message,
        handshake::server::{ErrorResponse, Request, Response},
        protocol::WebSocketConfig,
    },
};
type Socket = WebSocketStream<TlsStream<tokio::net::TcpStream>>;

pub(crate) async fn serve(control: Arc<Control>) {
    let permits = Arc::new(tokio::sync::Semaphore::new(8));
    while !control.stop.load(Ordering::Acquire) {
        let addresses: Vec<_> =
            interfaces().into_iter().filter(|i| i.name == control.adapter).map(|i| i.address).collect();
        let address = addresses
            .iter()
            .copied()
            .find(|ip| *ip == control.preferred_address)
            .or_else(|| addresses.first().copied());
        let Some(address) = address else {
            *control.error.lock().unwrap() = Some("Waiting for the selected private network".into());
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        };
        let prepared = (|| -> Result<_, String> {
            let mut identity = control.identity.lock().unwrap();
            let preferred = identity.preferred_port();
            let listener = match TcpListener::bind((address, preferred)) {
                Ok(l) => l,
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    TcpListener::bind((address, 0)).map_err(|e| e.to_string())?
                }
                Err(e) => return Err(e.to_string()),
            };
            let port = listener.local_addr().map_err(|e| e.to_string())?.port();
            listener.set_nonblocking(true).map_err(|e| e.to_string())?;
            if port != preferred {
                identity.remember_port(port)?;
            }
            let tls = identity.tls(address)?;
            let pc_id = identity.pc_id().to_owned();
            Ok((listener, port, tls, pc_id))
        })();
        let (listener, port, tls, pc_id) = match prepared {
            Ok(p) => p,
            Err(e) => {
                *control.error.lock().unwrap() = Some(e);
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        };
        let advertised = Advertisement::start(address, port, &pc_id);
        *control.error.lock().unwrap() =
            advertised.as_ref().err().map(|e| format!("Automatic discovery unavailable; use QR: {e}"));
        *control.endpoint.lock().unwrap() = Some(Endpoint { address, port });
        let epoch = control.epoch.fetch_add(1, Ordering::AcqRel) + 1;
        let listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(l) => l,
            Err(e) => {
                *control.error.lock().unwrap() = Some(e.to_string());
                break;
            }
        };
        let acceptor = TlsAcceptor::from(tls);
        let mut timer = tokio::time::interval(Duration::from_millis(250));
        let mut network_check = Instant::now();
        loop {
            tokio::select! {
                incoming=listener.accept()=>{
                    let Ok((stream,_))=incoming else{break};
                    let Ok(permit)=permits.clone().try_acquire_owned()else{continue};
                    let acceptor=acceptor.clone();let c=control.clone();
                    tokio::spawn(async move{
                        let _permit=permit;
                        if let Ok(Ok(tls))=tokio::time::timeout(Duration::from_secs(5),acceptor.accept(stream)).await{
                            let mut cfg=WebSocketConfig::default();cfg.max_message_size=Some(MAX_MESSAGE);cfg.max_frame_size=Some(MAX_MESSAGE);cfg.read_buffer_size=8192;cfg.write_buffer_size=0;cfg.max_write_buffer_size=32768;
                            let host=format!("{address}:{port}");
                            let callback=move|request:&Request,response:Response|->Result<Response,ErrorResponse>{
                                if request.uri().path()!="/phone"||request.headers().get("host").and_then(|v|v.to_str().ok())!=Some(host.as_str())||request.headers().contains_key("origin"){
                                    return Err(tokio_tungstenite::tungstenite::http::Response::builder().status(403).body(Some("Native phone endpoint only".into())).unwrap());
                                }Ok(response)
                            };
                            if let Ok(Ok(socket))=tokio::time::timeout(Duration::from_secs(5),tokio_tungstenite::accept_hdr_async_with_config(tls,callback,Some(cfg))).await{connection(c,socket,epoch).await;}
                        }
                    });
                }
                _=timer.tick()=>{
                    if control.stop.load(Ordering::Acquire){break;}
                    if network_check.elapsed()>=Duration::from_secs(3){network_check=Instant::now();if !interfaces().iter().any(|i|i.name==control.adapter&&i.address==address){break;}}
                }
            }
        }
        control.epoch.fetch_add(1, Ordering::AcqRel);
        *control.endpoint.lock().unwrap() = None;
        let generation = control.model.lock().unwrap().state().generation;
        control.model.lock().unwrap().disconnect(generation);
        drop(advertised);
        drop(listener);
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
}

async fn send(socket: &mut Socket, value: serde_json::Value) -> bool {
    tokio::time::timeout(Duration::from_secs(1), socket.send(Message::Text(value.to_string().into())))
        .await
        .is_ok_and(|r| r.is_ok())
}
async fn error(socket: &mut Socket, code: &str) {
    let _ = send(socket, serde_json::json!({"version":1,"type":"error","code":code,"message":code})).await;
    let _ = socket.close(None).await;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Authentication {
    version: u32,
    #[serde(rename = "type")]
    kind: String,
    device_id: String,
    device_name: String,
    platform: Platform,
    invite: Option<String>,
    token: Option<String>,
}
struct Session {
    token: Option<String>,
    revoke: u64,
    generation: u64,
    pc_id: String,
}
fn establish(control: &Control, auth: Authentication, device: DeviceInfo, epoch: u64) -> Result<Session, String> {
    // Forget and authentication must linearize before a socket can become connected.
    let _authorization = control.authorization.lock().unwrap();
    if control.stop.load(Ordering::Acquire) || control.epoch.load(Ordering::Acquire) != epoch {
        return Err("serviceStopped".into());
    }
    let mut identity = control.identity.lock().unwrap();
    let token = match auth.kind.as_str() {
        "pair" => {
            if identity.device().is_some() {
                return Err("alreadyPaired".into());
            }
            if !control.invite.lock().unwrap().redeem(auth.invite.as_deref().unwrap_or_default(), Instant::now()) {
                return Err("expiredInvite".into());
            }
            let token = crypto::secret();
            identity.pair(device.clone(), &token)?;
            Some(token)
        }
        "authenticate" if auth.token.as_deref().is_some_and(|t| t.len() == 64 && identity.authenticate(&device, t)) => {
            None
        }
        _ => return Err("unauthorized".into()),
    };
    let revoke = control.revocation.load(Ordering::Acquire);
    let generation = control.model.lock().unwrap().connect(device);
    Ok(Session { token, revoke, generation, pc_id: identity.pc_id().to_owned() })
}
async fn connection(control: Arc<Control>, mut socket: Socket, epoch: u64) {
    let Some(Ok(Message::Text(text))) =
        tokio::time::timeout(Duration::from_secs(5), socket.next()).await.ok().flatten()
    else {
        return;
    };
    if text.len() > 4096 {
        error(&mut socket, "unauthorized").await;
        return;
    }
    let auth: Authentication = match serde_json::from_str(&text) {
        Ok(a) => a,
        Err(_) => {
            error(&mut socket, "unauthorized").await;
            return;
        }
    };
    if auth.version != 1 {
        error(&mut socket, "protocolMismatch").await;
        return;
    }
    if auth.device_id.is_empty() || !text_valid(&auth.device_id, 128) || !text_valid(&auth.device_name, 80) {
        error(&mut socket, "unauthorized").await;
        return;
    }
    let device = DeviceInfo { id: auth.device_id.clone(), name: auth.device_name.clone(), platform: auth.platform };
    let session = match establish(&control, auth, device, epoch) {
        Ok(s) => s,
        Err(e) => {
            error(&mut socket, &e).await;
            return;
        }
    };
    let Session { token, revoke, generation, pc_id } = session;
    let mut ready = serde_json::json!({"version":1,"type":"ready","pcId":pc_id,"heartbeatMs":5000});
    if let Some(token) = token {
        ready["token"] = token.into();
    }
    if !send(&mut socket, ready).await {
        control.model.lock().unwrap().disconnect(generation);
        return;
    }
    let mut heartbeat = Instant::now();
    let mut timer = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            message=socket.next()=>{
                match message{
                    Some(Ok(Message::Text(text)))=>{
                        let envelope=match parse_event(&text){Ok(e)=>e,Err(_)=>{error(&mut socket,"invalidEvent").await;break;}};
                        let sequence=envelope.sequence;let ended=matches!(envelope.event,crate::Event::End);
                        if control.stop.load(Ordering::Acquire)||control.epoch.load(Ordering::Acquire)!=epoch||control.revocation.load(Ordering::Acquire)!=revoke{break;}
                        let accepted=control.model.lock().unwrap().apply(generation,envelope);
                        if !accepted{break;}heartbeat=Instant::now();
                        if !send(&mut socket,serde_json::json!({"version":1,"type":"ack","sequence":sequence})).await||ended{break;}
                    }
                    Some(Ok(Message::Ping(_)|Message::Pong(_)))=>{},
                    _=>break,
                }
            }
            _=timer.tick()=>{
                if control.stop.load(Ordering::Acquire){error(&mut socket,"serviceStopped").await;break;}
                if control.revocation.load(Ordering::Acquire)!=revoke{error(&mut socket,"revoked").await;break;}
                if control.epoch.load(Ordering::Acquire)!=epoch||heartbeat.elapsed()>Duration::from_secs(15){break;}
                if control.model.lock().unwrap().state().generation!=generation{break;}
            }
        }
    }
    control.model.lock().unwrap().disconnect(generation);
    let _ = tokio::time::timeout(Duration::from_secs(1), socket.close(None)).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdentityStore, Invite, PhoneModel};
    use std::sync::{Mutex, atomic::AtomicBool};
    #[test]
    fn forget_racing_authentication_cannot_register_a_revoked_session() {
        let path = std::env::temp_dir().join(format!("westplay-auth-race-{}.json", std::process::id()));
        let mut identity = IdentityStore::load(path.clone()).unwrap();
        let _ = identity.forget();
        let token = crypto::secret();
        let device = DeviceInfo { id: "race".into(), name: "test".into(), platform: Platform::Android };
        identity.pair(device.clone(), &token).unwrap();
        let control = Arc::new(Control {
            identity: Arc::new(Mutex::new(identity)),
            model: Arc::new(Mutex::new(PhoneModel::default())),
            stop: AtomicBool::new(false),
            epoch: 1.into(),
            revocation: 0.into(),
            invite: Mutex::new(Invite::new(Instant::now())),
            endpoint: Mutex::new(None),
            error: Mutex::new(None),
            adapter: String::new(),
            preferred_address: "192.168.1.1".parse().unwrap(),
            authorization: Mutex::new(()),
        });
        // Pause authentication on the identity lock after it takes the authorization gate.
        let identity_guard = control.identity.lock().unwrap();
        let c = control.clone();
        let d = device.clone();
        let t = token.clone();
        let auth = std::thread::spawn(move || {
            establish(
                &c,
                Authentication {
                    version: 1,
                    kind: "authenticate".into(),
                    device_id: d.id.clone(),
                    device_name: d.name.clone(),
                    platform: d.platform,
                    invite: None,
                    token: Some(t),
                },
                d,
                1,
            )
        });
        let end = Instant::now() + Duration::from_secs(3);
        loop {
            if control.authorization.try_lock().is_err() {
                break;
            }
            assert!(Instant::now() < end);
            std::thread::yield_now();
        }
        let c = control.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let forget = std::thread::spawn(move || {
            tx.send(()).unwrap();
            c.forget()
        });
        rx.recv().unwrap();
        drop(identity_guard);
        let session = auth.join().unwrap().unwrap();
        forget.join().unwrap().unwrap();
        assert_ne!(session.revoke, control.revocation.load(Ordering::Acquire));
        assert!(!control.model.lock().unwrap().state().connected);
        assert!(!control.identity.lock().unwrap().authenticate(&device, &token));
        drop(control);
        std::fs::remove_file(path).unwrap();
    }
}
