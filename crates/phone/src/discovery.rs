use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};
use std::net::{IpAddr, Ipv4Addr};
pub(crate) struct Advertisement {
    daemon: ServiceDaemon,
    name: String,
}
impl Advertisement {
    pub fn start(ip: Ipv4Addr, port: u16, pc_id: &str) -> Result<Self, String> {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        let result = (|| {
            daemon.disable_interface(IfKind::All).map_err(|e| e.to_string())?;
            daemon.enable_interface(IfKind::Addr(IpAddr::V4(ip))).map_err(|e| e.to_string())?;
            let instance = format!("WestPlay-{}", &pc_id[..8]);
            let host = format!("westplay-{pc_id}.local.");
            let service = ServiceInfo::new(
                "_westplay._tcp.local.",
                &instance,
                &host,
                std::net::IpAddr::V4(ip),
                port,
                &[("v", "1"), ("role", "phone-data"), ("pcId", pc_id)][..],
            )
            .map_err(|e| e.to_string())?;
            let name = service.get_fullname().to_owned();
            daemon.register(service).map_err(|e| e.to_string())?;
            Ok(name)
        })();
        match result {
            Ok(name) => Ok(Self { daemon, name }),
            Err(e) => {
                let _ = daemon.shutdown();
                Err(e)
            }
        }
    }
}
impl Drop for Advertisement {
    fn drop(&mut self) {
        let _ = self.daemon.unregister(&self.name);
        let _ = self.daemon.shutdown();
    }
}
