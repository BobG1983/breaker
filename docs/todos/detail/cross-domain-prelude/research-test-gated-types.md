# Research: `#[cfg(test)]`-Gated Items Used Across Domain Boundaries

## Summary

This analysis covers all `#[cfg(test)]`-gated items that are `pub(crate)` or broader, identifies which ones are consumed by test code in domains other than where they are defined, and enumerates any shared test utility infrastructure.

The short answer: there are **no dedicated cross-domain test utilities** in the codebase. All test helpers are scoped to `pub(super)` and stay inside their own subtree. However, there are **five distinct categories of test-gated items** that genuinely cross domain lines. These are the items that will need `#[cfg(test)]` re-exports in any cross-domain re-export module.

---

## Category 1 — `ChipDefinition` Test Factory Methods

**Location:** `breaker-game/src/chips/definition/types.rs`

**Items (all `pub(crate)`, inside `#[cfg(test)] impl ChipDefinition`):**
- `ChipDefinition::test(name, effect, max_stacks) -> Self`
- `ChipDefinition::test_simple(name) -> Self`
- `ChipDefinition::test_on(name, target, effect, max_stacks) -> Self`
- `ChipDefinition::with_template(template_name) -> Self` (builder pattern)

**What they are:** Test factory methods that construct minimal `ChipDefinition` instances without needing asset loading. `test_simple` builds a `PerfectBump → Shockwave` chip; `test_on` gives full control over target and effect tree; `with_template` sets the optional template name for evolution tests.

**Cross-domain consumers:**

| Consumer domain | Location |
|---|---|
| `state/run/chip_select` | `systems/tick_chip_timer.rs` (in `#[cfg(test)] mod tests`) |
| `state/run/chip_select` | `systems/handle_chip_input/tests.rs` |
| `state/run/chip_select` | `systems/generate_chip_offerings/tests.rs` |
| `state/run/chip_select` | `systems/spawn_chip_select.rs` (in `#[cfg(test)] mod tests`) |
| `state/run/chip_select` | `resources.rs` (in `#[cfg(test)] mod tests`) |

Note: `chips::mod.rs` also exports `#[cfg(test)] pub(crate) use resources::Recipe`, but `Recipe` is only consumed within the `chips` domain's own tests — not cross-domain.

**Re-export path to make available:**
Currently accessible via `crate::chips::ChipDefinition::test*(...)` because `ChipDefinition` is re-exported at `chips::mod` as `pub(crate) use definition::ChipDefinition`. The test methods live on the type, so they travel with it automatically.

---

## Category 2 — `EffectKind::test_shockwave` Factory Method

**Location:** `breaker-game/src/effect/core/types/tests.rs`

**Item:**
- `EffectKind::test_shockwave(base_range: f32) -> Self` — `pub fn` inside `#[cfg(test)] impl EffectKind`

**What it is:** A convenience constructor that builds a `Shockwave` variant with sensible test defaults (zero `range_per_level`, 1 stack, 400 speed). The `#[cfg(test)] mod tests` parent module gates the entire impl block.

**Why it's special:** `EffectKind` is in `effect::core::types::definitions`, re-exported publicly via `effect::core::*` → `effect::*`. The `#[cfg(test)] impl EffectKind` block in `tests.rs` adds a method that is therefore reachable from any domain in test builds — but only if the caller can see the `tests` module, which is private. In practice the method is callable from anywhere that can name `EffectKind` (which is everyone), because the method is declared `pub fn` and the impl is on the public type. Rust's method resolution picks it up without requiring the caller to import the `tests` module.

**Cross-domain consumers:**

| Consumer domain | Location |
|---|---|
| `chips` | `chips/definition/types.rs` (inside `#[cfg(test)] impl ChipDefinition::test_simple`) |

Only one cross-domain consumer, and it's itself inside a `#[cfg(test)]` impl block.

---

## Category 3 — `CellTypeDefinition` (cells domain, test-gated re-export)

**Location:** `breaker-game/src/cells/mod.rs`

**Item:**
```rust
#[cfg(test)]
pub(crate) use definition::CellTypeDefinition;
```

**What it is:** `CellTypeDefinition` is the struct loaded from `.cells.ron` files describing cell types (id, alias, hp, color, behavior, effects). Without the `#[cfg(test)]` re-export it is only accessible as `crate::cells::definition::CellTypeDefinition`. The re-export promotes it to `crate::cells::CellTypeDefinition` in test builds, making it convenient to name in test setup code.

