use crate::{Capabilities, IdentityStore, Invite, PhoneModel, PhoneState, PhoneUpdate, Platform};
use serde::Serialize;
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Serialize)]
pub struct Interface {
    pub name: String,
    pub address: Ipv4Addr,
}
pub fn interfaces() -> Vec<Interface> {
    let mut result = Vec::new();
    for (name, addr) in local_ip_address::list_afinet_netifas().unwrap_or_default() {
        if let std::net::IpAddr::V4(address) = addr
            && address.is_private()
            && !result.iter().any(|i: &Interface| i.address == address)
        {
            result.push(Interface { name, address });
        }
    }
    result
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneStatus {
    pub enabled: bool,
    pub connected: bool,
    pub pc_id: Option<String>,
    pub address: Option<Ipv4Addr>,
    pub port: Option<u16>,
    pub pair_uri: Option<String>,
    pub qr_svg: Option<String>,
    pub device_name: Option<String>,
    pub platform: Option<Platform>,
    pub error: Option<String>,
    pub capabilities: Capabilities,
}
#[derive(Clone, Copy)]
pub(crate) struct Endpoint {
    pub address: Ipv4Addr,
    pub port: u16,
}
pub(crate) struct Control {
    pub identity: Arc<Mutex<IdentityStore>>,
    pub model: Arc<Mutex<PhoneModel>>,
    pub stop: AtomicBool,
    pub epoch: AtomicU64,
    pub revocation: AtomicU64,
    pub invite: Mutex<Invite>,
    pub endpoint: Mutex<Option<Endpoint>>,
    pub error: Mutex<Option<String>>,
    pub adapter: String,
    pub preferred_address: Ipv4Addr,
    pub authorization: Mutex<()>,
}
impl Control {
    pub fn forget(&self) -> Result<(), String> {
        let _authorization = self.authorization.lock().unwrap();
        self.identity.lock().unwrap().forget()?;
        self.revocation.fetch_add(1, Ordering::AcqRel);
        *self.invite.lock().unwrap() = Invite::new(Instant::now());
        self.model.lock().unwrap().forget();
        Ok(())
    }
}
struct Runner {
    control: Arc<Control>,
    thread: std::thread::JoinHandle<()>,
}
pub struct PhoneBridge {
    identity: Option<Arc<Mutex<IdentityStore>>>,
    model: Arc<Mutex<PhoneModel>>,
    runner: Mutex<Option<Runner>>,
    initial_error: Option<String>,
}
impl PhoneBridge {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let identity = IdentityStore::load(path)?;
        let mut model = PhoneModel::default();
        if let Some(device) = identity.device() {
            let epoch = model.connect(device);
            model.disconnect(epoch);
        }
        Ok(Self {
            identity: Some(Arc::new(Mutex::new(identity))),
            model: Arc::new(Mutex::new(model)),
            runner: Mutex::new(None),
            initial_error: None,
        })
    }
    pub fn disabled() -> Self {
        Self {
            identity: None,
            model: Arc::new(Mutex::new(PhoneModel::default())),
            runner: Mutex::new(None),
            initial_error: None,
        }
    }
    pub fn unavailable(error: String) -> Self {
        {
            let mut value = Self::disabled();
            value.initial_error = Some(error);
            value
        }
    }
    pub fn configure(&self, address: Option<Ipv4Addr>) -> Result<(), String> {
        let mut runner = self.runner.lock().unwrap();
        if let Some(old) = runner.take() {
            old.control.stop.store(true, Ordering::Release);
            old.control.epoch.fetch_add(1, Ordering::AcqRel);
            let _ = old.thread.join();
        }
        let epoch = self.model.lock().unwrap().state().generation;
        self.model.lock().unwrap().disconnect(epoch);
        let Some(address) = address else { return Ok(()) };
        let adapter = interfaces()
            .into_iter()
            .find(|i| i.address == address)
            .ok_or("Choose an assigned private IPv4 interface")?
            .name;
        let identity = self
            .identity
            .clone()
            .ok_or_else(|| self.initial_error.clone().unwrap_or("Phone data is unavailable in this host".into()))?;
        let control = Arc::new(Control {
            identity,
            model: self.model.clone(),
            stop: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            revocation: AtomicU64::new(0),
            invite: Mutex::new(Invite::new(Instant::now())),
            endpoint: Mutex::new(None),
            error: Mutex::new(None),
            adapter,
            preferred_address: address,
            authorization: Mutex::new(()),
        });
        let run = control.clone();
        let thread = std::thread::Builder::new()
            .name("westplay-phone".into())
            .spawn(move || match tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build() {
                Ok(runtime) => {
                    runtime.block_on(crate::server::serve(run));
                    runtime.shutdown_timeout(Duration::from_secs(2));
                }
                Err(e) => {
                    *run.error.lock().unwrap() = Some(e.to_string());
                }
            })
            .map_err(|e| e.to_string())?;
        *runner = Some(Runner { control, thread });
        Ok(())
    }
    pub fn snapshot(&self) -> PhoneState {
        self.model.lock().unwrap().state().clone()
    }
    pub fn updates_after(&self, revision: u64) -> Option<PhoneUpdate> {
        self.model.try_lock().ok()?.take_after(revision)
    }
    pub fn renew_invite(&self) -> Result<(), String> {
        if let Some(r) = self.runner.lock().unwrap().as_ref() {
            *r.control.invite.lock().unwrap() = Invite::new(Instant::now());
            Ok(())
        } else {
            Err("Enable phone data first".into())
        }
    }
    pub fn forget(&self) -> Result<(), String> {
        let runner = self.runner.lock().unwrap();
        if let Some(r) = runner.as_ref() {
            r.control.forget()
        } else {
            self.identity.as_ref().ok_or("Phone data unavailable")?.lock().unwrap().forget()?;
            self.model.lock().unwrap().forget();
            Ok(())
        }
    }
    pub fn status(&self) -> PhoneStatus {
        let Some(identity) = &self.identity else {
            return PhoneStatus { error: self.initial_error.clone(), ..Default::default() };
        };
        let (pc_id, pin, device) = {
            let i = identity.lock().unwrap();
            (i.pc_id().to_owned(), i.fingerprint(), i.device())
        };
        let model = self.snapshot();
        let slot = self.runner.lock().unwrap();
        let mut status = PhoneStatus {
            pc_id: Some(pc_id.clone()),
            device_name: device.as_ref().map(|d| d.name.clone()),
            platform: device.map(|d| d.platform),
            connected: model.connected,
            capabilities: model.capabilities,
            ..Default::default()
        };
        if let Some(r) = slot.as_ref() {
            status.enabled = !r.control.stop.load(Ordering::Relaxed);
            status.error = r.control.error.lock().unwrap().clone();
            if let Some(endpoint) = *r.control.endpoint.lock().unwrap() {
                status.address = Some(endpoint.address);
                status.port = Some(endpoint.port);
                let uri = format!(
                    "westplay://phone?v=1&pc={pc_id}&host={}&port={}&fp={pin}&invite={}",
                    endpoint.address,
                    endpoint.port,
                    r.control.invite.lock().unwrap().secret()
                );
                status.qr_svg = qrcode::QrCode::new(uri.as_bytes())
                    .ok()
                    .map(|q| q.render::<qrcode::render::svg::Color>().min_dimensions(220, 220).build());
                status.pair_uri = Some(uri);
            }
        }
        status
    }
}
impl Drop for PhoneBridge {
    fn drop(&mut self) {
        let _ = self.configure(None);
    }
}
