# Execution record

Base 02c5a9553c44478cb2e08590c6b026f2ff6ae732; feat/mobile-mirror. User approved the design and continuation; implementation inline.

- Core behavior missing-implementation RED → 5/5 GREEN.
- Transport/parser/lifecycle missing-implementation RED → 3/3 GREEN.
- Native pointer missing remote_events RED → host 249 passing, 44 existing ignored.
- Slow HTTP body shutdown reproduced RED → bounded-shutdown fix GREEN.
- UI baseline 510/510 and UI build passed; workspace cargo check passed.
- Native probe blocked by installed westplay.exe PID 31112; user's session left untouched.
- Final reviewer: initial-offer retry, touch scrolling, input overflow Important; readback sequence ordering Minor.
- Final: fixed retry — actual browser failed-offer regression RED→GREEN; now reuses token.
- Final: fixed scrolling — missing scrollDelta export RED→GREEN, real wheel input assertion passes.
- Final: fixed overflow — lifting_after_overflow_allows_the_next_gesture RED→GREEN; remote suite 11/11 passes.
- Final: readback ordering promoted for low-latency requirement; sequence guard and late-readback regression added.
- Ruling: explicit touch scroll mode preserves existing native map/slider dragging; cost is a mode toggle before ordinary-page scrolling.
- Ruling: require existing FFmpeg on PATH or beside app, hardware-only; avoids silently distributing a new third-party binary; cost is an explicit prerequisite on other PCs. NVENC probe succeeded here.
- Only task-generated target and crate registry caches moved to D: via junctions. Registry relocation interrupted one compiler (os267); a concurrent suite attempt reported windows_sys metadata mmap failure under disk/memory pressure. Final reruns use D: TEMP/TMP and two jobs; failed attempts are not counted as passing.
- Deferred minors: none after capture-sequence guard.

Final verification and package results are recorded in outputs/WestPlay-dogrulama.md. No push, publish or installation performed.

- Additional final regression: disconnect must restore editable empty invitation field. Browser RED reproduced, source-client real WebRTC smoke GREEN after reset fix.
- Existing setup names_a_folder_as_windows_does failure is due to SYSTEMROOT=C:\WINDOWS versus canonical C:\Windows; targeted test passes with only subprocess env normalized. Complete normalized suite queued for final evidence.

## Final evidence

- Full cargo test --workspace: 717 passed, 0 failed, 60 existing ignored (subprocess SystemRoot normalized to canonical casing).
- UI: 510 tests pass; build passes; lint 0 errors, 17 existing warnings. Phone geometry: 3 tests pass.
- Current cargo build --release -p westplay-app: exit 0. Full current phone script byte-for-byte embedded in the release.
- Final rebuilt synthetic executable + actual Chrome/NVENC integration smoke: PASS, including failed-offer retry, scroll, touch, text, cancel and new invitation editability.
- Native game/device validation remains explicitly unverified because the installed user's WestPlay is running.
- Portable archive checked for executable, companion/cabin UI and guide.

## 15 November: offline packaging correction

- User screenshot reproduced localhost:5173 ERR_CONNECTION_REFUSED in the delivered preview. The previous plain cargo release lacked Tauri custom-protocol; build success had not verified companion startup.
- Actual generated-context regression failed because embedded index.html was absent.
- Added app custom-protocol feature mapping, explicit root build flag and release build-script guards for the flag and built UI.
- Negative release test now stops with the explicit missing-feature error; packaged app tests: 6 passed, 0 failed, 1 existing ignored.
- Corrected cargo build --release -p westplay-app --features custom-protocol completed successfully.
- Replaced the extracted preview executable. Native UI verification selected the exact extracted process path, observed http://tauri.localhost/, the working companion and live native Mobile screen interface list while localhost:5173 was stopped. Left the corrected app open on Controls with mobile control off.
- Original installed app was briefly selected by launcher app-id resolution; closed that test launch and used the exact process: executable identifier to verify the corrected preview.
