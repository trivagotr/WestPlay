use crate::{AccessUnits, control::Run};
use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

fn ffmpeg() -> std::path::PathBuf {
    let beside = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.join("ffmpeg.exe")));
    beside.filter(|p| p.is_file()).unwrap_or_else(|| "ffmpeg".into())
}
fn command() -> Command {
    let mut c = Command::new(ffmpeg());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000);
    }
    c.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
    c
}
fn options(c: &mut Command, codec: &str) {
    c.args([
        "-an",
        "-c:v",
        codec,
        "-pix_fmt",
        "nv12",
        "-profile:v",
        "baseline",
        "-level:v",
        "3.1",
        "-g",
        "30",
        "-bf",
        "0",
        "-b:v",
        "4000k",
    ]);
    match codec {
        "h264_nvenc" => {
            c.args(["-preset", "p1", "-tune", "ull", "-zerolatency", "1", "-rc", "cbr"]);
        }
        "h264_qsv" => {
            c.args(["-preset", "veryfast", "-async_depth", "1", "-look_ahead", "0"]);
        }
        "h264_amf" => {
            c.args(["-usage", "ultralowlatency", "-quality", "speed"]);
        }
        "h264_mf" => {
            c.args(["-hw_encoding", "1", "-scenario", "display_remoting"]);
        }
        _ => unreachable!("only hardware encoders may be selected"),
    }
    c.args(["-bsf:v", "h264_metadata=aud=insert", "-flush_packets", "1", "-f", "h264", "pipe:1"]);
}

pub fn probe() -> Result<String, String> {
    for codec in ["h264_nvenc", "h264_qsv", "h264_amf", "h264_mf"] {
        let mut c = command();
        c.args(["-f", "lavfi", "-i", "color=size=1280x720:rate=30", "-frames:v", "2"]);
        options(&mut c, codec);
        let mut child = c.stdout(Stdio::null()).stderr(Stdio::null()).spawn()
            .map_err(|e| format!("FFmpeg is required for mobile video. Install FFmpeg with hardware H.264 support or place ffmpeg.exe beside WestPlay. ({e})"))?;
        let deadline = Instant::now() + Duration::from_secs(4);
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(codec.into()),
                Ok(Some(_)) | Err(_) => break,
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
            }
        }
    }
    Err("No working hardware H.264 encoder found (NVIDIA, Intel, AMD or Media Foundation). Update the GPU driver and use a hardware-enabled FFmpeg build. Software encoding is not enabled.".into())
}

pub struct Encoder {
    child: Child,
}
impl Encoder {
    pub fn start(state: Arc<Run>) -> Result<(Self, mpsc::Receiver<Vec<u8>>), String> {
        let mut c = command();
        c.args([
            "-f",
            "rawvideo",
            "-pixel_format",
            "bgra",
            "-video_size",
            "1280x720",
            "-framerate",
            "30",
            "-i",
            "pipe:0",
        ]);
        options(&mut c, &state.encoder);
        let mut child =
            c.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|e| e.to_string())?;
        let mut stdin = child.stdin.take().ok_or("Encoder stdin unavailable")?;
        let mut stdout = child.stdout.take().ok_or("Encoder stdout unavailable")?;
        let (tx, rx) = mpsc::channel(2);
        let input_state = state.clone();
        std::thread::Builder::new()
            .name("westplay-video-input".into())
            .spawn(move || {
                let mut last = None;
                let period = Duration::from_micros(33_333);
                while !input_state.stop.load(Ordering::Acquire) && input_state.connected.load(Ordering::Acquire) {
                    let begin = Instant::now();
                    if let Some(frame) = input_state.frame.lock().unwrap().take()
                        && frame.width == 1280
                        && frame.height == 720
                        && frame.bgra.len() == 1280 * 720 * 4
                    {
                        last = Some(frame);
                    }
                    if let Some(frame) = &last
                        && stdin.write_all(&frame.bgra).is_err()
                    {
                        break;
                    }
                    std::thread::sleep(period.saturating_sub(begin.elapsed()));
                }
            })
            .map_err(|e| e.to_string())?;
        std::thread::Builder::new()
            .name("westplay-video-output".into())
            .spawn(move || {
                let mut parser = AccessUnits::default();
                let mut buffer = [0; 32 * 1024];
                while let Ok(n) = stdout.read(&mut buffer) {
                    if n == 0 {
                        break;
                    }
                    for unit in parser.push(&buffer[..n]) {
                        if tx.blocking_send(unit).is_err() {
                            return;
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok((Self { child }, rx))
    }
}
impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
