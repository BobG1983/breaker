---
name: Feature-gated test groups pattern
description: Test sub-modules in rantzsoft_defaults use #[cfg(feature = ...)] gates — split files must preserve these gates at the item level, not at the mod.rs level
type: project
---

## Pattern: Feature-gated test sub-modules

In `rantzsoft_defaults/src/systems.rs`, test groups are nested under `#[cfg(feature = "progress")]` and `#[cfg(feature = "hot-reload")]` gates. When splitting these into separate files, the gate moves from the `mod` declaration to the individual test items inside each file.

The `tests/mod.rs` declares all sub-modules unconditionally:
```rust
mod seed_config;        // contains #[cfg(feature = "progress")] items
mod propagate_defaults; // contains #[cfg(feature = "hot-reload")] items
```

The feature gate shifts from wrapping the entire sub-mod block to wrapping the individual `fn` items inside each file. This is the correct approach — do NOT gate the `mod` declaration in `mod.rs`.

## Pattern: Duplicated TestRegistry/TestRegistryAsset types

`rantzsoft_defaults/src/systems.rs` duplicates `TestRegistry`, `TestRegistryAsset`, and their `SeedableRegistry` impl across three test groups (`init_registry_handles_tests`, `seed_registry_tests`, `propagate_registry_tests`). When splitting, extract these into a shared `test_helpers.rs` inside the `tests/` directory and import via `use super::test_helpers::*;` in each group file.
