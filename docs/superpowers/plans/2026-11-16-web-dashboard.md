# Web Dashboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Web-only vehicle information and opt-in nearby Convoy diagnostics around the existing live screen.
**Architecture:** Host publishes latest telemetry into remote service; authenticated POST snapshot returns expiring data. A bounded helper worker supplies a separate nearby plot, never touches cabin rendering.
**Tech Stack:** Existing Rust/Axum, vanilla JS/CSS, Windows C# observer.
**Spec:** ../specs/2026-11-16-web-dashboard-design.md

## Global Constraints
- APK and in-game UI unchanged. No live game access/install/start during implementation.
- Unknown remains null; no speed, RPM or gear cards.
- Snapshots 2 Hz, expire after 2 s; observer opt-in, 120 s maximum.
- Logs bounded, no credentials/names/pointers/absolute coordinates.

## Review Focus
- Stopped/frozen game clears data and markers.
- Invalid/ambiguous own identity never labels local vehicle as friend.
- Unauthorized endpoints expose no game data or controls.
- Small viewports retain video controls and readable state.
- Missing SDK channels are unavailable rather than healthy/off.

### Task 1: Vehicle data and snapshot
Files: shared lib/new web_vehicle module, plugin telemetry registration module, host lib, remote dashboard/control/server modules and tests.
- [x] Add failing nullable/freshness/channel-reset/auth tests.
- [x] Implement documented SDK fields with availability mask, publish at frame end, bridge v29, host feeds remote without altering cabin messages.
- [x] Pass shared/plugin/remote tests.

### Task 2: Experimental Convoy worker
Files: remote convoy module, LiveResearchReader.cs bridge name, tests.
- [x] Add failing bounds, own matching, missing sample, timeout/generation and privacy tests.
- [x] Implement opt-in hidden helper worker and latest-only reduction, bounded logs and authenticated stop/start.
- [x] Pass replay/native helper self-tests, never observe live.

### Task 3: Web instrument panels
Files: remote web dashboard.js/test.mjs, index.html, style.css, client.js and server asset routes.
- [x] Add failing formatting/unknown/stale/relative marker tests.
- [x] Implement responsive dark panels, nearby plot, state notices, start/stop/export and polling lifecycle without affecting touch mapping.
- [x] Pass web tests; inspect tablet/phone/desktop fixture in browser.

### Task 4: Build and deliver
Files: docs/web-dashboard.tr.md, distribution script, artifacts outside repo.
- [x] Run relevant complete test suites, format/diff checks, release build, verify unchanged cabin/APK.
- [x] Package runnable app/plugin plus reader/source/licenses in a new output folder, verify manifest and ZIP.
- [x] Commit implementation and report tested scope/unsupported coach fields.

## Execution ledger

User explicitly selected web-only side panels and asked to apply them. Work is on feature/web-dashboard; no game access, app launch or plugin installation was performed. Ruling: retain current checkout on its dedicated feature branch rather than create another worktree — preserves existing local build caches and approved workspace. Ruling: use a relative nearby plot, not inaccurate markers pasted onto an unknown video projection.

Implemented SDK optional wear/warnings and existing-channel availability separately from cabin defaults, Bridge v29; authenticated latest snapshot, experimental 120 s hidden reader, ownership reduction, capped diagnostics, responsive panels. In-game UI and Android source unchanged. Reviewer identified stale/unknown core values and overlapping observer generations; both fixed with availability and concurrent/cancelled ownership regressions. Helper self-test passes without game access. Eight web tests and 518 existing UI tests pass. Headless browser fixture verifies four viewport layouts, buttons, marker clearing and stale in-flight polling.

Full cargo test --workspace ran; 612 tests across successful binaries passed before setup's existing places::tests::names_a_folder_as_windows_does failed (86 additional setup tests passed, 1 failed, 8 ignored). It compares C:\Windows against C:\WINDOWS on this host. Its source is byte-equivalent to baseline e50e012, and has not been modified. This unrelated environment-sensitive assertion is reported, not silently fixed. Focused final native tests and release build are pending.

Final verification: focused host/plugin/shared/remote suite passed (350 tests), 518 UI tests passed, 8 web tests passed; native helper own-memory-only self-test passed. Release app/plugin/probe/maps built successfully. No cabin/Android source diffs. Full workspace retains the documented unrelated setup casing failure. New package generated; final archive/manifest verification follows. No merge/push or game launch performed.
