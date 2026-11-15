# Web GPS friend markers correction

User rejected the radar and explicitly requested friends on the existing GPS like TruckersMP. Replace the radar; keep APK and in-game visuals unchanged. The existing video stays the GPS/route authority. Add an invisible coordinate-projection bridge to the existing MapLibre instance: project only verified peer world points using the actual camera, normalize them to captured page coordinates, return only screen points to the web client. Do not add map sources/layers/markers to the cabin instance.

- [x] Add projection validation, stale/source/response tests, true MapLibre projection and contain/fill mapping tests.
- [x] Keep peer world positions internal, correlate projection requests to observer generation/sample, reject late/invalid replies.
- [x] Add passive native page message projection handler using current live MapLibre camera; suppress hidden, offscreen and occluded positions.
- [x] Host requests at 10 Hz only for connected web peers and current candidates; data channel sends latest screen projection, no stale backlog.
- [x] Replace radar with pointer-transparent clipped SVG markers over video, clear within 350 ms if projection stops, preserve controls/focus/zoom behavior.
- [x] Run focused Rust/UI/web and browser integration tests, build release UI/native, package separately and verify hashes.

Existing Convoy data limitations and unsupported Coaches telemetry remain; never certify complete roster. Do not start/read the live game while developing. Existing baseline setup casing test failure remains reported.

Execution: passive projection only, no cabin source/layer/marker drawing changes. Review found duplicate lifetime extension and reliable-channel backlog; fixed by consuming reply IDs once, rejecting equal IDs in browser, deduplicating publications and using dedicated unordered map-overlay data channel with maxPacketLifeTime 150 ms. Actual MapLibre browser fixture passed camera changes, containment/fill, duplicate expiry and unchanged map-layer count. Responsive real shell passed with no radar and pointer-transparent video markers. UI build passed, full UI 520 and web 8 tests passed; host+remote test suite passed. Release/package verification follows.
