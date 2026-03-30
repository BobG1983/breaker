---
name: Current lint state
description: Full workspace lint result as of 2026-03-30 on develop — ALL PASS after module_inception + unused import fixes
type: project
---

Last run: 2026-03-30 (develop branch)
Branch HEAD: post file-split refactor wave + lint fix wave

## Format: PASS (no changes)

## Clippy (all-dclippy): PASS

### breaker-game: PASS
### rantzsoft_spatial2d: PASS
### rantzsoft_physics2d: PASS
### rantzsoft_defaults: PASS
### breaker-scenario-runner: PASS

## Previous state (failing)
- 2026-03-30: FAIL — 1 error (`clippy::module_inception` in `effect/commands/mod.rs`) + 2 warnings (unused re-exports in `commands/mod.rs` and `gravity_well/mod.rs`)
- Fixes applied: renamed `commands/commands.rs` → `commands/ext.rs`, removed unused `PushBoundEffects`/`TransferCommand` re-exports, removed unused `GravityWellConfig`/`GravityWellMarker` re-exports

## Clean baseline (pre-split)
- 2026-03-30: ALL PASS after error-fix wave (resolve `similar_names`, `manual_string_new`, `cast_lossless`)
