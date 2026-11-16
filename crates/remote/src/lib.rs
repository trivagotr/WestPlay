mod control;
pub mod convoy;
mod convoy_tracker;
pub use convoy_tracker::Tracker;
pub mod dashboard;
mod encoder;
mod frame;
mod h264;
mod input;
mod peer;
mod server;
mod session;

pub use control::{RemoteControl, RemoteStatus, interfaces};
pub use frame::{Frame, Latest};
pub use h264::AccessUnits;
pub use input::{Input, InputQueue};
pub use session::Pairing;
use std::net::Ipv4Addr;

pub fn allowed_address(ip: Ipv4Addr) -> bool {
    ip.is_private()
}
pub fn valid_origin(host: &str, origin: &str) -> bool {
    origin == format!("http://{host}")
}

mod shell_style;
pub use shell_style::ShellStyle;
