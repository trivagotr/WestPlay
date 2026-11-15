# Convoy research diagnostics

These are isolated research tools, not a released online-player map feature. Main WestPlay source, ABI and runtime packages are unchanged. The original probe.mjs and observation.mjs are file-only; the separate opt-in Windows observer described below can read one fingerprinted game process. None installs a plugin, modifies game memory, calls game code, injects anything or starts the game.

## Run without the game

Node.js 20+ is sufficient. No dependencies or native build are needed.

From the repository root:

- Synthetic demonstration: node tools/convoy/probe.mjs --demo
- Safety/decoder tests: node --test tools/convoy/probe.test.mjs
- Optional fabricated JSON fixture: node tools/convoy/probe.mjs --fixture synthetic.json

Only files marked westplay-convoy-synthetic-v1, synthetic:true and windows-1.60-unverified are accepted. Other arguments, including --pid and --attach, are refused. A synthetic:true tag describes the caller's test data; it does not authenticate provenance or certify compatibility. The tool has no live reader to invoke even if a file is mislabeled.

## What it demonstrates

The candidate layout comes from MIT-licensed ETS2LA source pinned to c925bd965a96c58b73eaea9ca033fe0f0dc623a7. Synthetic bytes exercise unique RIP-relative signature resolution; bounded pointer traversal; Windows array headers; sector-to-world conversion; own-object exclusion; and fail-closed invalid data handling. The reference sector conversion is worldX=512*cellX+localX and worldZ=512*cellZ+localZ. Pointer values and account identifiers are not emitted.

Every result explicitly says liveAccess:false and verifiedForLiveGame:false. Remaining rows are unverified-vehicle candidates, not confirmed online people. Ordinals are temporary fixture indexes, not stable player identities. No heading is inferred from the quaternion until it is compared against real game orientation.

Some upstream layouts are annotated 1.60. They are NOT validated for the previously observed ETS2 1.62.3.3 build. No live build is approved. Do not install the full upstream plugin as a shortcut: it has separate camera/input features outside this research scope.

## Limits

The upstream exported traffic feed selects the closest40 mixed traffic vehicles. It is not proof of a global Convoy roster. Classification, own-vehicle identity, distant-player coverage, join/leave behavior and actual frame overhead require later live validation. Synthetic single-player/disconnected fixtures clear their output; this does not prove detection of actual game session changes.

A fixture is bounded to64 nonoverlapping blocks,2MiB per block,4MiB decoded total,128 candidate entries and a256KiB code scan. The CLI limits JSON files to10MiB. These are prototype safety limits, not claims about the game's player limits. No raw game-memory capture/dump is provided or requested.

## Primary references

- SCS telemetry scope: https://modding.scssoft.com/wiki/Documentation/Engine/SDK/Telemetry
- Convoy vehicle source: https://github.com/dariowouters/ets2la_plugin/blob/c925bd965a96c58b73eaea9ca033fe0f0dc623a7/src/processing/traffic.cpp
- Candidate object layouts: https://github.com/dariowouters/ets2la_plugin/tree/c925bd965a96c58b73eaea9ca033fe0f0dc623a7/src/prism/traffic
- Sector conversion: https://github.com/dariowouters/ets2la_plugin/blob/c925bd965a96c58b73eaea9ca033fe0f0dc623a7/src/prism/common.cpp

See NOTICE.md for reference-code license and docs/convoy-native-feasibility.tr.md for results and the deferred live test.

## Authorized Windows observation update — 16 November2026

The user ended normal play and explicitly authorized a controlled two-player Convoy session. LiveResearchReader.cs was compiled outside the repo and used read-only process rights0x410 plus read-only access to the existing SDK mapping. Its exact file SHA256 gate is055ec5343d53bd427de54b06eca74937f0d348dbb0c5d71862edddcf45e2db66. Unknown files or a mismatched loaded signature are refused. The source intentionally keeps layoutVerified/onlinePlayersVerified false: nearby evidence is partial, not a production/global roster certification.

The traffic locator survived in this1.62.3.3 image, but the player-array data field moved from the legacy0x210 to0x218. Candidate objects were checked by reading the known constant-return traffic-type getter bytes (kind2); the getter was not executed. A candidate matched the SDK position of our truck, and the user-confirmed peer-only movement was observed. There were also19 missing peer rows, so reliability is not yet sufficient to release map markers.

Build explicitly, outside the repo if desired:

    powershell -NoProfile -File tools/convoy/Build-Observer.ps1 -OutDirectory <scratch> -SelfTest

SelfTest allocates and writes fixture bytes only in the helper itself; it never opens a game. Default invocation refuses to observe. Live observation requires an explicit PID, the matching image and prior user approval of a controlled session:

    ConvoyReadOnly.exe --observe --pid <game-pid> --seconds 60

Duration is bounded to120s, polling2Hz. The observer requests PROCESS_QUERY_INFORMATION|PROCESS_VM_READ; it has no remote-write or injection API. Read ranges, object counts, code-image bounds, placement values and snapshot headers are checked. Process exit stops observation. IDs are salted per helper run and no player names or raw memory dumps are emitted. Coordinates are retained only in local work logs, not published. Read-stage timing is not game-frame overhead.

For a future gap investigation, --trace-lists --pid <game-pid> --seconds 60 scans only a bounded traffic-object prefix and reports candidate player-type lists separately. It is prepared and helper-tested, but has NOT been run on the game. It does not merge those tentative lists into production data. It can inspect at most64 headers and256 detailed vehicle rows per sample. --discover --pid <game-pid> is a one-shot header diagnostic. Neither is an automated session finder.

## Recorded-data replay

fixtures/observed-motion.redacted.json contains117 normalized samples from the authorized test. Absolute coordinates, raw addresses, PIDs, names and account identifiers are removed. Metadata is not cryptographic provenance proof. The labels local/peer are research labels assigned using the independent SDK position and the user-confirmed controlled action.

    node tools/convoy/observation.mjs tools/convoy/fixtures/observed-motion.redacted.json
    node --test tools/convoy/probe.test.mjs tools/convoy/observation.test.mjs

The pure replay filter excludes a uniquely SDK-matched own vehicle, refuses ambiguous ownership and clears candidates immediately when data is missing. It never authenticates a game session or claims a complete online roster. It is research code, not connected to the production map. See docs/convoy-live-results.tr.md for metrics and the next required controlled tests.

SDK ownership freshness requires an observed telemetry sequence advance and expires after 1500 ms without publication progress. A new PID or sequence reset requires fresh progress again. Missing mappings reset the gate. This follows the existing plugin frame-end publisher; deterministic helper-only tests cover frozen publication, recovery and identity/reset changes. The new guard and process-exit check have not yet been verified in a live session. Top-level error messages use a bounded reason code to avoid exposing local paths.
