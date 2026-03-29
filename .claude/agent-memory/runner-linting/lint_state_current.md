---
name: Current lint state
description: Full workspace clippy result as of 2026-03-28 on feature/runtime-effects — Wave 2 run — game crate (lib test): 5 errors / 128 warnings; dsclippy: 1 warning (scenario runner test file)
type: project
---

Last run: 2026-03-28 (feature/runtime-effects branch) — after Wave 2 changes

## Format: PASS

## game crate (breaker-game, dclippy): 5 errors (lib test), 128 warnings (lib + lib test)

### Errors (lib test)

- `effect/effects/chain_lightning/tests/fire_tests.rs:657` — `clippy::uninlined_format_args` — variables can be used directly in format string
- `effect/effects/shield.rs:165` — `clippy::cast_possible_truncation` — `f64 as u32` may truncate
- `effect/effects/shield.rs:165` — `clippy::cast_sign_loss` — `f64 as u32` may lose sign
- `effect/effects/tether_beam/tests/mod.rs:97` — `clippy::doc_markdown` — `effective_damage_multiplier` missing backticks in doc comment
- `run/node/messages.rs:65` — `clippy::float_cmp` — strict `assert_eq!` on f32

### Warning categories (lib — recurring stubs/effects)
- `unused_imports` (3) — breaker/systems/mod.rs:25; chips/dispatch_chip_effects/system.rs:11; effect/triggers/until.rs:317
- `dead_code` (4+) — breaker/queries.rs:117; breaker/systems/init_breaker/system.rs (3 fns + 1 const fn); cells/definition.rs:94; cells/messages.rs:25; chain_lightning.rs:28; pulse.rs:46; shockwave.rs:17; tether_beam.rs:21
- `missing_const_for_fn` (nursery, ~20+) — effect stubs across effects/ and triggers/
- `use_self` (nursery, 6) — effect/core/types.rs
- `suboptimal_flops` (nursery, 6) — attraction.rs, gravity_well.rs, pulse.rs, shockwave.rs
- `needless_pass_by_ref_mut` (nursery, 4) — chain_lightning.rs:100; explode.rs:45; piercing_beam.rs:84; shockwave.rs:71
- `derive_partial_eq_without_eq` (nursery, 2) — bolt/components.rs:94; piercing.rs:32
- `option_if_let_else` (nursery, 2) — bolt_breaker_collision/system.rs:103; attraction.rs:132
- `unwrap_used` (pedantic/warn, 4) — bolt_lost/system.rs:118; bolt_wall_collision.rs:110; chain_lightning.rs:78; entropy_engine.rs:38
- `redundant_clone` (nursery, 1) — effect/triggers/timer.rs:143
- `or_fun_call` (nursery, 1) — tether_beam.rs:94
- `unreachable_pub` (~50+) — effect/* items that should be pub(crate)

## rantzsoft_spatial2d: PASS (0 warnings, 0 errors)
## rantzsoft_physics2d: PASS (0 warnings, 0 errors)
## rantzsoft_defaults: PASS (0 warnings, 0 errors)
## breaker-scenario-runner (dsclippy): 1 warning — lifecycle/tests/mod.rs:7 unused imports (pre-existing)
