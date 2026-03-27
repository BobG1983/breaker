---
name: defaults-config-pipeline
description: End-to-end data flow for the defaults/config system after SeedableRegistry feature: RantzDefaultsPluginBuilder, SeedableConfig, SeedableRegistry. DefaultsCollection and 14 seed systems are DELETED.
type: reference
---

# Defaults / Config Pipeline (SeedableRegistry era)

> **NOTE:** `DefaultsCollection` + `bevy_asset_loader` + 14 hand-written seed systems are DELETED as of the SeedableRegistry feature branch. The new pipeline uses `RantzDefaultsPluginBuilder` from `rantzsoft_defaults`.

---

## 1. The `GameConfig` Derive Macro

**Crate:** `rantzsoft_defaults_derive` (proc-macro crate), re-exported from `rantzsoft_defaults`.

**Reversed form — applied to `*Config` struct (game standard):**
```rust
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "FooDefaults",
    path = "config/defaults.foo.ron",
    ext = "foo.ron"
)]
pub struct FooConfig { ... }
```
Generates: `FooDefaults` struct (Asset+TypePath+Deserialize+Clone), bidirectional `From` impls, `Default for FooDefaults` (delegates to `FooConfig::default().into()`), `merge_from_defaults(&FooDefaults)` on `FooConfig`, and `impl SeedableConfig for FooDefaults` with `asset_path()` and `extensions()`.

**No Bevy systems, plugins, or asset loader wiring in the proc-macro itself. Pure code generation.**

All game `*Config` structs now use the reversed form. The forward form (`#[game_config(name = "FooConfig")]` on `*Defaults`) is available in the crate but NOT used by the game.

---

## 2. All `*Config` Structs and Their Defaults Types

| `*Config` type | `*Defaults` generated | File | RON asset path |
|---|---|---|---|
| `PlayfieldConfig` | `PlayfieldDefaults` | `shared/playfield.rs` | `config/defaults.playfield.ron` |
| `BoltConfig` | `BoltDefaults` | `bolt/resources.rs` | `config/defaults.bolt.ron` |
| `BreakerConfig` | `BreakerDefaults` | `breaker/resources.rs` | `config/defaults.breaker.ron` |
| `CellConfig` | `CellDefaults` | `cells/resources.rs` | `config/defaults.cells.ron` |
| `InputConfig` | `InputDefaults` | `input/resources.rs` | `config/defaults.input.ron` |
| `MainMenuConfig` | `MainMenuDefaults` | `screen/main_menu/resources.rs` | `config/defaults.mainmenu.ron` |
| `TimerUiConfig` | `TimerUiDefaults` | `ui/resources.rs` | `config/defaults.timerui.ron` |
| `ChipSelectConfig` | `ChipSelectDefaults` | `screen/chip_select/resources.rs` | `config/defaults.chipselect.ron` |
| `DifficultyCurve` | `DifficultyCurveDefaults` | `run/resources.rs` | `config/defaults.difficulty.ron` |

**NOT loaded through RantzDefaultsPluginBuilder (still use Default only):**
- `TransitionConfig` (`fx/transition.rs`) — `FxPlugin` does NOT seed from file; relies on `Default` impl. Gap accepted.
- `HighlightConfig` (`run/definition.rs`) — `RunPlugin::build` calls `app.init_resource::<HighlightConfig>()`. Gap accepted.

---

## 3. Registry Types

Registries implement `SeedableRegistry` trait from `rantzsoft_defaults::registry`:

| Registry | Asset type | `asset_dir()` | `extensions()` | File |
|---|---|---|---|---|
| `CellTypeRegistry` | `CellTypeDefinition` | `"cells"` | `["cell.ron"]` | `cells/resources.rs` |
| `BreakerRegistry` | `BreakerDefinition` | `"breakers"` | `["bdef.ron"]` | `breaker/registry.rs` |
| `NodeLayoutRegistry` | `NodeLayout` | `"nodes"` | `["layout.ron"]` | `run/node/resources.rs` |
| `ChipTemplateRegistry` | `ChipTemplate` | `"chips/templates"` | `["chip.ron"]` | `chips/resources.rs` |
| `EvolutionRegistry` | `ChipDefinition` | `"chips/evolution"` | `["evolution.ron"]` | `chips/resources.rs` |

**NOTE:** `ChipCatalog` (the flat lookup + recipe table built from templates) is NOT a `SeedableRegistry`. It is populated by `build_chip_catalog` (a separate `LoadingPlugin` system that runs after `ChipTemplateRegistry` is seeded). See §6.

---

## 4. Loading Pipeline (ScreenPlugin)

`ScreenPlugin::build` adds:

```rust
app.add_plugins(
    RantzDefaultsPluginBuilder::<GameState>::new(GameState::Loading)
        .add_config::<PlayfieldDefaults>()
        .add_config::<BoltDefaults>()
        .add_config::<BreakerDefaults>()
        .add_config::<CellDefaults>()
        .add_config::<InputDefaults>()
        .add_config::<MainMenuDefaults>()
        .add_config::<TimerUiDefaults>()
        .add_config::<ChipSelectDefaults>()
        .add_config::<DifficultyCurveDefaults>()
        // Registries
        .add_registry::<CellTypeRegistry>()
        .add_registry::<BreakerRegistry>()
        .add_registry::<NodeLayoutRegistry>()
        .add_registry::<ChipTemplateRegistry>()
        .add_registry::<EvolutionRegistry>()
        .build(),
)
.add_plugins(
    ProgressPlugin::<GameState>::new()
        .with_state_transition(GameState::Loading, GameState::MainMenu),
)
```

