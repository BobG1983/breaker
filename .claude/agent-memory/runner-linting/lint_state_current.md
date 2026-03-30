---
name: Current lint state
description: Full workspace lint result as of 2026-03-30 on develop — fmt formatted 10 files; clippy PASS with 1 doc-comment warning in scenario runner
type: project
---

Last run: 2026-03-30 (develop branch)
Branch HEAD: post unit-test coverage wave (58 tests added, fad7dfa)

## Format: FIXED (10 files)

Files formatted by `cargo fmt --all`:
- `breaker-game/src/effect/effects/gravity_well/effect.rs`
- `breaker-game/src/effect/effects/gravity_well/mod.rs`
- `breaker-scenario-runner/src/invariants/checkers/mod.rs`
- `breaker-scenario-runner/src/lifecycle/systems/frame_mutations.rs`
- `breaker-scenario-runner/src/lifecycle/systems/plugin.rs`
- `breaker-scenario-runner/src/types/definitions/invariants.rs`
- `breaker-scenario-runner/src/types/definitions/mutations.rs`
- `breaker-scenario-runner/src/types/definitions/scenario.rs`
- `breaker-scenario-runner/src/types/tests/frame_mutations.rs`
- `breaker-scenario-runner/src/types/tests/invariant_kinds.rs`

## Clippy (all-dclippy): PASS (1 warning, 0 errors)

### breaker-game: PASS
### rantzsoft_spatial2d: PASS
### rantzsoft_physics2d: PASS
### rantzsoft_defaults: PASS
### breaker-scenario-runner: 1 warning

`breaker-scenario-runner/src/invariants/checkers/check_aabb_matches_entity_dimensions.rs:33`
— `clippy::doc_link_code` — doc comment has inline code adjacent to link text; suggest wrapping in `<code>` tag.
This is a doc-style nursery warning, not an error.

## Previous state
- 2026-03-30 (earlier): ALL PASS — all clippy clean after module_inception + unused import fixes
