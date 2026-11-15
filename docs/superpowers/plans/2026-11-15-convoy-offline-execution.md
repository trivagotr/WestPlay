# Convoy native-data feasibility execution

Goal: investigate native ETS2 Convoy coordinates without a relay/VPS. First target is the already observed ETS2 1.62.3.3 build. Markers only, no names.

Binding constraint: user is playing. Do not access the game process or installed game files, read process memory, inject/load/copy plugins, restart anything, attach a debugger, capture the screen, or run heavy native builds. Live two-player validation is deferred until the user says play has ended.

Ruling: implement only an offline bounded decoder and synthetic evidence harness now. A synthetic pass is never proof of live access or current-build compatibility. No production map source, existing plugin, runtime package or launcher will change.

Evidence: official SDK1.16 exposes own-vehicle telemetry, not a Convoy roster. MIT-licensed ETS2LA source at c925bd965a96c58b73eaea9ca033fe0f0dc623a7 identifies traffic_player_vehicles and Convoy trucks, but its layouts explicitly mention1.60 and its exported traffic feed is closest40 mixed traffic vehicles. These are candidate layouts, not verified1.62 offsets or a global online-player roster.

Tasks:
- [x] Review SDK and primary candidate implementation.
- [x] Add a file-only synthetic decoder with strict bounds, unique-signature resolution and fail-closed live/build gates.
- [x] Test sector coordinate conversion, own-vehicle exclusion and invalid/ambiguous data without touching the game.
- [x] Record limits, source/license provenance and a post-game live-validation checklist.
- [x] Fresh read-only review of research files and final artifact delivery.

Live acceptance remains open: near/far Convoy players, stable identity/classification, own-truck exclusion, join/leave/disconnect behavior and actual game-frame overhead. No server or paid service fallback without a separate decision.

Review: one documentation overstatement about real-data rejection was corrected. Tags do not authenticate provenance; no live reader exists. Independent synthetic tests passed. No production files, installed game files, running processes, runtime packages or user game state were modified.
