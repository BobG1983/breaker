---
name: Current lint state
description: Full workspace clippy result as of 2026-03-28 on feature/runtime-effects — game crate: 9 errors / 56+ warnings; all other crates clean except dsclippy fails due to game crate dependency
type: project
---

Last run: 2026-03-28 (feature/runtime-effects branch) — post runtime-effects changes

## Format: PASS

## game crate (breaker-game): 9 errors, 56+ warnings (lib)

### Errors (new — from recently modified files)

- `chain_lightning.rs:994` — `clippy::similar_names` — `cell_b_damage` too similar to `cell_a_damage`
- `entropy_engine.rs:72` — `clippy::manual_string_new` — `"".into()` should be `String::new()`
- `entropy_engine.rs:76` — `clippy::manual_string_new` — `"".into()` should be `String::new()`
- `piercing_beam.rs:39` — `clippy::map_unwrap_or` — `.map(...).unwrap_or(...)` should be `.map_or(..., ...)`
- `piercing_beam.rs:133` — `clippy::map_unwrap_or` — `.map(...).unwrap_or(...)` should be `.map_or(..., ...)`
- `random_effect.rs:33` — `clippy::manual_string_new` — `"".into()` should be `String::new()`
- `tether_beam.rs:152` — `clippy::doc_markdown` — `bolt_a` missing backticks in doc comment
- `tether_beam.rs:152` — `clippy::doc_markdown` — `bolt_b` missing backticks in doc comment
- `tether_beam.rs:176` — `clippy::single_match_else` — `match` with single pattern should be `if let`
- `tether_beam.rs:183` — `clippy::single_match_else` — `match` with single pattern should be `if let`
- `chain_lightning.rs:325` — `clippy::uninlined_format_args` (lib test) — use inlined format args
- `entropy_engine.rs:190` — `clippy::uninlined_format_args` (lib test) — use inlined format args

Note: lib reports 9 errors; lib test reports 12 (3 additional from test code in chain_lightning.rs)

### Warning categories (lib — pre-existing / stubs)
- `unused_imports` (2) — breaker/systems/mod.rs:25; chips/dispatch_chip_effects/system.rs:11; effect/triggers/until.rs:317
- `dead_code` (5) — breaker/queries.rs:117; breaker/systems/init_breaker/system.rs (3 fns); cells/definition.rs:94; cells/messages.rs:25
- `private_interfaces` (4) — chain_lightning.rs:102; piercing_beam.rs:88; tether_beam.rs:160 (2x)
- `option_if_let_else` (nursery) — bolt/systems/bolt_breaker_collision/system.rs:103
- `unwrap_used` (pedantic/warn) — bolt/systems/bolt_wall_collision.rs:110; chain_lightning.rs:66; entropy_engine.rs:37
- `missing_const_for_fn` (nursery, ~20+) — effect stubs across effects/ and triggers/
- `use_self` (nursery, 6) — effect/core/types.rs
- `suboptimal_flops` (nursery, ~5) — attraction.rs, gravity_well.rs, pulse.rs, shockwave.rs
- `needless_pass_by_ref_mut` (nursery, 4) — chain_lightning.rs:94; explode.rs:37; piercing_beam.rs:78; shockwave.rs:61
- `derive_partial_eq_without_eq` (nursery, 2) — bolt/components.rs:94; piercing.rs:32
- `redundant_clone` (nursery, 1) — effect/triggers/timer.rs:143
- `or_fun_call` (nursery, 1) — tether_beam.rs:172

## rantzsoft_spatial2d: PASS (0 warnings, 0 errors)
## rantzsoft_physics2d: PASS (0 warnings, 0 errors)
## rantzsoft_defaults: PASS (0 warnings, 0 errors)
## breaker-scenario-runner: FAIL (game crate compilation fails; no scenario-runner-specific errors)
