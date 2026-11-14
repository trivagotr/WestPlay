//! Hardware/WebRTC smoke test without starting a game or touching game files.
//! cargo run -p westplay-remote --example synthetic -- <status.json>
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use westplay_remote::{Frame, RemoteControl, interfaces};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output =
        std::env::args().nth(1).ok_or("Pass a local status.json path (contains the temporary pairing code)")?;
    let remote = Arc::new(RemoteControl::new());
    let address = interfaces().first().ok_or("No private IPv4 interface")?.address;
    remote.configure(Some(address))?;
    std::fs::write(&output, serde_json::to_vec(&remote.status())?)?;
    let begin = Instant::now();
    while begin.elapsed() < Duration::from_secs(120) {
        if std::path::Path::new(&format!("{output}.stop")).exists() {
            break;
        }
        if remote.wants_frames() {
            let mut bgra = vec![0u8; 1280 * 720 * 4];
            for y in 0..720 {
                for x in 0..1280 {
                    let i = (y * 1280 + x) * 4;
                    bgra[i..i + 4].copy_from_slice(&[
                        if x < 640 { 30 } else { 180 },
                        if y < 360 { 190 } else { 40 },
                        60,
                        255,
                    ]);
                }
            }
            remote.publish(Frame { bgra, width: 1280, height: 720 });
        }
        for input in remote.take_input(Instant::now()) {
            // Only event kinds, never entered text or session credentials.
            let kind = match input {
                westplay_remote::Input::Down { .. } => "down",
                westplay_remote::Input::Up { .. } => "up",
                westplay_remote::Input::Move { .. } => "move",
                westplay_remote::Input::Cancel => "cancel",
                westplay_remote::Input::Wheel { .. } => "wheel",
                _ => "text",
            };
            println!("input: {kind}");
        }
        std::thread::sleep(Duration::from_millis(33));
    }
    remote.configure(None)?;
    let _ = std::fs::remove_file(output);
    Ok(())
}
