# Research: Dev-Gated Types in `breaker-game/src/`

## Summary

The `dev` feature enables `bevy_egui`, `bevy/file_watcher`, and `rantzsoft_defaults/hot-reload`.
All `#[cfg(feature = "dev")]` gating in game code falls into four domains:
`debug/`, `breaker/queries/`, `chips/systems/`, and `state/run/node/systems/`.
There are no dev-only types that cross domain boundaries via `pub` re-exports —
the gating pattern is consistently: modules that are dev-only are declared with `#[cfg(feature = "dev")]`
at the module level, and any `pub(crate)` items inside them are only accessible
when the feature is enabled.

---

## Domain-by-Domain Inventory

### `debug/` domain

The entire `debug/` domain is unconditionally part of the crate (`pub(crate) mod debug` in `lib.rs`),
but most of its submodules are individually gated in `debug/mod.rs`:

```rust
// debug/mod.rs
#[cfg(feature = "dev")] mod hot_reload;
#[cfg(feature = "dev")] mod overlays;
mod plugin;                                 // always present (no-op without dev)
#[cfg(feature = "dev")] pub(crate) mod recording;
pub(crate) mod resources;                   // always present — see below
#[cfg(feature = "dev")] mod telemetry;

pub(crate) use plugin::DebugPlugin;         // always present
```

#### `debug/resources.rs` — partially gated, always compiled

| Item | Kind | Visibility | Gate | Cross-domain usage |
|------|------|------------|------|--------------------|
| `Overlay` (enum) | enum | `pub(crate)` | `#[cfg(feature = "dev")]` | No — only used inside `debug/` |
| `DebugOverlays` | Resource | `pub(crate)` | `#[cfg(feature = "dev")]` | No — only used inside `debug/` |
| `LastBumpResult` | Resource | `pub(crate)` | `#[cfg(feature = "dev")]` | No — only used inside `debug/` |

The `resources.rs` file is always compiled (not gated), but its types only exist when `dev` is enabled.
The file contains tests that are themselves gated with `#[cfg(feature = "dev")]`.

#### `debug/plugin.rs` — always compiled, dev logic in block

| Item | Kind | Visibility | Gate |
|------|------|------------|------|
| `DebugPlugin` | Plugin | `pub(crate)` | None (always present, no-op without dev) |

The plugin `build()` body wraps all dev-specific registration in `#[cfg(feature = "dev")]` block.
`DebugPlugin` itself is always re-exported and always added to the `Game` plugin group.

#### `debug/recording/` — gated module, no cross-domain type exports

| Item | Kind | Visibility | Cross-domain usage |
|------|------|------------|---------------------|
| `RecordingPlugin` | Plugin | `pub(crate)` | No |
| `RecordingConfig` | Resource | `pub(crate)` | No — but `app.rs::apply_dev_flags` inserts it |

`RecordingConfig` is imported by `app.rs::apply_dev_flags` (also dev-gated):
```rust
// app.rs
#[cfg(feature = "dev")]
pub fn apply_dev_flags(app: &mut App) {
    use crate::debug::recording::RecordingConfig;
    ...
}
```
This is a `pub fn` that is only compiled under `dev`. The `RecordingPlugin` is registered inside
`debug/plugin.rs`'s dev-gated block. So no leakage outside the gated path.

`capture_frame.rs` inside `recording/systems/` imports `crate::run::node::ActiveNodeLayout`
and `crate::input::resources::InputActions` — these are non-dev types. The import is safe
because the whole `recording` module is gated. No dev-only type escapes into those domains.

#### `debug/hot_reload/` — gated module, imports from many domains

The hot-reload systems read from breaker, bolt, cells, chips, and run/node domains.
All cross-domain reads are of non-dev types (registries, components, resources).
The systems live entirely inside the dev-gated `hot_reload` module.

Key cross-domain imports (all safe — consumers are inside gated module):

