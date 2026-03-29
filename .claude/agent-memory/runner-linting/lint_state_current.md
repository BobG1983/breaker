---
name: Current lint state
description: Full workspace clippy result as of 2026-03-29 on feature/runtime-effects (post-fix): fmt PASS, dclippy 0 errors ~138 warnings, dsclippy 0 errors ~1 warning
type: project
---

Last run: 2026-03-29 (feature/runtime-effects branch, post-runtime-effects changes confirmed)

## Format: PASS (no files needed reformatting)

## game crate (breaker-game, dclippy): PASS (0 errors, ~138 warnings)

All previous chain_lightning errors from earlier in the session have been resolved.

### Warning categories (recurring, ~138 total)
- `dead_code` — breaker/queries.rs, init_breaker system fns, cells/definition.rs, cells/messages.rs, effect fields
- `unused_imports` — breaker/systems/mod.rs, chips/dispatch_chip_effects, effect/triggers/until.rs
- `unreachable_pub` — effect/* items pub that should be pub(crate)
- `missing_const_for_fn` (nursery)
- `suboptimal_flops` (nursery)
- `use_self` (nursery)
- `needless_pass_by_ref_mut` (nursery)
- `derive_partial_eq_without_eq` (nursery)
- `option_if_let_else` (nursery)
- `unwrap_used` (pedantic/warn)
- `or_fun_call` (nursery)
- `doc_markdown` (pedantic)
- `redundant_clone` (nursery)

## breaker-scenario-runner (dsclippy): PASS (0 errors, ~1 warning)
- 1 warning: unused imports in scenario runner (unused GameState, PlayingState, RunSeed, ChipSelected)

## rantzsoft_spatial2d: not checked this run
## rantzsoft_physics2d: not checked this run
## rantzsoft_defaults: not checked this run
