# Mobile fill view and friend package implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:executing-plans for inline work and dispatch independent packaging preparation.

**Goal:** Make the web mirror visibly larger and deliver a self-contained shareable bundle.
**Architecture:** CSS viewport sizing plus explicit fill/contain geometry; existing video and input transport unchanged. Reproducible file-validated distribution script gathers release binaries/resources and APK.
**Tech stack:** browser JS/CSS, Rust/Tauri, PowerShell, Android APK.
**Spec:** docs/superpowers/specs/2026-11-15-fill-and-friend-package.md

- [x] Add failing geometry regressions for fill bounds, drag clamp, exact control targets and contain compatibility.
- [x] Implement fill toggle, safe preference persistence, active-pointer cancellation and viewport sizing.
- [x] Run geometry tests and headless real WebRTC viewer smoke in portrait/landscape/fullscreen, checking native pointer coordinates in both modes.
- [x] Prepare independent friend distribution script and prerequisite/license assets; validate missing input handling.
- [x] Build Windows app, game plugin, probe and map helper; reuse verified APK.
- [x] Review implementation, prepare archive content/hash validation, export source changes and deliver concise first-run instructions.