**Cross-domain consumers:**

| Consumer domain | Location |
|---|---|
| `debug` | `hot_reload/systems/propagate_node_layout_changes.rs` — inside `#[cfg(test)] mod tests` |
| `debug` | `hot_reload/systems/propagate_cell_type_changes.rs` — inside `#[cfg(test)] mod tests` |
| `state/run/node` | `systems/spawn_cells_from_layout/tests/helpers.rs` |
| `state/run/node` | `systems/spawn_cells_from_layout/tests/behaviors.rs` |
| `state/run/node` | `systems/spawn_cells_from_layout/tests/position2d.rs` |
| `state/run/node` | `systems/spawn_cells_from_layout/tests/shield_cells/helpers.rs` |
| `state/run/node` | `systems/spawn_cells_from_layout/tests/shield_cells/clearance_and_hierarchy.rs` |
| `state/run/node` | `definition/tests/helpers.rs` |

All consumers are inside `#[cfg(test)]` blocks or `tests/` subdirectories.

---

## Category 4 — `CellTypeRegistry` Mutation Methods (cells domain)

**Location:** `breaker-game/src/cells/resources.rs`

**Items (all `#[cfg(test)]`, all `pub(crate)`):**
- `CellTypeRegistry::contains(alias: char) -> bool`
- `CellTypeRegistry::insert(alias: char, def: CellTypeDefinition) -> Option<CellTypeDefinition>`
- `CellTypeRegistry::len() -> usize`

**What they are:** Methods that allow test code to populate a `CellTypeRegistry` without going through the asset loading pipeline. Production code only reads from the registry via the existing `pub(crate) fn get()`.

**Cross-domain consumers:**

| Consumer domain | Location |
|---|---|
| `debug` | `hot_reload/systems/propagate_node_layout_changes.rs` — inside `#[cfg(test)] mod tests`, calls `registry.insert(...)` |
| `debug` | `hot_reload/systems/propagate_cell_type_changes.rs` — inside `#[cfg(test)] mod tests`, calls `registry.insert(...)` |
| `state/run/node` | `systems/spawn_cells_from_layout/tests/helpers.rs` — calls `registry.insert(...)` |
| `state/run/node` | `definition/tests/helpers.rs` — calls `registry.insert(...)` |

Note: `chips/offering/tests/` also uses a registry's `insert` method, but that is `ChipCatalog::insert`, not `CellTypeRegistry::insert`.

---

## Category 5 — `CellWidth::half_width()` and `CellHeight::half_height()` (cells domain)

**Location:** `breaker-game/src/cells/components/types.rs`

**Items:**
- `CellWidth::half_width(&self) -> f32` — `#[cfg(test)]`, `pub(crate)`
- `CellHeight::half_height(&self) -> f32` — `#[cfg(test)]`, `pub(crate)`

**What they are:** Accessor helpers that return half the stored dimension value. In production builds the value field is stripped from `CellWidth` and `CellHeight` unless `feature = "dev"` is also active, so these methods are only meaningful in test builds. Production code uses `BaseWidth::half_width()` and `BaseHeight::half_height()` from `shared::components`, which are NOT test-gated.

**Cross-domain consumers:**

| Consumer domain | Location |
|---|---|
| `bolt` | `systems/bolt_cell_collision/tests/helpers.rs` — calls `.half_width()` and `.half_height()` on `CellWidth` / `CellHeight` instances obtained via `CellConfig::default()` |

The `bolt` test file spawns cell entities for collision tests and needs exact cell dimensions to position bolts correctly.

---

## Non-Cross-Domain Test-Gated Items (intra-domain only)

These items are `#[cfg(test)]`-gated and `pub(crate)`, but all their consumers are within the same domain. Listed here for completeness so they are not confused with cross-domain items.

### `chips` domain

- `ChipTemplateRegistry::get(name) -> Option<&(AssetId, ChipTemplate)>` — used in `chips/systems/build_chip_catalog/tests/` only
- `ChipTemplateRegistry::len() -> usize` — same
- `EvolutionTemplateRegistry::get(name) -> Option<&(AssetId, EvolutionTemplate)>` — same
- `EvolutionTemplateRegistry::len() -> usize` — same
- `chips::mod.rs` re-exports `Recipe` with `#[cfg(test)]` — only consumed in `chips/resources/tests/`

### `state/run/node/definition` domain

