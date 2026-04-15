# Effect v3 Test Migration to TestAppBuilder

## Summary
Migrate all `effect_v3/` tests to use `TestAppBuilder` instead of raw `App::new()` + `MinimalPlugins`. Add any missing builder features needed to support effect test patterns.

## Context
Many `effect_v3/` tests were written before `TestAppBuilder` existed or matured. They construct test apps manually with `App::new()`, `MinimalPlugins`, and inline message/resource registration. This creates inconsistency with newer tests (death pipeline, bolt, cells) that all use `TestAppBuilder`.

The `TestAppBuilder` at `shared/test_utils/builder.rs` provides `.with_message::<M>()`, `.with_message_capture::<M>()`, `.with_system()`, `.with_resource::<R>()`, `.with_playfield()`, and `.build()`. If effect tests need patterns the builder doesn't support (e.g., `EffectV3Plugin` registration, `register_effect_v3_test_infrastructure`, `GameRng` seeding), those should be added as builder methods.

## Scope
- In: All test files under `breaker-game/src/effect_v3/` that use `App::new()` + `MinimalPlugins` directly
- In: Adding new `TestAppBuilder` methods if needed (e.g., `.with_effect_v3()`, `.with_game_rng(seed)`)
- In: Verifying all migrated tests still pass
- Out: Changing test logic or assertions — only the app construction changes
- Out: Tests outside `effect_v3/`

## Dependencies
- Depends on: Effect system refactor (done), unified death pipeline (done)
- Blocks: Nothing — this is cleanup/consistency work

## Notes
- `register_effect_v3_test_infrastructure` was promoted to `pub(crate)` during the death pipeline work. A builder method wrapping it would be cleaner than calling it directly.
- The 33 oversized effect_v3 files (todo #1) will also be split — coordinate so the test migration and file splits don't conflict. Recommend doing file splits first, then migrating the extracted test files.
- ~64 test modules in effect_v3 may need migration. Scope a quick grep for `App::new()` in effect_v3 tests to estimate effort.

## Status
`ready`
