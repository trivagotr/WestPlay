use crate::{DeviceInfo, crypto};
use base64::{Engine, engine::general_purpose::STANDARD};
use rcgen::{CertificateParams, KeyPair};
use serde::{Deserialize, Serialize};
use std::{
    net::Ipv4Addr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Serialize, Deserialize)]
struct Paired {
    device: DeviceInfo,
    token_hash: String,
}
#[derive(Serialize, Deserialize)]
struct Saved {
    version: u8,
    pc_id: String,
    private_key: String,
    preferred_port: u16,
    paired: Option<Paired>,
}
pub struct IdentityStore {
    path: PathBuf,
    saved: Saved,
    key: KeyPair,
}
impl IdentityStore {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let (saved, key) = if path.exists() {
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            if bytes.len() > 32768 {
                return Err("Phone identity file too large".into());
            }
            let saved: Saved = serde_json::from_slice(&bytes)
                .map_err(|_| "Phone identity is invalid; preserve it before repairing pairing".to_owned())?;
            if saved.version != 1 || saved.pc_id.len() != 32 || !saved.pc_id.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("Unsupported phone identity file".into());
            }
            let sealed = STANDARD.decode(&saved.private_key).map_err(|_| "Invalid identity key")?;
            let plain = crypto::protect(&sealed, true)?;
            let pem = std::str::from_utf8(&plain).map_err(|_| "Invalid private key")?;
            let key = KeyPair::from_pem(pem).map_err(|_| "Invalid private key")?;
            (saved, key)
        } else {
            let key = KeyPair::generate().map_err(|e| e.to_string())?;
            let protected = crypto::protect(key.serialize_pem().as_bytes(), false)?;
            let saved = Saved {
                version: 1,
                pc_id: crypto::secret()[..32].into(),
                private_key: STANDARD.encode(protected),
                preferred_port: 0,
                paired: None,
            };
            (saved, key)
        };
        let store = Self { path, saved, key };
        store.save()?;
        Ok(store)
    }
    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temporary = self.path.with_extension(format!("tmp-{}", &crypto::secret()[..12]));
        let bytes = serde_json::to_vec_pretty(&self.saved).map_err(|e| e.to_string())?;
        std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        if let Err(e) = std::fs::rename(&temporary, &self.path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(e.to_string());
        }
        Ok(())
    }
    pub fn pc_id(&self) -> &str {
        &self.saved.pc_id
    }
    pub fn fingerprint(&self) -> String {
        crypto::hash(&self.key.public_key_der())
    }
    pub fn preferred_port(&self) -> u16 {
        self.saved.preferred_port
    }
    pub fn remember_port(&mut self, port: u16) -> Result<(), String> {
        let old = self.saved.preferred_port;
        self.saved.preferred_port = port;
        if let Err(e) = self.save() {
            self.saved.preferred_port = old;
            return Err(e);
        }
        Ok(())
    }
    pub fn device(&self) -> Option<DeviceInfo> {
        self.saved.paired.as_ref().map(|p| p.device.clone())
    }
    pub fn pair(&mut self, device: DeviceInfo, token: &str) -> Result<(), String> {
        if self.saved.paired.is_some() {
            return Err("alreadyPaired".into());
        }
        self.saved.paired = Some(Paired { device, token_hash: crypto::hash(token.as_bytes()) });
        if let Err(e) = self.save() {
            self.saved.paired = None;
            return Err(e);
        }
        Ok(())
    }
    pub fn authenticate(&self, device: &DeviceInfo, token: &str) -> bool {
        self.saved.paired.as_ref().is_some_and(|p| {
            p.device.id == device.id
                && p.device.platform == device.platform
                && crypto::equal(&p.token_hash, &crypto::hash(token.as_bytes()))
        })
    }
    pub fn forget(&mut self) -> Result<(), String> {
        let old = self.saved.paired.take();
        if let Err(e) = self.save() {
            self.saved.paired = old;
            return Err(e);
        }
        Ok(())
    }
    /// A certificate for the current interface, signed by the persistent pinned key.
    pub fn certificate_der(&self, ip: Ipv4Addr) -> Result<rustls::pki_types::CertificateDer<'static>, String> {
        let mut params = CertificateParams::new(vec![ip.to_string()]).map_err(|e| e.to_string())?;
        params.not_before = time::OffsetDateTime::now_utc() - time::Duration::days(1);
        params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);
        let cert = params.self_signed(&self.key).map_err(|e| e.to_string())?;
        Ok(cert.der().clone())
    }
    pub fn tls(&self, ip: Ipv4Addr) -> Result<Arc<rustls::ServerConfig>, String> {
        let cert = self.certificate_der(ip)?;
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(rustls::pki_types::PrivatePkcs8KeyDer::from(
            self.key.serialize_der(),
        ));
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .map_err(|e| e.to_string())?;
        Ok(Arc::new(config))
    }
}

pub struct Invite {
    secret: String,
    issued: Instant,
    attempts: u8,
    used: bool,
}
impl Invite {
    pub fn new(now: Instant) -> Self {
        Self { secret: crypto::secret(), issued: now, attempts: 0, used: false }
    }
    pub fn secret(&self) -> &str {
        &self.secret
    }
    pub fn redeem(&mut self, value: &str, now: Instant) -> bool {
        if self.used || self.attempts >= 5 || now.saturating_duration_since(self.issued) > Duration::from_secs(120) {
            return false;
        }
        if !crypto::equal(&self.secret, value) {
            self.attempts += 1;
            return false;
        }
        self.used = true;
        true
    }
}
