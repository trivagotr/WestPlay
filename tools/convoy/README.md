# Convoy research diagnostics

This directory contains historical file-only research tools and the fingerprint-gated Windows reader now used by automatic native GPS tracking. See the final section for the current integration and its deferred two-player identity test. probe.mjs and observation.mjs remain file-only. No tool modifies game memory, calls game code, injects anything or starts the game.

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

Some upstream layouts are annotated 1.60. They are NOT validated for the previously observed ETS2 1.62.3.3 build. Those legacy synthetic layouts do not authorize a live reader. The current Windows reader uses its own exact-build gates described below. Do not install the full upstream plugin as a shortcut: it has separate camera/input features outside this research scope.

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

The traffic locator survived in this1.62.3.3 image, but the player-array data field moved from the legacy0x210 to0x218. Candidate objects were checked by reading the known constant-return traffic-type getter bytes (kind2); the getter was not executed. A candidate matched the SDK position of our truck, and the user-confirmed peer-only movement was observed. There were also19 missing peer rows. Current markers are therefore experimental and clear on missing samples; complete coverage/reliability is not certified.

Build explicitly, outside the repo if desired:

    powershell -NoProfile -File tools/convoy/Build-Observer.ps1 -OutDirectory <scratch> -SelfTest

SelfTest allocates and writes fixture bytes only in the helper itself; it never opens a game. Default invocation refuses to observe. The separate bounded research observation command requires an explicit PID and the matching image:

    ConvoyReadOnly.exe --observe --pid <game-pid> --seconds 60

Duration is bounded to120s, polling2Hz. The observer requests PROCESS_QUERY_INFORMATION|PROCESS_VM_READ; it has no remote-write or injection API. Read ranges, object counts, code-image bounds, placement values and snapshot headers are checked. Process exit stops observation. IDs are salted per helper run. Optional display names and positions are delivered to the local map UI; diagnostic exports omit names and absolute coordinates. No raw memory dump is produced. Read-stage timing is not game-frame overhead.

For a future gap investigation, --trace-lists --pid <game-pid> --seconds 60 scans only a bounded traffic-object prefix and reports candidate player-type lists separately. It is prepared and helper-tested, but has NOT been run on the game. It does not merge those tentative lists into production data. It can inspect at most64 headers and256 detailed vehicle rows per sample. --discover --pid <game-pid> is a one-shot header diagnostic. Neither is an automated session finder.

## Recorded-data replay

fixtures/observed-motion.redacted.json contains117 normalized samples from the authorized test. Absolute coordinates, raw addresses, PIDs, names and account identifiers are removed. Metadata is not cryptographic provenance proof. The labels local/peer are research labels assigned using the independent SDK position and the user-confirmed controlled action.

    node tools/convoy/observation.mjs tools/convoy/fixtures/observed-motion.redacted.json
    node --test tools/convoy/probe.test.mjs tools/convoy/observation.test.mjs

The pure replay filter excludes a uniquely SDK-matched own vehicle, refuses ambiguous ownership and clears candidates immediately when data is missing. It never authenticates a game session or claims a complete online roster. It is research code, not connected to the production map. See docs/convoy-live-results.tr.md for metrics and the next required controlled tests.

SDK ownership freshness requires an observed telemetry sequence advance and expires after 1500 ms without publication progress. A new PID or sequence reset requires fresh progress again. Missing mappings reset the gate. This follows the existing plugin frame-end publisher; deterministic helper-only tests cover frozen publication, recovery and identity/reset changes. The new guard and process-exit check have not yet been verified in a live session. Top-level error messages use a bounded reason code to avoid exposing local paths.

## Automatic native GPS reader — 16 November 2026

The main app may now run `WestPlay.ConvoyReader.exe --follow --pid <game-pid> --parent-pid <westplay-pid>` while its SDK connection identifies that ETS2 process. This continuous mode samples at4Hz, has no120-second cutoff, and ends on target exit, parent-app exit, or when the owning app terminates the helper. Follow mode requires the parent PID; it captures and holds that process handle and creation time before observing the game, then checks parent lifetime every sample. A crash/forced close therefore ends observation within one polling interval; PID reuse cannot transfer ownership. Research `--observe` remains explicitly bounded to120seconds at2Hz. The helper itself never starts a game.

For the exact fingerprint above, a second set of loaded instruction signatures gates the native session-name route. Session records join to traffic vehicles only by exact actor-pointer equality (`session+0x2D8` array; `info+0x430` remote; `remote+0x290` actor). Default display text comes from `info+0x108`, payload+8. There is no nearest-position, ordering, or index name association. No game function is executed.

Names require unique roster records and actor bindings, native map flags, canonical readable pointers, at most128records, at most256UTF-8 bytes with NUL termination, strict decoding, and coherent before/after snapshots. Control and Unicode format characters are removed. A broken identity route omits all names from that sample while retaining independently validated anonymous positions. Successful vehicle rows include `name` and `nameBinding: native-session-actor`; `identityVerified` stays false. Diagnostics carry only counts and bounded reason codes. Runtime display names are used only by the UI, not diagnostic export. No names are cached across samples.

`--self-test` covers exact actor matching, rejection of nearest/index matching, malformed UTF-8, length/count budgets, duplicate actor mappings, changing pointers/rosters, inactive actors, empty rosters, and text controls using synthetic memory. Parent lifetime checks cover the live helper itself plus exited, changed-creation, and inaccessible fixture states. The own-process read-rights/placement test remains. It never opens ETS2. Two-player runtime identity and reconnect/vehicle-change acceptance are still deferred; static binding is not authentication or a verified complete session roster.
