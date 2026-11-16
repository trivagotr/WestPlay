# Coaches telemetry and realistic web instruments

User requests actual passenger data and automotive symbols for cruise/lane/brake/etc, web only. Existing live GPS projection, APK and in-game visuals stay unchanged.

Evidence: SDK1.16 public headers omit bus channels. Static inspection of fingerprinted installed ETS2 1.62.3.3 executable found bus_job.average_satisfaction adjacent to official SDK channel names; its getter returns the average passenger float. Register this channel through official SDK API (float type5), never call internal code or read live process memory. Registration success and live value must be verified before claiming passenger support. Lane state is not exposed in currently identified SDK names, so its symbol stays visibly unavailable.

- [x] Test nullable satisfaction normalization, identity/freshness and independent read-only mapping.
- [x] Implement plugin SDK registration and separate Local\WestPlay.CoachTelemetry.v1 sample mapping. Keep primary Bridge v29 unchanged to avoid losing game detection with older plugins. Host reads only matching PID and advancing samples.
- [x] Test automotive state semantics and null/zero distinction. Replace text rows with accessible custom SVG telltales (green cruise/beam/turn, blue high beam, red parking brake, amber/red warnings), subdued unknown and text labels. No speed/RPM/gear cards.
- [x] Show actual average satisfaction percent; distinguish not installed / unsupported / waiting for a passenger job / live data. Never invent happiness.
- [x] Run native/UI/web tests and responsive browser checks, build package, then request one controlled game restart to install/test new DLL. Back up first, no game file changes while game runs.
- [x] Validate SDK registration/value against user's Coaches/F3 display and deliver verified package or explicit remaining limitation.

Implementation: separate per-PID read-only transport with main bridge unchanged; real SDK float registration plus no_value flag, verified by simulated SDK available→unavailable regression. Reviewer caught missing null flag; fixed red→green. 356 native and 10 web tests plus responsive browser/color/null tests pass. Release app and DLL built. Live registration/F3 comparison remains pending before success claim.

Live acceptance: with game and WestPlay closed, old plugin backed up, new DLL installed/verified and new packaged app started. User opened a Coaches passenger job and reported F3=52. SDK registration returned0; read-only sample matched process, supported=known=1, raw0.5166412, percent51.6641 and advancing counter; displayed integer52 matches. Host recognized official MAN coach. Main bridge remains29 and APK unchanged. Standalone package source/guide/manifest synchronization and final archive verification follows.
