# WestPlay LAN Mobile Mirror Implementation Plan
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
**Goal:** Mirror the exact WestPlay texture to one phone on the same LAN and accept touch/text input without blocking the game frame pipeline.
**Architecture:** A new westplay-remote crate owns the opt-in LAN server, pairing, WebRTC session and hardware-only H264 subprocess. The host offers frames via a nonblocking GPU readback and latest-frame mailbox, drains validated remote input on its existing UI thread, and the companion controls lifecycle. The mobile HTML client is embedded in the crate, requires no external assets and receives the single authoritative PC session.
**Tech Stack:** Rust; Tokio/Axum; webrtc-rs; Windows D3D11; FFmpeg hardware H264 (NVENC, QSV, AMF or Media Foundation), HTML/CSS/JavaScript; existing React/Tauri companion.
**Spec:** ../specs/2026-11-14-mobile-mirror-design.md

## Global Constraints
- Windows 10/11; no cloud relay or internet access required at runtime; feature off by default.
- Same LAN; one controller; PC audio; 1280x720 30 FPS starting target; no guaranteed latency claim.
- Preserve existing game GPU pipeline; no encoding/network waits on host UI thread; bound frame and input queues.
- Require explicit pairing and expire credentials; validate Host/Origin; release held input on disconnect; never control OS-wide input.
- No push, publishing, game installation or blanket firewall changes.

## Review Focus
- A paired socket disappears during a drag: button must release and controller lease expire.
- Wi-Fi encoder congestion: bounded mailbox must replace old frames instead of growing queues.
- Portrait display and truck screen height changes: reject letterbox taps and use authoritative dimensions.
- Forged cross-origin requests, reused/expired pairing credentials and stale SDP: deny without replacing controller.
- Missing encoder or missing FFmpeg: report actionable error and leave game rendering functional.

### Task 1: Validated session and control core
**Files:** Create crates/remote/Cargo.toml, src/lib.rs, src/session.rs, src/input.rs, src/frame.rs; modify Cargo.toml.
**Interfaces:** RemoteControl::new(), configure(bind: Option<Ipv4Addr>), status() -> RemoteStatus, take_input() -> Vec<Input>, publish(Frame); all sync and bounded. RemoteStatus contains enabled, url, code, qr_svg, connected, error and metrics.
- [ ] Write failing unit tests: pairing expires at 120s, fifth wrong code locks session, exclusive one-controller, token invalid on stop, invalid/nonfinite coordinates denied, text bounded to 1024 UTF-8 bytes, latest-frame replacement and queue saturation produces release.
- [ ] Run cargo test -p westplay-remote; observe missing implementations before implementing session, input and mailbox behavior.
- [ ] Implement randomized tokens via OS-backed randomness, explicit lifecycle generation and bounded channels.
- [ ] Run crate tests and record result.

### Task 2: Real WebRTC transport and hardware video
**Files:** Create crates/remote/src/server.rs, peer.rs, encoder.rs, h264.rs, web/index.html, web/client.js, web/style.css; test integration via local HTTP and synthetic frames.
**Interfaces:** RemoteControl starts/stops a local IPv4 listener at selected address; authenticated offer endpoint returns SDP answer; data channel carries Input JSON; hardware encoder emits H264 access units into a WebRTC H264 static sample track.
- [ ] Write failing tests: fragmented Annex-B NALs preserve SPS/PPS and frame boundaries, no unbounded stale frames, valid Origin/Host required, unauthorized offers denied, stop closes listener and invalidates token.
- [ ] Implement static local assets; short-lived code exchange; one authenticated peer lease; local-only ICE; disconnect heartbeat/timeout and explicit release. Scope network candidates to chosen private IPv4 interface.
- [ ] Probe hardware encoders using a synthetic short frame sequence; use only successful hardware encoder, no software fallback. Pipe raw BGRA via worker with one pending frame; parse H264 access units and send sample track; drain RTCP and handle keyframe requests with short GOP.
- [ ] Implement client video, pairing, normalized pointer capture, wheel, native text field, full screen, reconnect and RTT display. Tests use actual protocol helpers and real HTTP instead of mocked success.
- [ ] Run cargo tests; exercise real WebRTC loopback with synthetic frames where feasible; record unavailable mobile hardware tests clearly.

### Task 3: Host GPU and input integration
**Files:** Modify crates/host/Cargo.toml, src/lib.rs, src/capture.rs, src/pointer.rs; create src/remote.rs.
**Interfaces:** Config gains Arc<RemoteControl>; Capture optionally copies final composed texture into a staging texture after successful game publish; nonblocking Map(DO_NOT_WAIT) exports latest BGRA with current screen crop/size metadata. Pointer::update accepts an optional remote absolute input state and shares existing embedded-page routing.
- [ ] Write failing tests for absolute coordinate conversion, source ownership across drag and releases on timeout/cancel, pointer input under camera cover and crop/height normalization.
- [ ] Implement GPU readback using its own two staging buffers, throttled to 30 FPS, never waiting for GPU completion. Do not hold game texture lock across CPU readback, encoding or network.
- [ ] Drain remote input on UI thread; route mouse and text exclusively to existing WebViews. Preserve desktop mouse behavior and embedded-page zoom/gesture targeting.
- [ ] Run host and workspace tests; compile Windows host.

### Task 4: Companion lifecycle, documentation and validation
**Files:** Modify app/src/main.rs, ui/src/companion/pages/Controls.tsx; create ui/src/companion/pages/MobileControl.tsx, docs/mobile-control.md; update dependencies and lockfiles.
**Interfaces:** Tauri mobile_status, mobile_interfaces, mobile_configure commands; feature disabled at startup; dropdown of private IPv4 interfaces, start/stop, QR/code, connected/error state; explicit FFmpeg requirement.
- [ ] Write failing integration tests for stop/restart credentials, pointer client geometry and cancel behavior.
- [ ] Add companion controls consistent with existing styles; show trust-LAN notice, no hidden firewall modifications, no secrets in logs. Native API returns only eligible private adapters.
- [ ] Document FFmpeg installation/runtime lookup, local build commands, supported gestures, trusted network assumption, firewall guidance, measured results and missing real-game/mobile verification.
- [ ] Run npm test --prefix ui, npm run build --prefix ui, cargo test --workspace, cargo check --workspace and relevant formatting/lint. Run a fresh whole-branch code review, fix critical/important issues and rerun affected checks.
- [ ] Deliver reviewable source and patch in outputs; report exact validation and limitations. Do not claim latency target achieved without measurements.

## Execution decisions
User approved design and said continue. Execute inline without another permission gate per session instructions. Managed worktree creation did not return; it was cancelled, no worktree appeared in git worktree list, and work proceeds on feat/mobile-mirror in the dedicated clean clone. Source base: 02c5a9553c44478cb2e08590c6b026f2ff6ae732. Rust toolchain is being installed under work/toolchains, not the user's home. Synthetic probe succeeded for h264_nvenc; other hardware backends failed on this machine. FFmpeg is already installed here; packaging it for end users is outside this source change and missing executable must be reported before enabling.
