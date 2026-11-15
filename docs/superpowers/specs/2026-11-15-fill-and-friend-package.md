# Mobile fill view and shareable friend package

User request: enlarge the existing web mirror by stretching toward the sides; then prepare APK and Windows runnable distribution for a friend.

Design: keep the existing captured image and low-latency WebRTC transport. Default to Fill (object-fit: fill), with a one-tap Fit mode preserving aspect ratio. Fill may distort proportions, but never crops controls. Use all available viewport space, including fullscreen. Touch mapping must match the visible rectangle in both modes; cancel active touch before switching; retain existing letterbox exclusion in Fit. Persist preference locally when browser storage is available.

Distribution: one ZIP with Android APK and Windows runnable bundle, game DLL, probe/map helper, UI resources, FFmpeg and required third-party notices/source, plus first-run Turkish steps. No user pairing credentials, signing keys or developer filesystem paths. Game installation and notification access remain explicit first-run user actions. WebView2 prerequisite must be handled or clearly surfaced. No hardware/Doze/latency claims without actual phone test.
