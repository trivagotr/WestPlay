# Fill viewer and friend package validation

- Geometry: five tests pass. New fill tests first failed on old letterboxed mapping, then passed with axis-specific mapping; existing contain behavior retained.
- Real browser/WebRTC hardware-encoded stream: fullscreen and landscape viewport sizing, fill/fit edges, letterbox exclusion, touch, scroll, text, cancellation, preference storage and blocked storage all pass.
- Rapid reconnect regression: old behavior failed during authenticated temporary409 and superseded attempt; bounded offer-only retry and stale-attempt guards now pass against a held HTTP offer boundary. No input replay or credential retry added.
- Independent code review completed for viewer and distribution; identified reconnect overlap addressed with deterministic regression.
- Launcher: scripts parse; original FFmpeg archive, exact entries and archive/exe SHA256 verified; official Microsoft WebView2 GUID verified; signed VC++ installer downloaded and authenticated but not run.
- Safe launcher fast path verified with an invisible temporary helper and correct working directory; no actual app, download or installer executed in that test.
- Release binaries require x64 VC++ for probe/maps, so setup handles it. Four required release targets built successfully; final app rebuild incorporates reconnect fixes. PDB filename collision warning concerns unshipped debug symbols, not runtime binaries.
- Android APK unchanged from verified16-test build. Physical Android background/Doze acceptance and fresh friend-PC installer interaction remain pending.

Final archive contents and hashes are recorded with the output artifact after creation.
