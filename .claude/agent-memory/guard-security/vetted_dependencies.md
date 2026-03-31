---
name: vetted_dependencies
description: Known dependency state and audit findings for the brickbreaker workspace
type: project
---

## Workspace direct dependencies (as of 2026-03-30)

### breaker-game
- bevy 0.18.1 (default-features = false, features = ["2d", "serialize"])
- bevy_egui 0.39 (optional, dev only)
- rantzsoft_defaults, rantzsoft_physics2d, rantzsoft_spatial2d (workspace paths)
- tracing 0.1, tracing-appender 0.2, tracing-subscriber 0.3
- serde 1 (with derive)
- ron 0.12
- iyes_progress 0.16
- rand 0.9, rand_chacha 0.9
- (proptest removed — no dev-dependencies remaining)

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

## Phase 4+5 note (2026-03-28, feature/runtime-effects)
No new dependencies added in the runtime effects implementation (attraction, chain_bolt,
explode, pulse, second_wind, shockwave, spawn_phantom). Dependency baseline unchanged.
cargo audit: same single warning (paste RUSTSEC-2024-0436, unmaintained, transitive via metal→wgpu).
cargo machete: no unused dependencies found.

## Phase 6 note (2026-03-29, feature/source-chip-shield-absorption)
No new dependencies added. Dependency baseline unchanged.
cargo audit: same single warning (paste RUSTSEC-2024-0436, unmaintained, transitive via metal→wgpu).
cargo machete: no unused dependencies found.

## develop post-merge note (2026-03-30, refactor/split-23-files)
Refactor commit c9964b7 split 23 oversized .rs files into directory modules (code-only
structural change, no logic changes). No new dependencies added. Dependency baseline
unchanged from Phase 6.
cargo audit: same single warning (paste RUSTSEC-2024-0436, unmaintained, transitive).
cargo deny: exits code 1 due to deny.toml treating paste warning as error — same as before.
  Also warns on Unicode-DFS-2016 license not encountered (harmless allowlist forward-compat entry).
  40+ transitive duplicate crates (all wgpu/Windows ecosystem churn, no direct deps affected).
cargo machete: no unused dependencies found.
All vetted direct dependencies confirmed unchanged.

## feature/missing-unit-tests note (2026-03-30)
Branch adds unit tests only; one production change: overlay_color in
breaker-game/src/fx/transition/system.rs widened from private to pub(super) to enable
test access. No new dependencies added. Dependency baseline unchanged.
cargo audit: same single warning (paste RUSTSEC-2024-0436, unmaintained, transitive).
All test .unwrap()/.expect() calls are inside #[cfg(test)] modules — not production panic
surface. No new unsafe blocks.

## feature/scenario-coverage note (2026-03-30)
One new direct dependency: rantzsoft_physics2d (workspace path) added to
breaker-scenario-runner/Cargo.toml. This is the workspace-internal physics2d crate —
already vetted in prior audits, no CVEs, no unsafe code.
cargo audit: same single warning (paste RUSTSEC-2024-0436, unmaintained, transitive via metal→wgpu).
cargo deny: same error (paste unmaintained) + same warnings (Unicode-DFS-2016 unmatched, 40+ duplicate transitive).
cargo machete: no unused dependencies found.
No new external crates introduced.

## Cache removal refactor audit (2026-03-30, feature/scenario-coverage — commits d6d9b80 + 2bdb81b)
No new dependencies. Dependency baseline unchanged from prior note.
cargo audit: same single warning (paste RUSTSEC-2024-0436, unmaintained, transitive).
cargo machete: no unused dependencies found.
No new external crates introduced. Removals are internal types only.