- `NodeDefinition::validate(&self, registry: &CellTypeRegistry) -> Result<(), String>` — `#[cfg(test)]`, `pub(crate)`, used only in `state/run/node/definition/tests/`
- `MAX_GRID_COLS: u32 = 128` — `#[cfg(test)]`, `pub(crate)`, used only in `definition/tests/`
- `MAX_GRID_ROWS: u32 = 128` — same
- `MIN_ENTITY_SCALE: f32 = 0.5` — same
- `MAX_ENTITY_SCALE: f32 = 1.0` — same

### Builder typestate re-exports (bolt, breaker, wall)

Each builder's `core/mod.rs` has:
```rust
#[cfg(test)]
pub(crate) use types::*;
```

This re-exports typestate marker types (`BoltBuilder<...>`, `NoPosition`, `HasPosition`, `WallBuilder<...>`, `NoSide`, `Left`, `Right`, etc.) from the `types` module. These are used only within each domain's own `builder/tests/` for type-annotation tests (e.g. asserting that `.at_position()` transitions the builder to `HasPosition`). No cross-domain consumers.

- `bolt/builder/core/mod.rs` → consumed only in `bolt/builder/tests/`
- `breaker/builder/core/mod.rs` → consumed only in `breaker/builder/tests/`
- `wall/builder/core/mod.rs` → consumed only in `wall/builder/tests/`

### `effect/triggers/until` domain

- `until::system::register(app: &mut App)` — `#[cfg(test)]`, `pub(crate)`, used only in `effect/triggers/until/tests/helpers.rs`

---

## No Shared Test Utility Modules

There is no `test_utils.rs`, `test_helpers.rs`, or shared test module at the crate root or in `shared/`. Every test helper file is:
1. Inside a `tests/` subdirectory of the system being tested
2. Declared `pub(super)` — visible only within its test subtree
3. Named `helpers.rs` by convention

The pattern is consistently local. There is no "common test infrastructure" crate or module.

---

## Implications for Cross-Domain Re-Export Modules

If re-export modules are created (e.g. `crate::cells_test_support` or similar), these items need `#[cfg(test)]` gating on the re-export:

| Item | Must-have `#[cfg(test)]` re-export? | Reason |
|---|---|---|
| `CellTypeDefinition` | Yes | Currently `#[cfg(test)]` gated in `cells::mod` |
| `CellTypeRegistry::insert` | Yes | Method only exists under `#[cfg(test)]` |
| `CellTypeRegistry::contains` | Yes | Same |
| `CellTypeRegistry::len` | Yes | Same |
| `CellWidth::half_width` | Yes | Method only exists under `#[cfg(test)]` |
| `CellHeight::half_height` | Yes | Same |
| `ChipDefinition::test*` | Yes | Methods only exist under `#[cfg(test)]` |
| `EffectKind::test_shockwave` | Yes | Method only exists under `#[cfg(test)]` |

Items that are used in tests but are NOT test-gated (already fully public, no re-export gating needed):
- `Bolt::builder()` — plain `pub fn`, used broadly in effect/cells/chips tests
- `Breaker::builder()` — plain `pub fn`, used in chips/debug tests
- `Wall::builder()` — plain `pub fn`, used in cells tests
- `BoltDefinition` (struct) — plain `pub`
- `CellTypeRegistry::get` — plain `pub(crate)`, works everywhere

---

## Key Files

- `breaker-game/src/chips/definition/types.rs` — `ChipDefinition::test*` factory methods under `#[cfg(test)] impl`
- `breaker-game/src/effect/core/types/tests.rs` — `EffectKind::test_shockwave` under `#[cfg(test)] impl`
- `breaker-game/src/cells/mod.rs` — `#[cfg(test)] pub(crate) use definition::CellTypeDefinition`
- `breaker-game/src/cells/resources.rs` — `CellTypeRegistry::contains`, `insert`, `len` under `#[cfg(test)]`
- `breaker-game/src/cells/components/types.rs` — `CellWidth::half_width`, `CellHeight::half_height` under `#[cfg(test)]`
- `breaker-game/src/bolt/builder/core/mod.rs` — `#[cfg(test)] pub(crate) use types::*` (intra-domain only)
- `breaker-game/src/breaker/builder/core/mod.rs` — same pattern (intra-domain only)
- `breaker-game/src/wall/builder/core/mod.rs` — same pattern (intra-domain only)
