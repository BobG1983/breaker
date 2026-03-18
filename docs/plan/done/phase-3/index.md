# Phase 3: Dev Infrastructure — Done

**Goal**: Restructure the project for multi-crate growth, add live-reload tooling, and build an automated scenario runner to catch runtime failures before the vertical slice.

## Subphases

- [3a: Workspace Restructure](phase-3a-workspace-restructure.md) — Axum-style workspace: breaker-game/, breaker-derive/, breaker-scenario-runner/ as peer directories
- [3b: Debug Domain Restructure](phase-3b-debug-restructure.md) — Split debug into overlays/, telemetry/, hot_reload/, recording/ sub-domains
- [3c: RON Hot-Reload](phase-3c-hot-reload.md) — File watching → defaults → config → components propagation pipeline
- [3d: Scenario Runner](phase-3d-scenario-runner.md) — Automated gameplay testing with argh CLI, chaos monkey, invariant checking, CI integration
- [3e: Structured Logging](phase-3e-structured-logging.md) — Rolling file appender, lifecycle log calls, dev CLI log flags
