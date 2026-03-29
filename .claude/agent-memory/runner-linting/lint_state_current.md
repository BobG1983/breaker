---
name: Current lint state
description: Full workspace clippy result as of 2026-03-28 post-doc_markdown fix — game crate: 0 errors, 63 warnings (lib); all other crates clean
type: project
---

Last run: 2026-03-28 (develop branch, post-doc_markdown fix)

## game crate (breaker-game): 0 errors, 63 warnings (lib)

doc_markdown errors fully resolved. 0 errors remain in lib or lib test.

### Warning categories (lib build, all pre-existing stubs / nursery / dead_code)

- `unused_imports` (3) — breaker/systems/mod.rs:25; chips/dispatch_chip_effects/system.rs:11; effect/triggers/until.rs:317
- `unused_variables` (1) — effect/effects/random_effect.rs:9
- `dead_code` (5) — breaker/queries.rs:117; breaker/systems/init_breaker/system.rs (3 fns); cells/definition.rs:94; cells/messages.rs:25
  - NOTE: `apply_stat_overrides` dead_code only surfaces via dsclippy (not dclippy) — 64 vs 63 warning count
- `option_if_let_else` (nursery, 1) — bolt/systems/bolt_breaker_collision/system.rs:103
- `unwrap_used` (pedantic, 1) — bolt/systems/bolt_wall_collision.rs:110
- `missing_const_for_fn` (nursery, ~29) — effect stubs across effects/ and triggers/no_bump.rs
- `use_self` (nursery, 6) — effect/core/types.rs
- `suboptimal_flops` (nursery, 3) — gravity_well.rs:100-101, shockwave.rs:53
- `needless_pass_by_ref_mut` (nursery, 9) — effect stub fire/reverse fns
- `derive_partial_eq_without_eq` (nursery, 2) — bolt/components.rs:94, effect/effects/piercing.rs:32
- `redundant_clone` (nursery, 1) — effect/triggers/timer.rs:143

## rantzsoft_spatial2d: PASS (0 warnings, 0 errors)
## rantzsoft_physics2d: PASS (0 warnings, 0 errors)
## breaker-scenario-runner: PASS (0 warnings, 0 errors — scenario runner crate itself only)
