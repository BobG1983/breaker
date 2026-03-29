---
name: vetted_dependencies
description: Known dependency state and audit findings for the brickbreaker workspace
type: project
---

## Workspace direct dependencies (as of 2026-03-28)

### breaker-game
- bevy 0.18.1 (default-features = false, features = ["2d", "serialize"])
- bevy_egui 0.39 (optional, dev only)
- rantzsoft_defaults, rantzsoft_physics2d, rantzsoft_spatial2d (workspace paths)
- tracing 0.1, tracing-appender 0.2, tracing-subscriber 0.3
- serde 1 (with derive)
- ron 0.12
- iyes_progress 0.16
- rand 0.9, rand_chacha 0.9
- proptest 1 (dev-dependency)

## cargo audit findings (2026-03-28)

### Warnings (not errors)
- `paste 1.0.15` — RUSTSEC-2024-0436 — unmaintained
  - Transitive via: metal → wgpu-hal → wgpu → bevy_render → bevy
  - Not directly controllable; no CVE, no known exploit. Info-level only.
  - Resolution: wait for wgpu/bevy to update or replace metal backend.

### cargo deny check findings
- `error[unmaintained]`: paste — same as above, mapped to deny error by deny.toml policy
- `warning[duplicate]`: 40+ crates have duplicate versions — all from transitive
  Windows/wgpu/objc2 churn. None are direct dependencies. Normal for Bevy ecosystem.
- `warning[license-not-encountered]`: Unicode-DFS-2016 in the allow list but not
  encountered in this dep tree scan. Harmless — allowlist entry is forward-compatible.

### cargo machete
- No unused dependencies found.

## Known unsafe blocks in workspace
- None found in breaker-game/src/ (workspace lint: `unsafe_code = "deny"`)
- No build.rs files anywhere in the workspace

## Phase 3 note (2026-03-28)
No new dependencies added in the effect system rewrite or trigger bridge phases.
Dependency baseline unchanged from Phase 1 audit.
