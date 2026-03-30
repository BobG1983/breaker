---
name: Core lint patterns — effect system (historical + current)
description: Historical Wave 8 stub warning patterns (now resolved) and current warning patterns as of feature/source-chip-shield-absorption
type: project
---

## Current State (feature/source-chip-shield-absorption, 2026-03-29)

All Wave 8 stubs are fully implemented. The warning patterns below are RESOLVED. Current state is 0 errors, 4 warnings — all `dead_code` on unused tuple fields in source marker structs. See `lint_state_current.md` for exact locations.

**Current warnings (all dead_code on field `0`):**
- `cells/messages.rs:25` — `CellDestroyedAt.position`
- `effect/effects/pulse/effect.rs:46` — `PulseSource.0`
- `effect/effects/shockwave/effect.rs:17` — `ShockwaveSource.0`
- `effect/effects/tether_beam/effect.rs:21` — `TetherBoltMarker.0`

These are source-chip attribution marker types whose fields are currently populated but not read downstream. Will resolve when attribution display/scoring is wired.

## Historical (RESOLVED — no longer present)

These patterns existed during the Phase 1A effect system rewrite while "Wave 8" trigger bridges and dispatch systems were stubs. All are fixed as of feature/source-chip-shield-absorption:

- `clippy::missing_const_for_fn` — stub register/reverse/fire fns; RESOLVED (all real)
- `clippy::needless_pass_by_ref_mut` — placeholder fire/reverse fns; RESOLVED (all real)
- `dead_code` on `init_breaker`, `dispatch_breaker_effects`, `evaluate_*` — RESOLVED (all wired)
- `clippy::use_self` in `effect/core/types.rs` — RESOLVED
- `clippy::suboptimal_flops` in gravity_well.rs, shockwave.rs — RESOLVED
- `clippy::redundant_clone` in `effect/triggers/timer.rs` — RESOLVED
