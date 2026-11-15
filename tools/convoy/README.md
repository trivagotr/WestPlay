# Convoy offline research probe

This is an isolated research prototype, not a working online-player map feature. It has no process-access APIs, game hooks, network connection, SDK registration or plugin installation. It never starts or attaches to the game. Main WestPlay source, ABI and runtime packages are unchanged.

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
