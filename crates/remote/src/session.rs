use std::time::{Duration, Instant};

const CODE_LIFETIME: Duration = Duration::from_secs(120);
const TOKEN_LIFETIME: Duration = Duration::from_secs(12 * 60 * 60);

pub fn secret(bytes: usize) -> String {
    let mut data = vec![0u8; bytes];
    getrandom::fill(&mut data).expect("Windows random number generator unavailable");
    data.iter().map(|b| format!("{b:02x}")).collect()
}

/// Manual short-lived code and an unguessable, server-scoped QR invitation.
pub struct Pairing {
    code: String,
    invitation: String,
    issued: Instant,
    failures: u8,
    token: Option<String>,
}

impl Pairing {
    pub fn new(now: Instant) -> Self {
        let mut data = [0u8; 4];
        getrandom::fill(&mut data).expect("Windows random number generator unavailable");
        Self {
            code: format!("{:06}", u32::from_le_bytes(data) % 1_000_000),
            invitation: secret(24),
            issued: now,
            failures: 0,
            token: None,
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn invitation(&self) -> &str {
        &self.invitation
    }
    pub fn invitation_url(&self, base: &str) -> String {
        format!("{base}#pair={}", self.invitation)
    }

    pub fn pair(&mut self, code: &str, now: Instant) -> Result<String, &'static str> {
        if equal(code, &self.invitation) {
            if now.saturating_duration_since(self.issued) > TOKEN_LIFETIME {
                return Err("QR expired. Restart browser control on the PC.");
            }
            // Rescans/reloads reuse the existing session, without replacing an active peer.
            return Ok(self.token.get_or_insert_with(|| secret(32)).clone());
        }
        if self.token.is_some() {
            return Err("A phone is already paired. Restart mobile control on the PC to replace it.");
        }
        if self.failures >= 5 || now.duration_since(self.issued) > CODE_LIFETIME {
            return Err("Pairing expired or locked. Restart mobile control on the PC.");
        }
        if !equal(code, &self.code) {
            self.failures += 1;
            return Err("Incorrect pairing code.");
        }
        let token = secret(32);
        self.token = Some(token.clone());
        Ok(token)
    }

    pub fn authorized(&self, token: &str, now: Instant) -> bool {
        now.duration_since(self.issued) <= TOKEN_LIFETIME
            && self.token.as_deref().is_some_and(|expected| equal(expected, token))
    }

    pub fn reset(&mut self, now: Instant) {
        *self = Self::new(now);
    }
}

fn equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |v, (a, b)| v | (a ^ b)) == 0
}
