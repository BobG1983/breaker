# Phase 3: Dev Infrastructure

**Goal**: Restructure the project for multi-crate growth, add live-reload tooling, and build an automated scenario runner to catch runtime failures before the vertical slice.

## Subphases

- [3a: Workspace Restructure](phase-3a-workspace-restructure.md) — Axum-style workspace: game/, derive/, scenario-runner/ as peer directories
- [3b: Debug Domain Restructure](phase-3b-debug-restructure.md) — Split debug into overlays/, telemetry/, hot_reload/ sub-domains
- [3c: RON Hot-Reload](phase-3c-hot-reload.md) — File watching → defaults → config → components propagation pipeline
- [3d: Scenario Runner](phase-3d-scenario-runner.md) — Automated gameplay testing with argh CLI, chaos monkey, invariant checking

## Build Order Rationale

3a (workspace restructure) is a prerequisite for 3d (scenario runner needs its own crate). 3b (debug restructure) creates the sub-domain homes for 3c (hot-reload systems). Do the structural work first, then fill in the systems.

Dev infrastructure before the vertical slice means Phase 4 benefits from hot-reload (tune RON values live) and scenario testing (catch regressions automatically).
