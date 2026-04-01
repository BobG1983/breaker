---
name: Current lint state
description: Full workspace lint result as of 2026-04-01 on feature/chip-evolution-ecosystem — fmt PASS, clippy 0 errors, 11 warnings in breaker-game (all unused_import + unreachable_pub)
type: project
---

Last run: 2026-04-01 (feature/chip-evolution-ecosystem branch, Standard Verification Tier commit gate)

## Format: PASS

No files reformatted.

## Clippy (all-dclippy): PASS (0 errors) — 11 warnings in breaker-game

### rantzsoft_spatial2d: PASS
### rantzsoft_physics2d: PASS
### rantzsoft_defaults: PASS
### breaker-scenario-runner: PASS (0 own errors/warnings)
### breaker-game: 0 errors, 11 warnings

## Warning inventory (breaker-game, 11 total)

- unused_import — bolt/builder/mod.rs:5 — `pub use core::*`
- unused_import — bolt/builder/tests/build_tests.rs:8 — `super::super::core::*`
- unused_import — bolt/builder/tests/optional_methods_tests.rs:5 — `super::super::core::*`
- unused_import — bolt/builder/tests/optional_methods_tests.rs:12 — `shared::CleanupOnNodeExit`
- unused_import — bolt/builder/tests/ordering_and_layers_tests.rs:8 — `super::super::core::*`
- unused_import — bolt/builder/tests/spawn_and_effects_tests.rs:8 — `super::super::core::*`
- unused_import — bolt/builder/tests/typestate_tests.rs:8 — `PrimaryBolt`
- unused_import — bolt/systems/bolt_wall_collision/tests/last_impact_tests.rs:2 — `Velocity2D`
- unused_import — bolt/systems/spawn_bolt/tests/helpers.rs:3 — `super::super::*`
- unused_import — bolt/systems/bolt_wall_collision/tests/impact_tests.rs:1 — `bevy::prelude`
- unreachable_pub — bolt/builder/mod.rs:5 — `pub use core::*` should be `pub(crate)`

Note: redundant_clone warnings from breaker/definition.rs (lines 168 and 182) are no longer present as of this run.

## Previous state
- 2026-04-01 (feature/chip-evolution-ecosystem, Wave 1 Basic Verification Tier re-run after fixes): fmt PASS, clippy 0 errors, 15 warnings (included 2x redundant_clone in breaker/definition.rs)
- 2026-04-01 (feature/chip-evolution-ecosystem, Wave 1 Basic Verification Tier): fmt PASS, clippy 22 errors + 16 warnings in breaker-game
- 2026-04-01 (feature/chip-evolution-ecosystem, second run): fmt PASS, clippy 1 warning (missing_const_for_fn), 0 errors
- 2026-04-01 (feature/chip-evolution-ecosystem, first run): fmt PASS, all clippy PASS (0 warnings, 0 errors)
- 2026-03-31 (feature/chip-evolution-ecosystem, first run): fmt PASS, all clippy PASS (0 warnings, 0 errors)
- 2026-03-30 (feature/scenario-coverage, eleventh run): fmt PASS, all clippy PASS, all tests PASS — 2994 total tests, 0 failed
