//! Host-owned reader lifetime, independent of HTTP and WebRTC clients.
use crate::convoy::Observer;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Default)]
pub struct Tracker {
    pub(crate) observer: Arc<Observer>,
    target: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}
impl Tracker {
    pub fn update_pid(&self, pid: Option<u32>) {
        let pid = pid.unwrap_or(0);
        if self.target.swap(pid, Ordering::AcqRel) != pid {
            self.observer.stop();
            if pid == 0 {
                self.observer.data.lock().unwrap().clear("waiting-game");
            }
        }
        if pid == 0 {
            return;
        }
        let mut thread = self.thread.lock().unwrap();
        if thread.is_some() {
            return;
        }
        let observer = self.observer.clone();
        let target = self.target.clone();
        let stop = self.stop.clone();
        *thread = std::thread::Builder::new()
            .name("westplay-convoy".into())
            .spawn(move || {
                if let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    runtime.block_on(worker(observer, target, stop));
                }
            })
            .ok();
        if thread.is_none() {
            self.observer.data.lock().unwrap().clear("reader-start-failed");
        }
    }
    pub fn native_snapshot(&self) -> serde_json::Value {
        self.observer.data.lock().unwrap().native_snapshot(Instant::now())
    }
}
impl Drop for Tracker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.observer.stop();
        if let Some(thread) = self.thread.get_mut().unwrap().take() {
            let _ = thread.join();
        }
    }
}
fn refusal(reason: &str) -> (&'static str, bool) {
    if reason.contains("fingerprint") || reason.contains("signature") || reason == "locator-mismatch" {
        ("unsupported", true)
    } else if matches!(reason, "wrong-process" | "invalid-parent-pid" | "invalid-duration-or-architecture") {
        ("reader-refused", true)
    } else {
        ("reader-refused", false)
    }
}
async fn worker(observer: Arc<Observer>, target: Arc<AtomicU32>, stop: Arc<AtomicBool>) {
    let mut blocked_pid = 0;
    while !stop.load(Ordering::Acquire) {
        let pid = target.load(Ordering::Acquire);
        if pid == 0 {
            blocked_pid = 0;
        }
        if pid == 0 || pid == blocked_pid {
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        }
        let Some(generation) =
            observer.begin_when(|| target.load(Ordering::Acquire) == pid && !stop.load(Ordering::Acquire))
        else {
            continue;
        };
        let path = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.join("WestPlay.ConvoyReader.exe")));
        let Some(path) = path.filter(|p| p.is_file()) else {
            observer.with_current(generation, |d| d.clear("reader-missing"));
            blocked_pid = pid;
            continue;
        };
        let mut command = tokio::process::Command::new(path);
        command
            .args(["--follow", "--pid", &pid.to_string(), "--parent-pid", &std::process::id().to_string()])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        // Recheck cancellation immediately before spawn; updates invalidate all subsequent publications.
        if target.load(Ordering::Acquire) != pid || stop.load(Ordering::Acquire) {
            continue;
        }
        let Ok(mut child) = command.spawn() else {
            observer.with_current(generation, |d| d.clear("reader-start-failed"));
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        };
        if let Some(stdout) = child.stdout.take() {
            let mut lines = BufReader::new(stdout).lines();
            let mut tick = tokio::time::interval(Duration::from_millis(250));
            loop {
                tokio::select! {
                    line = lines.next_line() => {
                        if target.load(Ordering::Acquire) != pid || stop.load(Ordering::Acquire) { break; }
                        let Ok(Some(line)) = line else { break; };
                        if line.len() > 65536 { break; }
                        let keep = observer.with_current(generation, |d| match serde_json::from_str::<serde_json::Value>(&line) {
                            Ok(v) if v["status"] == "candidate-sample" || v["status"] == "sample-refused" => {d.accept(&v, Instant::now()); true},
                            Ok(v) if v["status"] == "refused" => {
                                let reason = v["reason"].as_str().unwrap_or("reader-refused");
                                let (status, terminal) = refusal(reason);
                                d.clear(status); if terminal { blocked_pid = pid; } false
                            },
                            Ok(v) if v["status"] == "observation-finished" => false,
                            Ok(_) => true,
                            Err(_) => {d.clear("reader-invalid-output"); false}
                        });
                        if keep != Some(true) { break; }
                    },
                    _ = tick.tick() => {
                        if stop.load(Ordering::Acquire) || target.load(Ordering::Acquire) != pid || !observer.is_current(generation) { break; }
                    }
                }
            }
        }
        let _ = child.kill().await;
        let _ = child.wait().await;
        observer.with_current(generation, |d| {
            if matches!(d.status.as_str(), "starting" | "observing" | "sample-unavailable") {
                d.clear("retrying");
            }
        });
        // Interruptible backoff; changing game never waits behind a previous game's failure.
        for _ in 0..8 {
            if stop.load(Ordering::Acquire) || target.load(Ordering::Acquire) != pid {
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        if target.load(Ordering::Acquire) != pid {
            blocked_pid = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_build_is_terminal_but_temporary_access_failure_retries() {
        assert_eq!(refusal("unsupported-image-fingerprint"), ("unsupported", true));
        assert_eq!(refusal("read-access-denied"), ("reader-refused", false));
    }
    #[test]
    fn fresh_native_state_exists_without_a_web_server() {
        let tracker = Tracker::default();
        let now = Instant::now();
        tracker.observer.data.lock().unwrap().accept(&serde_json::json!({"status":"candidate-sample","vehicles":[{"id":"aaaaaaaaaaaa","x":1.0,"z":2.0},{"id":"bbbbbbbbbbbb","x":10.0,"z":2.0}],"ownTelemetry":{"x":1.0,"z":2.0}}), now);
        assert_eq!(tracker.native_snapshot()["players"].as_array().unwrap().len(), 1);
        // Neither an HTTP listener nor a browser is needed to receive host data.
        tracker.target.store(100, Ordering::Release);
        tracker.update_pid(None);
        assert!(tracker.native_snapshot()["players"].as_array().unwrap().is_empty());
    }
}