### For each `add_config::<D>()`:
- Registers `RonAssetLoader<D>` (reads the file declared by `D::asset_path()`)
- Adds `Startup` system: `init_defaults_handle::<D>` — loads the asset and inserts `DefaultsHandle<D>` resource
- Adds `Update` (in loading state, tracked as progress): `seed_config::<D>` — waits for asset to load, calls `commands.init_resource::<D::Config>()` by converting via `From<D> for D::Config`

### For each `add_registry::<R>()`:
- Registers `RonAssetLoader<R::Asset>` (reads files in `R::asset_dir()` matching `R::extensions()`)
- Adds `Startup` system: `init_registry_handles::<R>` — calls `asset_server.load_folder(R::asset_dir())` and inserts `RegistryHandles<R::Asset>`
- Adds `Update` (in loading state, tracked as progress): `seed_registry::<R>` — waits for folder load, resolves typed handles, calls `registry.seed(&pairs)` after all handles loaded

### Progress tracking:
`iyes_progress::ProgressPlugin<GameState>` drives `GameState::Loading → GameState::MainMenu` when all tracked systems report `done`. Each `seed_config` and `seed_registry` system returns `Progress { done, total }` via `.track_progress::<GameState>()`.

---

## 5. `LoadingPlugin` (remaining systems)

After `DefaultsCollection` deletion, `LoadingPlugin` has only:
- `build_chip_catalog` — `Update`, tracked as progress — builds `ChipCatalog` (flat lookup + recipes) from `ChipTemplateRegistry` + `EvolutionRegistry` after they are seeded
- `spawn_loading_screen` — `OnEnter(GameState::Loading)`
- `update_loading_bar` — `Update, run_if(GameState::Loading)`
- `cleanup_entities::<LoadingScreen>` — `OnExit(GameState::Loading)`

---

## 6. ChipCatalog

`ChipCatalog` is NOT a `SeedableRegistry`. It is a derived resource built from `ChipTemplateRegistry` (which stores raw templates) by `build_chip_catalog`. The system:
- Expands each `ChipTemplate` via `expand_template()` → produces one `ChipDefinition` per `RaritySlot`
- Absorbs `EvolutionRegistry` definitions as `Recipe` entries
- Result: `ChipCatalog` with flat name→ChipDefinition map (insertion-order preserved) and recipe list

`build_chip_catalog` is the only remaining game-specific seed system in `LoadingPlugin`.

---

## 7. Configs Available Post-Loading

After `GameState::Loading` exits, guaranteed resources:

**`*Config` resources (all via `seed_config`):**
- `PlayfieldConfig`, `BoltConfig`, `BreakerConfig`, `CellConfig`
- `InputConfig`, `MainMenuConfig`, `TimerUiConfig`, `ChipSelectConfig`
- `DifficultyCurve` (via `seed_config::<DifficultyCurveDefaults>`)

**Registry resources (all via `seed_registry`):**
- `CellTypeRegistry`, `BreakerRegistry`, `NodeLayoutRegistry`
- `ChipTemplateRegistry`, `EvolutionRegistry`

**Derived registry (via `build_chip_catalog`):**
- `ChipCatalog`

**NOT via DefaultsPlugin (use `Default`):**
- `TransitionConfig` (gap — not seeded from file)
- `HighlightConfig` (gap — init_resource in RunPlugin, uses Default)

---

## 8. Hot-Reload Path (HotReloadPlugin)

`HotReloadPlugin` handles two categories:
- **Layer 2 (PropagateDefaults):** Registry rebuilds on asset change — `propagate_cell_type_changes`, `propagate_node_layout_changes`, `propagate_breaker_changes`. Simple config propagation (bolt, breaker, etc.) is handled automatically by `rantzsoft_defaults::systems::propagate_defaults` — no longer needs per-config hot-reload systems in the game.
- **Layer 3 (PropagateConfig):** Config → entity components — `propagate_bolt_config` (resource_changed::<BoltConfig>), `propagate_breaker_config` (resource_changed::<BreakerConfig>)

---

## 9. Key Files

| File | Role |
|---|---|
| `rantzsoft_defaults/src/registry.rs` | `SeedableRegistry` trait + `RegistryHandles<A>` |
| `rantzsoft_defaults/src/plugin.rs` | `RantzDefaultsPluginBuilder`, `RantzDefaultsPlugin`, `DefaultsSystems` |
| `rantzsoft_defaults/src/seedable.rs` | `SeedableConfig` trait |
| `rantzsoft_defaults/src/systems.rs` | `seed_config`, `seed_registry`, `init_defaults_handle`, `init_registry_handles` |
| `rantzsoft_defaults_derive/src/lib.rs` | Proc-macro: `GameConfig` derive (reversed form) |
| `breaker-game/src/screen/plugin.rs` | `ScreenPlugin` — adds `RantzDefaultsPluginBuilder` + `ProgressPlugin` |
| `breaker-game/src/screen/loading/plugin.rs` | `LoadingPlugin` — `build_chip_catalog`, loading UI |
| `breaker-game/src/chips/resources.rs` | `ChipCatalog`, `ChipTemplateRegistry`, `EvolutionRegistry` |
| `breaker-game/src/breaker/registry.rs` | `BreakerRegistry` (implements SeedableRegistry) |
| `breaker-game/src/debug/hot_reload/plugin.rs` | `HotReloadPlugin` — reduced to registry + entity-component propagation only |
