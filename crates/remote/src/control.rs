use crate::{Frame, Input, InputQueue, Latest, Pairing, allowed_address};
use serde::Serialize;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
use webrtc::peer_connection::RTCPeerConnection;

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatus {
    pub enabled: bool,
    pub url: Option<String>,
    pub code: Option<String>,
    pub qr_svg: Option<String>,
    pub connected: bool,
    pub encoder: Option<String>,
    pub error: Option<String>,
    pub frames: u64,
}

#[derive(Serialize)]
pub struct Interface {
    pub name: String,
    pub address: Ipv4Addr,
}
pub fn interfaces() -> Vec<Interface> {
    let mut result = Vec::new();
    for (name, addr) in local_ip_address::list_afinet_netifas().unwrap_or_default() {
        if let std::net::IpAddr::V4(address) = addr
            && allowed_address(address)
            && !result.iter().any(|i: &Interface| i.address == address)
        {
            result.push(Interface { name, address });
        }
    }
    result
}

pub(crate) struct Run {
    pub dashboard: Mutex<crate::dashboard::Dashboard>,
    pub observer: crate::convoy::Observer,
    pub host: String,
    pub bind: Ipv4Addr,
    pub stop: AtomicBool,
    pub connected: AtomicBool,
    pub pairing: Mutex<Pairing>,
    pub inputs: Mutex<InputQueue>,
    pub frame: Mutex<Latest<Frame>>,
    pub peer: tokio::sync::Mutex<Option<Arc<RTCPeerConnection>>>,
    pub negotiating: AtomicBool,
    pub heartbeat: Mutex<Instant>,
    pub error: Mutex<Option<String>>,
    pub frames: AtomicU64,
    pub encoder: String,
}
impl Run {
    pub(crate) fn new(host: String, bind: Ipv4Addr, encoder: String) -> Self {
        Self {
            dashboard: Mutex::new(crate::dashboard::Dashboard::default()),
            observer: crate::convoy::Observer::default(),
            host,
            bind,
            stop: AtomicBool::new(false),
            connected: AtomicBool::new(false),
            pairing: Mutex::new(Pairing::new(Instant::now())),
            inputs: Mutex::new(InputQueue::default()),
            frame: Mutex::new(Latest::default()),
            peer: tokio::sync::Mutex::new(None),
            negotiating: AtomicBool::new(false),
            heartbeat: Mutex::new(Instant::now()),
            error: Mutex::new(None),
            frames: AtomicU64::new(0),
            encoder,
        }
    }
    pub fn release(&self) {
        self.connected.store(false, Ordering::Release);
        self.observer.stop();
        self.inputs.lock().unwrap().push(Input::Cancel);
    }
    pub fn fail(&self, error: String) {
        *self.error.lock().unwrap() = Some(error);
        self.release();
    }
    pub fn authorized(&self, token: &str) -> bool {
        !self.stop.load(Ordering::Acquire) && self.pairing.lock().unwrap().authorized(token, Instant::now())
    }
}

struct Runner {
    state: Arc<Run>,
    thread: std::thread::JoinHandle<()>,
}
#[derive(Default)]
pub struct RemoteControl {
    runner: Mutex<Option<Runner>>,
}
impl RemoteControl {
    pub fn new() -> Self {
        Self::default()
    }
    /// Called by a companion command, never by the host's frame/UI thread.
    pub fn configure(&self, bind: Option<Ipv4Addr>) -> Result<(), String> {
        let mut slot = self.runner.lock().unwrap();
        if let Some(runner) = slot.take() {
            runner.state.stop.store(true, Ordering::Release);
            runner.state.release();
            let _ = runner.thread.join();
        }
        let Some(bind) = bind else {
            return Ok(());
        };
        if !allowed_address(bind) || !interfaces().iter().any(|i| i.address == bind) {
            return Err("Choose a private IPv4 address assigned to this PC.".into());
        }
        let encoder = crate::encoder::probe()?;
        let listener = TcpListener::bind((bind, 0)).map_err(|e| format!("Cannot listen on this interface: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let host = listener.local_addr().map_err(|e| e.to_string())?.to_string();
        let state = Arc::new(Run::new(host, bind, encoder));
        let run = state.clone();
        let thread = std::thread::Builder::new()
            .name("westplay-mobile".into())
            .spawn(move || match tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build() {
                Ok(runtime) => {
                    runtime.block_on(crate::server::serve(run, listener));
                    runtime.shutdown_timeout(Duration::from_secs(2));
                }
                Err(e) => run.fail(format!("Mobile runtime could not start: {e}")),
            })
            .map_err(|e| e.to_string())?;
        *slot = Some(Runner { state, thread });
        Ok(())
    }
    pub fn status(&self) -> RemoteStatus {
        let slot = self.runner.lock().unwrap();
        let Some(runner) = slot.as_ref() else {
            return RemoteStatus::default();
        };
        let state = &runner.state;
        let url = format!("http://{}/", state.host);
        let qr_svg = qrcode::QrCode::new(url.as_bytes())
            .ok()
            .map(|q| q.render::<qrcode::render::svg::Color>().min_dimensions(200, 200).build());
        RemoteStatus {
            enabled: !state.stop.load(Ordering::Acquire),
            url: Some(url),
            code: Some(state.pairing.lock().unwrap().code().to_owned()),
            qr_svg,
            connected: state.connected.load(Ordering::Acquire),
            encoder: Some(state.encoder.clone()),
            error: state.error.lock().unwrap().clone(),
            frames: state.frames.load(Ordering::Relaxed),
        }
    }
    pub fn wants_frames(&self) -> bool {
        self.runner
            .try_lock()
            .ok()
            .and_then(|slot| {
                slot.as_ref()
                    .map(|r| !r.state.stop.load(Ordering::Relaxed) && r.state.connected.load(Ordering::Relaxed))
            })
            .unwrap_or(false)
    }
    pub fn publish(&self, frame: Frame) {
        if let Ok(slot) = self.runner.try_lock()
            && let Some(r) = slot.as_ref()
            && let Ok(mut pending) = r.state.frame.try_lock()
        {
            pending.put(frame);
        }
    }
    /// Latest snapshot for web only. Never waits behind server work on the frame thread.
    pub fn publish_dashboard(&self, pid: u32, vehicle: serde_json::Value) {
        if let Ok(slot) = self.runner.try_lock()
            && let Some(r) = slot.as_ref()
            && let Ok(mut data) = r.state.dashboard.try_lock()
        {
            data.publish(pid, vehicle, Instant::now());
        }
    }
    pub fn report_error(&self, message: String) {
        if let Ok(slot) = self.runner.try_lock()
            && let Some(r) = slot.as_ref()
        {
            r.state.fail(message);
        }
    }
    pub fn take_input(&self, _now: Instant) -> Vec<Input> {
        if let Ok(slot) = self.runner.try_lock()
            && let Some(r) = slot.as_ref()
            && let Ok(mut inputs) = r.state.inputs.try_lock()
        {
            return inputs.take();
        }
        vec![]
    }
}
impl Drop for RemoteControl {
    fn drop(&mut self) {
        let _ = self.configure(None);
    }
}
