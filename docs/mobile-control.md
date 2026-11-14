# Mobile screen preview

Opt-in mirroring of WestPlay's existing Windows/WebView2 screen to one phone on the same LAN. Audio stays on the PC. Mobile control is disabled on each startup. This is a development preview.

## Requirements and connection

Windows 10/11, working WestPlay/WebView2, a private IPv4 LAN and a browser supporting WebRTC H.264 are required. PC Ethernet and phone Wi-Fi work if both can reach each other. Guest-network isolation prevents connection.

FFmpeg must be on PATH or ffmpeg.exe beside westplay.exe. A hardware-enabled build is required. WestPlay probes NVIDIA NVENC, Intel Quick Sync, AMD AMF and Windows Media Foundation. There is no silent software fallback.

1. Close the previously running WestPlay yourself before starting this preview. Do not run two hosts at once.
2. Open Controls → Mobile screen in the companion, select the private IPv4 interface used by your phone and enable the feature.
3. If Windows requests network access, restrict it to your trusted Private network. No firewall/router rules are automatically created.
4. Scan the QR or enter the HTTP address. Enter the PC's six-digit code within two minutes; five wrong attempts lock it.
5. Use landscape for more screen area. Tam ekran requests full screen where supported.
6. Turn the PC switch off to close the server and revoke the session. Toggle off/on for a fresh invitation or another phone.

Only one phone is paired. Retries reuse its in-memory token. Reloading/closing the page loses the token and requires a new invitation. An initial failed offer can be retried without re-pairing.

## Controls

With Kaydırma: kapalı, tap, hold and drag control the screen, map and sliders. For an ordinary list or embedded webpage, turn Kaydırma: açık on and swipe vertically; it sends wheel input without holding a mouse button. Turn it off to tap or drag again.

Klavye opens a phone text field. Select the intended field on the mirrored screen first, type, then press Gönder. Sil sends Backspace; Enter submits. Each send is limited to 1024 UTF-8 bytes. Audio stays on the PC; multi-touch/pinch and entire-desktop control are outside this preview.

A held gesture retains its input source. Cancellation, page hiding, connection loss and service shutdown release remote buttons. An overloaded input queue cancels the incomplete gesture and recovers when the finger lifts.

## Network and performance

The existing game GPU pipeline stays intact. Separate staging textures use nonblocking Map; CPU readback is outside the game texture lock. A bounded latest-frame mailbox prevents unlimited old raw images; monotonic sequence numbers prevent late readbacks replacing newer frames. Hardware H.264 starts at 1280×720, 30 FPS, without audio. WebRTC carries video and ordered DataChannel messages carry input.

No STUN, TURN, cloud relay or UPnP is used. The server binds and advertises ICE on the selected private IPv4 interface. Host/Origin checks, body/input limits, short-lived code, five-attempt lockout, token expiry and one controller constrain access. Temporary credentials are not logged.

Use only a trusted home LAN: the local HTTP invitation/signaling exchange can be intercepted on an untrusted network. WebRTC media/data transport is encrypted. Do not expose the port to the internet.

The RTT label measures a control-channel round trip, not touch-to-visible-result delay. 100 ms or below on a good LAN is an engineering target, not a guarantee. Router, GPU, decoder and display all contribute. Real-game FPS impact and real-phone latency remain to be measured.

## Build and test

From the root with stable Rust and Node.js >=24.15:

    npm ci --prefix ui
    npm run build --prefix ui
    cargo test --workspace
    node --test crates/remote/web/geometry.test.mjs
    cargo build --release -p westplay-app

Portable layout: westplay.exe, adjacent ui/ containing ui/dist contents, optionally ffmpeg.exe, plus the project's existing LICENSE and third-party notices. The rest of setup/maps/updater behavior follows the original app.

Optional real hardware + browser integration test:

    cargo build -p westplay-remote --example synthetic
    # Install Playwright separately and set PLAYWRIGHT_MODULE to its installation if needed.
    # Set CHROME_PATH to an installed Chrome executable if using that browser.
    node tools/mobile/smoke.cjs

This tests actual hardware encode/browser decode, deliberately failed initial offer and retry, letterbox rejection, down/up, scrolling, text, cancel and Origin rejection. Synthetic colors do not prove game integration. Scratch credentials are removed on exit.

Native capture probe (close installed WestPlay first):

    cargo run -p westplay-host --example mobile_probe -- <scratch-status.json> <scratch-profile-directory>

It exercises WebView2→D3D11→WebRTC without game-file changes. It stops after two minutes or upon creation of <scratch-status.json>.stop.

## Validation limits

Windows source checks and desktop Chrome decoding of real NVENC 1280×720 video have been exercised. Installed WestPlay was already running, so the separate native probe was blocked and the user's instance was left untouched. ETS2/ATS capture during gameplay, embedded media in a real game, Android/iPhone behavior, Wi-Fi performance and actual end-to-end latency remain unverified. Existing interactive/network-dependent ignored repository tests keep that status.