| System | Cross-domain types consumed |
|--------|-----------------------------|
| `propagate_breaker_changes` | `BreakerRegistry`, `SelectedBreaker`, `Breaker`, all breaker components, `BoundEffects`, `RootEffect`, `Target`, `LivesCount` |
| `propagate_bolt_definition` | `BoltRegistry`, `Bolt`, `BoltBaseDamage`, `BoltDefinitionRef`, `BoundEffects`, `EffectCommandsExt`, `EffectNode`, `RootEffect`, `Target` |
| `propagate_cell_type_changes` | `CellTypeRegistry`, all `cells::components::*` |
| `propagate_node_layout_changes` | `CellConfig`, `CellTypeRegistry`, `Cell`, `ActiveNodeLayout`, `ClearRemainingCount`, `NodeLayoutRegistry`, `PlayfieldConfig`, `RenderAssets`, `spawn_cells_from_grid` |
| `propagate_chip_catalog` (from `chips/`) | `ChipTemplateRegistry`, `EvolutionTemplateRegistry`, `ChipCatalog` |

None of these cross-domain imports are dev-only types. The only dev-gated imports are
`RenderAssets` and `spawn_cells_from_grid` from `state::run::node::systems` (see below).

#### `debug/telemetry/` — gated module, cross-domain reads

| System | Cross-domain types consumed (dev-only or not) |
|--------|-------------------------------------------------|
| `bolt_info_ui` | `Bolt` (non-dev), `DebugOverlays` (dev), `Overlay` (dev) |
| `breaker_state_ui` | `Breaker` (non-dev), `BreakerTelemetryData` (dev — see below), `DebugOverlays` (dev), `LastBumpResult` (dev), `Overlay` (dev) |
| `input_actions_ui` | `InputActions` (non-dev), `DebugOverlays` (dev), `Overlay` (dev) |
| `track_bump_result` | `BumpGrade`, `BumpPerformed`, `BumpWhiffed` (all non-dev), `LastBumpResult` (dev) |
| `debug_ui_system` | `GameState` (non-dev), `DebugOverlays` (dev), `Overlay` (dev) |

The `breaker_state_ui` system queries `BreakerTelemetryData` — this is a dev-gated QueryData struct
defined in the `breaker` domain (see below). Its usage is safe because `telemetry` is itself dev-gated.

#### `debug/overlays/` — gated module, reads dev-gated `CellWidth`/`CellHeight` fields

| System | Cross-domain types consumed |
|--------|-----------------------------|
| `draw_hitboxes` | `Bolt`, `BoltRadius`, `BaseHeight`, `BaseWidth`, `Breaker`, `Cell`, `CellHeight`, `CellWidth`, `DebugOverlays`, `Overlay` |
| `draw_velocity_vectors` | `Bolt`, `Breaker`, `Velocity2D`, `DebugOverlays`, `Overlay` |

`draw_hitboxes` accesses `cell_w.value` and `cell_h.value` — these fields only exist under
`#[cfg(any(test, feature = "dev"))]`. This is safe because `overlays` is gated behind
`#[cfg(feature = "dev")]`.

---

### `breaker/queries/` domain

#### `BreakerTelemetryData` — dev-only QueryData struct

| Item | Kind | Visibility | Gate | Cross-domain usage |
|------|------|------------|------|--------------------|
| `BreakerTelemetryData` | `#[derive(QueryData)]` struct | `pub(crate)` | `#[cfg(feature = "dev")]` | Yes — consumed by `debug/telemetry/systems/breaker_state_ui.rs` |

`BreakerTelemetryData` is defined in `breaker/queries/data.rs` and re-exported via
`pub(crate) use data::*` in `breaker/queries/mod.rs`. The `*` glob naturally propagates the
cfg gate — the item only exists when `dev` is on, so callers that import it must be in
dev-only paths (which `debug/telemetry/` is).

**Conclusion:** `BreakerTelemetryData` is used cross-domain (breaker → debug) but only under
`dev`. The re-export path from `breaker/queries/mod.rs` does not need an explicit
`#[cfg(feature = "dev")]` because the glob-exported item simply doesn't exist without the feature.
**However, any new `crate::components` re-export module that includes `breaker::queries::*`
would need `#[cfg(feature = "dev")]` on the `BreakerTelemetryData` re-export line.**

---

### `chips/systems/` domain

#### `propagate_chip_catalog` — dev-only system function

| Item | Kind | Visibility | Gate | Cross-domain usage |
|------|------|------------|------|--------------------|
| `propagate_chip_catalog` (fn) | system | `pub(crate)` | `#[cfg(feature = "dev")]` | Yes — imported by `debug/hot_reload/plugin.rs` |

