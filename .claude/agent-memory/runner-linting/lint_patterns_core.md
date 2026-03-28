---
name: Core lint patterns — effect system rewrite (Phase 1A)
description: Recurring warning patterns introduced by the effect system rewrite; all warnings, zero errors as of develop branch 2026-03-28
type: project
---

All warnings cluster in `breaker-game/src/effect/` and are a direct result of the Phase 1A effect system rewrite where placeholder stubs were written before full wiring (Wave 8 not yet complete).

**Pattern 1 — `clippy::missing_const_for_fn`** (nursery)
Bridge functions and stub `register`/`reverse`/`fire` functions throughout `src/effect/triggers/` and `src/effect/effects/` are empty or trivially no-op bodies. Clippy suggests `const fn`. These are intentional stubs — will be real implementations in Wave 8.

**Pattern 2 — `clippy::needless_pass_by_ref_mut`** (nursery)
Placeholder `fire` and `reverse` fns accept `world: &mut World` but do not use it mutably yet. Same cause as above — stubs awaiting real implementation.

**Pattern 3 — `dead_code` / `unused_imports`**
`init_breaker`, `apply_breaker_config_overrides`, `dispatch_breaker_effects`, `evaluate_*`, `walk_*` — functions/types declared but not yet wired into the plugin registration. Also `BoundEffects`, `EffectCommandsExt`, `RootEffect`, `Target` imports in `dispatch_chip_effects/system.rs`.

**Pattern 4 — `clippy::use_self`** (nursery)
`EffectNode` used by name inside its own `impl` block in `src/effect/core/types.rs`. Clippy wants `Self`.

**Pattern 5 — `clippy::suboptimal_flops`** (nursery)
`x += a * b` patterns in `gravity_well.rs` and `shockwave.rs`. Clippy suggests `.mul_add()`.

**Pattern 6 — `clippy::redundant_clone`** (nursery)
`inner.clone()` in `effect/triggers/timer.rs:143` where the value is dropped immediately after.

**Zero errors** — all findings are warnings. `rantzsoft_spatial2d`, `rantzsoft_physics2d`, `rantzsoft_defaults`, and `breaker-scenario-runner` (own code only) are all clean.

**Why:** These accumulate because the effect system rewrite is mid-flight (Wave 8 wiring not done). Not regressions — known state.
**How to apply:** When routing these to writer-code, group by pattern. Pattern 6 (redundant_clone) and Pattern 4 (use_self) are the cheapest fixes. Patterns 1+2 will resolve naturally when stubs get real implementations in Wave 8.
