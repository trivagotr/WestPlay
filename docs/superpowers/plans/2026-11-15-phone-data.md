# Android Phone Data Implementation Plan
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Independent Android and frontend domains may use superpowers:dispatching-parallel-agents after the wire contract is frozen.
**Goal:** Real Android battery/charging/notifications appear on in-game WestPlay, including while phone screen is off, with automatic port discovery and a platform-neutral iPhone-ready contract.
**Architecture:** A new westplay-phone Rust library owns persistent PC identity, TLS WebSocket phone service, pairing and mDNS. Android foreground + notification listener services send typed events; host forwards validated updates to the existing cabin WebView. Existing WebRTC mirror stays independent.
**Tech Stack:** Rust/Tokio/rustls/tungstenite/rcgen/mdns-sd; Windows DPAPI; Java17 Android SDK36; React/Zustand; existing Tauri.
**Spec:** ../specs/2026-11-15-phone-data-design.md
**Wire contract:** ../../../protocol/phone-v1.md

## Global constraints
- Android8/API26+, compile/target SDK36. Only native companion APK in first release; iPhone-ready protocol and capability rendering, no promised iOS feature parity.
- Data link independent from video/FFmpeg; private LAN only; explicit user-started service; no cloud/router/firewall/system-privacy changes.
- Persistent port + mDNS discovery by PC identity; TLS key pinned from first QR; never trust mDNS for identity.
- DPAPI protects PC private key; Android Keystore protects stored credentials; token hashes only on PC; no real credentials in logs/source.
- 120-second single-use invite, one paired phone, explicit forget revocation, max8KiB frame, bounded notification queue3, body500/title160 characters, 15-second disconnect detection.
- Build Windows with custom-protocol; preserve the offline-assets regression and reset computer-use kernel after any native UI capture.

## Review focus
- Port/IP changes with same PC identity reconnect without re-pairing; forged discovery with wrong TLS key fails.
- Screen-off/background delivery is required; if no real-device/owned-emulator proof exists, label APK preview and do not claim that acceptance passed.
- Permission loss and disconnect clear stale battery/toasts; duplicate notification updates never flood the screen.
- Existing mobile mirror pairing and keyboard controls continue working, independent from data-only mode.
- Secrets, malformed Unicode/oversized payloads, expired invites and another phone cannot bypass lease/validation.

### Task 1 — Shared protocol and native service (root)
Files: protocol/phone-v1.md, protocol/fixtures/*, crates/phone/{Cargo.toml,src/*,tests/*}; root Cargo.toml/Cargo.lock.
Interface: PhoneBridge::new(PathBuf)->Result<Self,String>, disabled()->Self, configure(Option<Ipv4Addr>)->Result<(),String>, status()->PhoneStatus, renew_invite(), forget(), updates_after(u64)->Option<PhoneUpdate>, snapshot()->PhoneState. PhoneStatus defined in wire contract.
- [ ] Write tests first for event parsing/limits, battery null/0, duplicate sequence, old/new notifications, expiry/one-device auth, persistence and ephemeral port change identity.
- [ ] Implement state/auth store; DPAPI secret wrapper; stable key certificate generation; preferred-port fallback and mdns advertisement.
- [ ] Implement TLS WebSocket /phone handshake and events; authenticate before accepting event data; ack valid sequence; heartbeat timeout/revoke/stop; bound handshake count and frames.
- [ ] Verify real pinned TLS loopback, wrong-key rejection, token revocation, stop cleanup and nonblocking update consumption. Run cargo test -p westplay-phone.

### Task 2 — Android app (independent Android owner)
Files owned: android/** only; use shared wire docs/fixtures read-only. No other agent touches this subtree.
Interface: parse westplay://phone URI, pinned WSS /phone, pair/auth request + ready response, event envelope and NsdManager _westplay._tcp. lookup, persistent PC id/pin/token, Java app.westplay.phone.
- [ ] Write JVM tests for QR/endpoint validation, known-PC discovery endpoint changes, credential redaction and protocol encoding.
- [ ] Implement connection/permissions activity, QR scan + paste fallback, Android Keystore credential store, discovery/reconnect manager and connectedDevice foreground service.
- [ ] Add battery broadcasts and NotificationListenerService with app filters, own/ongoing/secret exclusion, optional body sharing, permission status, remove/update events and test-notification button.
- [ ] Add app-specific background guidance; no auto-grant or global settings changes. Retain user-started background link screen-off.
- [ ] Run tests/build APK using JDK17, SDK D:/Android/Sdk and workspace-only Gradle/cache/signing paths. Never install onto a user's physical phone or modify its permissions without user handoff. An owned test emulator may be used if available.

### Task 3 — Cabin/companion UI (independent frontend owner)
Files owned: ui/src/shared/phone.ts, ui/src/cabin/phone/**, ui/src/cabin/CabinApp.tsx, ui/src/cabin/shell/Sidebar.tsx, ui/src/shared/host.ts, ui/src/companion/pages/PhoneDataControl.tsx, ui/src/companion/pages/Controls.tsx and ui/src/companion/companion.css. Do not change Rust/Android.
Interface: HostMessage phoneState {state:PhoneState} and phoneNotification {event:PhoneNotificationEvent}. Tauri phone_status, phone_interfaces, phone_configure {address:string|null}, phone_forget, phone_renew.
- [ ] Write failing pure reducer tests for low battery dedupe/reset, null vs0, disconnect cleanup, capability handling including iOS fixture, max3 queue, notification removal/update.
- [ ] Add actual phone status replacing fake5G, charging/battery glyph and six-second game notification cards with text-only rendering.
- [ ] Companion Phone data group: native start/stop/network selection, QR and full pairing URI, registered phone/connection/capabilities/error, renew/forget actions. Preview shows a clear native-only notice.
- [ ] Run UI test/build/lint and headless browser visual check; no desktop computer capture is needed.

### Task 4 — Root integration, review and delivery
Files: crates/host/Cargo.toml/src/lib.rs/src/main.rs/examples/mobile_probe.rs, app/src/main.rs, docs/phone-data.md, outputs deliverables.
- [ ] Integrate PhoneBridge in Tauri setup/config and host loop. UI reload gets a fresh snapshot; update generation prevents per-frame polling messages. No network or JSON decode waits on render thread.
- [ ] Validate end-to-end synthetic phone events through real TLS into cabin message handler; check trusted identity survives service restart/port conflict.
- [ ] Run appropriate Rust/UI/app production-feature checks, Android tests/APK build; receive one fresh whole-feature code review and fix important findings with regression tests.
- [ ] Build corrected Windows portable ZIP + APK + source/patch/guide. Explicitly report missing physical background/Doze/game latency tests; do not imply they passed.

## Execution choices
User approved native APK and background behavior. Execute without another approval gate. Dedicated clean clone work/WestPlay is already isolated from the installed app; create feat/android-phone-data in that checkout rather than invoking the previously nonresponsive managed-worktree API. No push/publish/installation on physical phones. All task progress and rulings go in 2026-11-15-phone-data-execution.md.
