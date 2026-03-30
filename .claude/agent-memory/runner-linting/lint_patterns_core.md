---
name: Core lint patterns — effect system (historical)
description: Historical Wave 8 stub warning patterns (now resolved); source marker dead_code warnings resolved; see lint_state_current.md for current state
type: project
---

## Current State (as of 2026-03-30)

All Wave 8 stubs are fully implemented and all dead_code warnings are resolved.
Current state: 0 errors, 1 doc-style nursery warning in breaker-scenario-runner.
See `lint_state_current.md` for exact location.

**Previously tracked dead_code warnings — ALL RESOLVED:**

The four source-chip marker types (`CellDestroyedAt`, `PulseSource`, `ShockwaveSource`,
`TetherBoltMarker`) were previously tuple structs with an unused `.0` field, generating
`dead_code` warnings. They have since been converted to unit structs (no tuple field),
eliminating all four warnings. Verified by inspection of effect.rs files 2026-03-30.

## Historical (RESOLVED — no longer present)

These patterns existed during the Phase 1A effect system rewrite while "Wave 8" trigger bridges and dispatch systems were stubs. All are fixed as of feature/source-chip-shield-absorption:

- `clippy::missing_const_for_fn` — stub register/reverse/fire fns; RESOLVED (all real)
- `clippy::needless_pass_by_ref_mut` — placeholder fire/reverse fns; RESOLVED (all real)
- `dead_code` on `init_breaker`, `dispatch_breaker_effects`, `evaluate_*` — RESOLVED (all wired)
- `clippy::use_self` in `effect/core/types.rs` — RESOLVED
- `clippy::suboptimal_flops` in gravity_well.rs, shockwave.rs — RESOLVED
- `clippy::redundant_clone` in `effect/triggers/timer.rs` — RESOLVED
