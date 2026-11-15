//! Native WebView2 -> D3D11 -> hardware H264 -> WebRTC probe, without game files.
//! cargo run -p westplay-host --example mobile_probe -- <status.json> <profile-dir>
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use westplay_host::{Config, Content, RemoteControl, SCREEN_HEIGHT, SCREEN_WIDTH};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let output = args.get(1).ok_or("Pass status.json and a scratch profile directory")?.clone();
    let profile = args.get(2).ok_or("Pass a scratch WebView profile directory")?;
    let _instance = westplay_host::Instance::claim().ok_or("Close the running WestPlay before the native probe")?;
    let remote = Arc::new(RemoteControl::new());
    let address = westplay_host::mobile_interfaces().first().ok_or("No private IPv4 interface")?.address;
    remote.configure(Some(address))?;
    std::fs::write(&output, serde_json::to_vec(&remote.status())?)?;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let stop_path = format!("{output}.stop");
    std::thread::spawn(move || {
        let began = Instant::now();
        while began.elapsed() < Duration::from_secs(120) && !std::path::Path::new(&stop_path).exists() {
            std::thread::sleep(Duration::from_millis(100));
        }
        stop_thread.store(true, Ordering::Release);
    });
    let config = Config {
        phone: Arc::new(westplay_host::PhoneBridge::disabled()),
        remote: remote.clone(), width: SCREEN_WIDTH, height: SCREEN_HEIGHT,
        content: Content::Html(r#"<!doctype html><meta charset="utf-8"><style>html,body{margin:0;width:1280px;height:720px;background:#12202b;color:white;font:40px sans-serif}button{position:absolute;left:480px;top:260px;width:320px;height:200px;border:0;border-radius:20px;background:#c03070;color:white;font:36px sans-serif}</style><h1>WestPlay native capture probe</h1><button onclick="this.style.background='#30c060';this.textContent='Touch received'">Tap here</button>"#.into()),
        data_dir: profile.into(), open_request: Arc::default(), accounts: Arc::default(),
        settings: Arc::new(westplay_host::settings::SharedSettings::default()),
        on_settings: Box::new(|_| {}), customize_changed: Arc::default(), game: Arc::default(), map_builder: None,
        on_status: Box::new(|status| { if let Some(error) = &status.error { eprintln!("native probe: {error}"); } }),
    };
    let result = westplay_host::run(config, stop);
    remote.configure(None)?;
    let _ = std::fs::remove_file(output);
    result?;
    Ok(())
}
