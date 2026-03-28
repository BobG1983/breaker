---
name: Current lint state
description: Full workspace clippy result as of 2026-03-28 Phase 2 merge gate — game crate: 0 errors, 66 warnings; all other crates clean
type: project
---

Last run: 2026-03-28 (develop branch, Phase 2 merge gate)

## game crate (breaker-game): 0 errors, 66 warnings

### Warning categories (all intentional stubs / nursery / dead_code)

- `unused_imports` (2) — breaker/systems/mod.rs:25 (init_breaker fns); chips/dispatch_chip_effects/system.rs:11 (effect imports)
- `unused_variables` (1) — effect/effects/random_effect.rs:9 (`world` param in fire stub)
- `dead_code` (7) — breaker/queries.rs:117 (InitBreakerQuery alias); breaker/systems/init_breaker/system.rs (apply_stat_overrides, apply_breaker_config_overrides, init_breaker, dispatch_breaker_effects); cells/definition.rs:94 (CellTypeDefinition.effects); cells/messages.rs:25 (CellDestroyedAt.position)
- `dead_code` (3) — chips/components.rs (ChainHit, BoltSizeBoost, BumpForceBoost)
- `option_if_let_else` (nursery, 1) — bolt/systems/bolt_breaker_collision/system.rs:101
- `unwrap_used` (pedantic, 1) — bolt/systems/bolt_wall_collision.rs:110
- `missing_const_for_fn` (nursery, ~30) — effect stubs across effects/ and triggers/no_bump.rs
- `use_self` (nursery, 6) — effect/core/types.rs (EffectNode self-references in impl)
- `suboptimal_flops` (nursery, 3) — gravity_well.rs:100-101, shockwave.rs:53
- `needless_pass_by_ref_mut` (nursery, 9) — effect stub fire/reverse fns in chain_lightning, explode, piercing_beam, pulse, random_effect, shockwave

## rantzsoft_spatial2d: PASS (0 warnings, 0 errors)
## rantzsoft_physics2d: PASS (0 warnings, 0 errors)
## rantzsoft_defaults: PASS (0 warnings, 0 errors)
## breaker-scenario-runner: PASS (0 warnings, 0 errors)

**Why:** All warnings are intentional effect system stubs (Phase 2) or nursery lints on partially-implemented code. The 5 `inconsistent_struct_constructor` errors from the previous session have been resolved (trigger test code was fixed).

**How to apply:** Treat all game crate warnings as expected until effect stubs are implemented. Only action needed if a new `error:` line appears.
