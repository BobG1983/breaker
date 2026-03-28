---
name: Current lint state
description: Game crate clippy result as of 2026-03-28 — 1 error (too_many_lines in bolt_breaker_collision), 91-92 warnings from effect stubs
type: project
---

Last run: 2026-03-28 (develop branch, after /simplify pass on feature/effect-system-rewrite merge)

## game crate (breaker-game): 1 ERROR, 91 warnings

### Errors (must fix)
- `breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs:99` — `clippy::too_many_lines` — function is 105 lines, limit is 100

### Warning categories (all expected — stubs/nursery/dead_code)
- `dead_code` — message struct fields on impact/collision messages not yet consumed (bolt/breaker/cells domains); chip components (ChainHit, BoltSizeBoost, BumpForceBoost); effect stub functions (evaluate.rs)
- `unused_imports` — effect imports in dispatch stubs (chips, effect domain); breaker/systems/mod.rs
- `missing_const_for_fn` (nursery) — ~49 placeholder `fn` stubs in effect triggers and effects that have empty bodies
- `needless_pass_by_ref_mut` (nursery) — `world: &mut World` params on stub fire/reverse fns not yet implemented
- `suboptimal_flops` (nursery) — `+=` with `*` in shockwave.rs
- `redundant_clone` (nursery) — one `.clone()` in timer.rs test (line 143)
- `unused_variables` — `world` param in random_effect.rs fire stub
- `option_if_let_else` (nursery) — likely still in bolt_breaker_collision
- `struct_variant_names` (nursery) — 6 occurrences of unnecessary structure name repetition
- `InitBreakerQuery`, `apply_breaker_config_overrides`, `init_breaker`, `dispatch_breaker_effects` — dead code in breaker/init_breaker (pending wiring)

## rantzsoft_spatial2d: PASS (0 errors, 0 warnings)
## rantzsoft_physics2d: PASS (0 errors, 0 warnings)
## scenario runner (breaker-scenario-runner): BLOCKED — game crate error prevents compilation

**Why:** The /simplify pass resolved type_complexity in bolt_wall_collision.rs. The remaining too_many_lines error needs writer-code to split bolt_breaker_collision into helper functions. The 91 warnings are from intentional placeholder stubs and will be resolved as Wave 8+ wiring completes.

**How to apply:** 1 error needs a writer-code fix (split the function). Do not treat the warnings as actionable until Wave 8+ wiring is complete.