Export chain:
```
chips/systems/build_chip_catalog/system.rs     — #[cfg(feature = "dev")] pub(crate) fn propagate_chip_catalog
chips/systems/build_chip_catalog/mod.rs        — #[cfg(feature = "dev")] pub(crate) use system::propagate_chip_catalog
chips/systems/mod.rs                           — #[cfg(feature = "dev")] pub(crate) use build_chip_catalog::propagate_chip_catalog
```
All three re-export layers carry the `#[cfg(feature = "dev")]` gate.

**Conclusion:** If a `crate::systems` re-export module re-exports `chips::systems::*`,
`propagate_chip_catalog` will only appear under `dev` (since the item doesn't exist otherwise).
No explicit gate needed on the re-export line, but it should be documented.

---

### `state/run/node/systems/` domain

#### `RenderAssets` and `spawn_cells_from_grid` — dev-only re-exports

| Item | Kind | Visibility | Gate | Cross-domain usage |
|------|------|------------|------|--------------------|
| `RenderAssets` (struct) | helper struct | `pub(crate)` (unconditional in `system.rs`) | Re-export gated in `mod.rs` | Yes — `debug/hot_reload/systems/propagate_node_layout_changes.rs` |
| `spawn_cells_from_grid` (fn) | helper fn | `pub(crate)` (unconditional in `system.rs`) | Re-export gated in `mod.rs` | Yes — `debug/hot_reload/systems/propagate_node_layout_changes.rs` |

The items themselves live in `spawn_cells_from_layout/system.rs` without any feature gate —
they are always compiled. The feature gate only appears on the re-export in `systems/mod.rs`:

```rust
// state/run/node/systems/mod.rs
#[cfg(feature = "dev")]
pub(crate) use spawn_cells_from_layout::RenderAssets;
#[cfg(feature = "dev")]
pub(crate) use spawn_cells_from_layout::spawn_cells_from_grid;
pub(crate) use spawn_cells_from_layout::spawn_cells_from_layout;  // always exported
```

**Distinction:** `spawn_cells_from_layout` (the Bevy system) is always exported.
`spawn_cells_from_grid` (the inner helper) and `RenderAssets` (its parameter struct) are only
re-exported under `dev` — because the only consumer outside the module is the hot-reload system.

**Conclusion:** Any new `crate::systems` re-export module that re-exports
`state::run::node::systems::*` would need `#[cfg(feature = "dev")]` on the
`RenderAssets` and `spawn_cells_from_grid` re-export lines.

---

### `cells/components/` domain — conditional struct fields (not type-level gating)

`CellWidth` and `CellHeight` are NOT dev-gated types — they always exist. Only their internal
`value: f32` field and the `new(value: f32)` constructor are conditional:

```rust
pub(crate) struct CellWidth {
    #[cfg(any(test, feature = "dev"))]
    pub value: f32,
}

impl CellWidth {
    #[cfg(any(test, feature = "dev"))]
    pub(crate) const fn new(value: f32) -> Self { Self { value } }

    #[cfg(not(any(test, feature = "dev")))]
    pub(crate) const fn new(_value: f32) -> Self { Self {} }
}
```

| Pattern | Gate | Effect |
|---------|------|--------|
| `CellWidth::new(x)` in production code | Always callable (two impls) | In non-dev builds, drops the value |
| `cell_w.value` field access | `any(test, feature = "dev")` | Compile error in non-dev production builds |

**Cross-domain field access:** `debug/overlays/systems/draw_hitboxes.rs` accesses `.value` —
safe because `overlays` is gated. The `bolt` domain accesses `CellWidth::new()` in test helpers
only (gated by `#[cfg(test)]`) — safe because `test` satisfies `any(test, feature = "dev")`.

**Conclusion:** No re-export module needs special handling for `CellWidth`/`CellHeight` as types.
The types themselves require no feature gate. Only `.value` field access needs dev context,
and all such accesses are already in appropriately gated code.

---

### `app.rs` — dev-only public function

| Item | Kind | Visibility | Gate |
|------|------|------------|------|
| `apply_dev_flags` | function | `pub` | `#[cfg(feature = "dev")]` |

This is used by `main.rs`. It is `pub` (not `pub(crate)`) because `main.rs` calls it from
outside the library crate. Not relevant to cross-domain re-export modules.

---

## Cross-Domain Re-export Impact Matrix

For the planned `crate::components`, `crate::resources`, `crate::systems`, etc. re-export modules:

| Re-export module | Dev-gated items to handle | Recommendation |
|-----------------|--------------------------|----------------|
| `crate::components` | None — no components are dev-gated at type level | No special handling needed |
| `crate::resources` | `DebugOverlays`, `LastBumpResult`, `RecordingConfig` | Gate with `#[cfg(feature = "dev")]` |
| `crate::queries` | `BreakerTelemetryData` | Gate with `#[cfg(feature = "dev")]` |
| `crate::systems` | `propagate_chip_catalog`, `RenderAssets`, `spawn_cells_from_grid` | Gate with `#[cfg(feature = "dev")]` |
| `crate::plugins` | `DebugPlugin` (always present, no-op w/o dev), `RecordingPlugin` | `RecordingPlugin` needs `#[cfg(feature = "dev")]` |

---

## Summary Table: All Dev-Gated Items

| Item | Domain | Kind | Gate | Cross-domain consumer |
|------|--------|------|------|-----------------------|
| `Overlay` | debug | enum | `#[cfg(feature = "dev")]` | No (debug-internal only) |
| `DebugOverlays` | debug | Resource | `#[cfg(feature = "dev")]` | No (debug-internal only) |
| `LastBumpResult` | debug | Resource | `#[cfg(feature = "dev")]` | No (debug-internal only) |
| `apply_dev_flags` | app | fn (pub) | `#[cfg(feature = "dev")]` | main.rs |
| `RecordingPlugin` | debug | Plugin | module-gated | debug/plugin.rs (dev block) |
| `RecordingConfig` | debug | Resource | module-gated | app.rs (dev-gated fn) |
| `HotReloadPlugin` | debug | Plugin | module-gated | debug/plugin.rs (dev block) |
| `HotReloadSystems` | debug | SystemSet | module-gated | debug/hot_reload/plugin.rs |
| `BreakerTelemetryData` | breaker | QueryData | `#[cfg(feature = "dev")]` | debug/telemetry (dev-gated) |
| `propagate_chip_catalog` | chips | system fn | `#[cfg(feature = "dev")]` (×3 layers) | debug/hot_reload (dev-gated) |
| `RenderAssets` | state/run/node | struct | Re-export gated | debug/hot_reload (dev-gated) |
| `spawn_cells_from_grid` | state/run/node | fn | Re-export gated | debug/hot_reload (dev-gated) |
| `CellWidth::value` (field) | cells | field | `#[cfg(any(test, feature = "dev"))]` | debug/overlays (dev-gated), bolt tests (test-gated) |
| `CellHeight::value` (field) | cells | field | `#[cfg(any(test, feature = "dev"))]` | debug/overlays (dev-gated), bolt tests (test-gated) |

---

## Key Findings

1. **All dev-gated cross-domain consumers are themselves in dev-gated modules.**
   The gating is consistent — no non-dev code path reaches a dev-only item.

2. **Three items need explicit gate on any new re-export line** if a re-export module
   exposes them:
   - `BreakerTelemetryData` (breaker domain)
   - `propagate_chip_catalog` (chips domain)
   - `RenderAssets` and `spawn_cells_from_grid` (state/run/node domain — but these are
     implementation details unlikely to belong in a public re-export module)

3. **`DebugPlugin` is always present** (no `#[cfg]` on the type itself) and always added
   to `Game`. The dev feature only controls what the plugin *does* in its `build()` method.
   A `crate::plugins` re-export of `DebugPlugin` needs no feature gate.

4. **`RecordingPlugin` and `RecordingConfig` are inside a dev-gated module** (`debug::recording`).
   A re-export module that exposes them must carry `#[cfg(feature = "dev")]`.

5. **`CellWidth` and `CellHeight` are not dev-only types** — they are always present.
   Only their `.value` field accessor is dev-gated. Re-export modules need no special handling
   for these types, but any code that accesses `.value` must be in a dev-gated context.

6. **The `debug/hot_reload/` module imports non-dev types from many domains.** The hot-reload
   systems are the single largest cross-domain consumer under the `dev` gate, but they consume
   only non-dev types (registries, components, messages) from those domains.
