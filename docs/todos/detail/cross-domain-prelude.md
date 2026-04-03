# Cross-Domain Prelude & Import Cleanup

## Summary
Create `pub(crate)` re-export modules in `lib.rs` that aggregate common cross-domain types, then a `crate::prelude` that imports all of them. Then migrate existing imports to use the new paths: replace `super::super::` chains with `crate::` paths and consolidate verbose `use` statements into glob imports where appropriate.

## Context
Currently, importing a type from another domain requires knowing its exact module path (e.g., `crate::bolt::components::BoltVelocity`). This creates fragile, verbose imports that break when files are reorganized. A set of facade modules (`crate::components`, `crate::resources`, `crate::states`, `crate::messages`) would provide stable import points, and a `crate::prelude` would offer a single glob import for the common case.

The codebase has also accumulated `super::super::` import chains (especially in test modules after file splits) that are harder to read and more fragile than `crate::` absolute paths.

## Research

Full research reports in `cross-domain-prelude/`:
- [research-cross-domain-pub-crate-types.md](cross-domain-prelude/research-cross-domain-pub-crate-types.md)
- [research-dev-gated-types.md](cross-domain-prelude/research-dev-gated-types.md)
- [research-test-gated-types.md](cross-domain-prelude/research-test-gated-types.md)

### Key findings

**Production cross-domain types (4 tiers by usage breadth):**
- Tier 1 — Universal (5+ domains): `Bolt`, `Breaker`, `Cell`, `Wall` entity markers; `GameState`, `PlayingState` states; `BoundEffects`, `StagedEffects` effect containers; `CleanupOnNodeExit`, `PlayfieldConfig`
- Tier 2 — Effect coupling (3-4 domains): `ActivePiercings`, `ActiveDamageBoosts`, `ActiveSizeBoosts`, `ActiveSpeedBoosts`, `ActiveVulnerability`, `AnchorActive`, `AnchorPlanted`, `FlashStepActive`
- Tier 3 — Messages (2-3 domains): `BumpPerformed`, `CellDestroyedAt`, `DamageCell`, `NodeCleared`, `ChipSelected`, and many more
- Tier 4 — Narrow (2 domains): various smaller cross-domain imports

**Dev-gated types (`#[cfg(feature = "dev")]`):**
- All cross-domain dev consumers are themselves dev-gated — gating is consistent
- Re-exports needing `#[cfg(feature = "dev")]`: `DebugOverlays`, `LastBumpResult`, `RecordingConfig` (resources); `BreakerTelemetryData` (query); `propagate_chip_catalog`, `RenderAssets`, `spawn_cells_from_grid` (systems)
- `CellWidth::value`, `CellHeight::value` fields gated with `#[cfg(any(test, feature = "dev"))]`

**Test-gated types (`#[cfg(test)]`):**
- `ChipDefinition::test*` factory methods — used by `state/run/chip_select/` tests
- `EffectKind::test_shockwave` — used by `chips` tests
- `CellTypeDefinition` re-export and `CellTypeRegistry` mutation methods — used by `debug` and `state/run/node` tests
- `CellWidth::half_width()`, `CellHeight::half_height()` — used by `bolt` collision tests
- No shared test infrastructure exists — all 51 `helpers.rs` files are `pub(super)` local to their test subtree

## Scope

### Phase 1: Create re-export modules
- In: Create `src/prelude/` directory with:
  - `src/prelude/mod.rs` — declares submodules, re-exports curated subset for `use crate::prelude::*`
  - `src/prelude/components.rs` — re-exports ALL cross-domain components (full set)
  - `src/prelude/resources.rs` — re-exports ALL cross-domain resources (full set)
  - `src/prelude/states.rs` — re-exports ALL cross-domain states (full set)
  - `src/prelude/messages.rs` — re-exports ALL cross-domain messages (full set)
- In: `crate::prelude` glob re-exports only the most universal types (tier 1-2: entity markers, states, effect containers, lifecycle markers)
- In: Less common types available via `crate::prelude::components`, `crate::prelude::messages`, etc.
- In: All re-exports are `pub(crate)` for now
- In: `#[cfg(feature = "dev")]` gating on dev-only re-exports
- In: `#[cfg(test)]` gating on test-only re-exports

### Phase 2: Migrate imports
- In: Replace `super::super::` (2+ levels) with `crate::` absolute paths across all workspace crates
- In: Consolidate multiple `use` items from the same module into grouped imports or globs
- In: Migrate cross-domain imports to use the new facade modules / prelude where appropriate
- Out: Single `super::` (one level up is fine and idiomatic)
- Out: Changing any domain module structure or visibility
- Out: Making anything `pub` — internal only
- Out: Changing any logic, signatures, or visibility — imports only

### Notes
- Do phase 1 first, then phase 2 — phase 2 targets the paths phase 1 creates
- Be careful with `#[cfg(test)]` modules — `super::` in test modules referring to the parent production module is idiomatic and should stay as single `super::` (but `super::super::` should still become `crate::`)

## Decisions
- **Curated prelude**: `crate::prelude::*` gives you tier 1-2 universal types only. Full set available via `crate::prelude::components`, `crate::prelude::messages`, etc.
- **Directory structure**: `src/prelude/` directory with `mod.rs` + `components.rs`, `resources.rs`, `states.rs`, `messages.rs`
- **No name collisions in prelude-relevant categories**: all components, resources, messages, and states are uniquely named across domains. Only collisions are 6 builder typestate markers (bolt vs breaker) which are not re-export candidates. See [research-name-collisions.md](cross-domain-prelude/research-name-collisions.md).

## Open questions
- Re-validate research after state lifecycle refactor (item 1) lands — the refactor moves `crate::run` → `crate::state::run` and other modules, so import paths and cross-domain relationships in the research reports will be stale

## Dependencies
- Depends on: State lifecycle refactor (item 1) — folder structure should be settled first

## Status
`NEEDS DETAIL` — blocked on item 1 (state lifecycle refactor) for research re-validation
