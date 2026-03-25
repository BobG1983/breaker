---
name: defaults-config-pipeline
description: End-to-end data flow for the defaults/config system: GameConfig derive macro, DefaultsCollection, asset loading, seed systems, progress tracking, and hot-reload. Accurate for Bevy 0.18 + bevy_asset_loader + iyes_progress.
type: reference
---

# Defaults / Config Pipeline

## 1. The `GameConfig` Derive Macro

**Crate:** `rantzsoft_defaults_derive` (proc-macro crate), re-exported from `rantzsoft_defaults`.

**Usage annotation required on the `*Defaults` struct:**
```rust
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "FooConfig")]
pub struct FooDefaults { ... }
```

**What the macro generates:**
1. A `FooConfig` struct with identical named fields, deriving `Resource + Debug + Clone`
2. `impl From<FooDefaults> for FooConfig` — field-by-field copy (no doc attrs on the config type, only forwarded `#[doc]` attrs from each field)
3. `impl Default for FooConfig` — delegates to `FooDefaults::default().into()`

No Bevy systems, plugins, or asset loader wiring. Pure code generation.

---

## 2. All `*Defaults` Structs That Derive `GameConfig`

| `*Defaults` type | `*Config` generated | File | RON asset key in `DefaultsCollection` |
|---|---|---|---|
| `PlayfieldDefaults` | `PlayfieldConfig` | `shared/mod.rs` | `collection.playfield` |
| `BoltDefaults` | `BoltConfig` | `bolt/resources.rs` | `collection.bolt` |
| `BreakerDefaults` | `BreakerConfig` | `breaker/resources.rs` | `collection.breaker` |
| `CellDefaults` | `CellConfig` | `cells/resources.rs` | `collection.cell_defaults` |
| `InputDefaults` | `InputConfig` | `input/resources.rs` | `collection.input` |
| `MainMenuDefaults` | `MainMenuConfig` | `screen/main_menu/resources.rs` | `collection.main_menu` |
| `TimerUiDefaults` | `TimerUiConfig` | `ui/resources.rs` | `collection.timer_ui` |
| `ChipSelectDefaults` | `ChipSelectConfig` | `screen/chip_select/resources.rs` | `collection.chip_select` |
| `TransitionDefaults` | `TransitionConfig` | `fx/transition.rs` | **NOT in DefaultsCollection** (see §6) |
| `HighlightDefaults` | `HighlightConfig` | `run/definition.rs` | **NOT in DefaultsCollection** (see §6) |

**Non-`GameConfig` but loaded via `DefaultsCollection`:**
- `DifficultyCurveDefaults` — derives `Asset + TypePath + Deserialize + Clone + Debug` only (no `GameConfig`). Has a hand-written `impl From<DifficultyCurveDefaults> for DifficultyCurve`. `DifficultyCurve` is the live `Resource`.

**Collection-style (registry building, not Config resources):**
- `Vec<Handle<CellTypeDefinition>>` — builds `CellTypeRegistry`
- `Vec<Handle<NodeLayout>>` — builds `NodeLayoutRegistry` (depends on `CellTypeRegistry`)
- `Vec<Handle<ArchetypeDefinition>>` — builds `ArchetypeRegistry`
- `Vec<Handle<ChipDefinition>>` — two consumers: `seed_chip_registry` (non-evolution only) + `seed_evolution_registry` (evolution only)
- `Vec<Handle<ChipTemplate>>` — consumed by `seed_chip_registry` via `expand_template()`

---

## 3. `DefaultsCollection` Definition

```
breaker-game/src/screen/loading/resources.rs
```

```rust
#[derive(AssetCollection, Resource)]
pub(crate) struct DefaultsCollection {
    #[asset(path = "config/defaults.playfield.ron")]  pub playfield: Handle<PlayfieldDefaults>,
    #[asset(path = "config/defaults.bolt.ron")]       pub bolt: Handle<BoltDefaults>,
    #[asset(path = "config/defaults.breaker.ron")]    pub breaker: Handle<BreakerDefaults>,
    #[asset(path = "config/defaults.cells.ron")]      pub cell_defaults: Handle<CellDefaults>,
    #[asset(path = "config/defaults.input.ron")]      pub input: Handle<InputDefaults>,
    #[asset(path = "config/defaults.mainmenu.ron")]   pub main_menu: Handle<MainMenuDefaults>,
    #[asset(path = "config/defaults.timerui.ron")]    pub timer_ui: Handle<TimerUiDefaults>,
    #[asset(path = "cells", collection(typed))]       pub cells: Vec<Handle<CellTypeDefinition>>,
    #[asset(path = "nodes", collection(typed))]       pub nodes: Vec<Handle<NodeLayout>>,
    #[asset(path = "breakers", collection(typed))]    pub breakers: Vec<Handle<ArchetypeDefinition>>,
    #[asset(path = "config/defaults.chipselect.ron")] pub chip_select: Handle<ChipSelectDefaults>,
    #[asset(path = "chips", collection(typed))]       pub chips: Vec<Handle<ChipDefinition>>,
    #[asset(path = "chips", collection(typed))]       pub chip_templates: Vec<Handle<ChipTemplate>>,
    #[asset(path = "config/defaults.difficulty.ron")] pub difficulty: Handle<DifficultyCurveDefaults>,
}
```

**Key facts:**
- `DefaultsCollection` is a Bevy `Resource` and also a `bevy_asset_loader` `AssetCollection`.
- All asset paths are relative to `assets/`.
- Collection fields (`collection(typed)`) load all files in a directory matching the registered extension.
- `chips` and `chip_templates` are two different asset types loaded from the same `chips/` directory. `ChipDefinition` → `.evolution.ron`; `ChipTemplate` → `.chip.ron` (unique extensions, set in `RonAssetPlugin` registrations).
- `DefaultsCollection` itself is NOT manually inserted — `bevy_asset_loader` inserts it as a resource once all asset loads succeed.

---

## 4. Asset Loading Pipeline (Phase 1: ScreenPlugin setup)

```
breaker-game/src/screen/plugin.rs
```

### Step A — Register RON asset plugins (unique file extension per type)

`ScreenPlugin::build` calls `app.add_plugins(RonAssetPlugin::<T>::new(&["ext.ron"]))` for all 14 asset types. Each unique extension prevents `bevy_common_assets` from ambiguously trying every loader on every file.

### Step B — Register state machine

```rust
app.init_state::<GameState>()     // starts in GameState::Loading (default)
   .add_sub_state::<PlayingState>()
```

### Step C — Register progress plugin

```rust
app.add_plugins(
    ProgressPlugin::<GameState>::new()
        .with_state_transition(GameState::Loading, GameState::MainMenu)
)
```

`iyes_progress` tracks a global progress counter per state. When `done >= total` in `GameState::Loading`, it automatically drives `NextState` to `GameState::MainMenu`.

### Step D — Register asset loader

```rust
app.add_loading_state(
    LoadingState::new(GameState::Loading)
        .load_collection::<DefaultsCollection>()
)
```

`bevy_asset_loader` fires all asset loads declared in `DefaultsCollection` when `GameState::Loading` is entered. Once all handles resolve, it inserts `DefaultsCollection` as a resource.

---

## 5. Config Seeding (Phase 2: LoadingPlugin systems)

```
breaker-game/src/screen/loading/plugin.rs
breaker-game/src/screen/loading/systems/seed_*.rs
```

All seed systems run in `Update` while `in_state(GameState::Loading)`, registered via `.track_progress::<GameState>()`.

### Seed system pattern (for `*Config` resources from `*Defaults` assets)

```rust
pub(crate) fn seed_foo_config(
    collection: Option<Res<DefaultsCollection>>,  // Option: not yet inserted if assets still loading
    assets: Res<Assets<FooDefaults>>,
    mut commands: Commands,
    mut seeded: Local<bool>,                       // idempotency guard
) -> Progress {
    if *seeded { return Progress { done: 1, total: 1 }; }
    let Some(collection) = collection else { return Progress { done: 0, total: 1 }; };
    let Some(defaults) = assets.get(&collection.foo) else { return Progress { done: 0, total: 1 }; };
    commands.insert_resource::<FooConfig>(defaults.clone().into()); // From<FooDefaults> via GameConfig macro
    *seeded = true;
    Progress { done: 1, total: 1 }
}
```

**Critical details:**
- `collection` is `Option<Res<>>` because `DefaultsCollection` doesn't exist until `bevy_asset_loader` finishes — the seed system polls for it.
- `assets.get(&collection.foo)` also returns `None` while the handle is unresolved — double guard.
- `Local<bool>` is the idempotency guard: once seeded, always returns `done:1/total:1` without re-reading.
- The system returns `Progress { done, total }`, which `iyes_progress` aggregates into the global counter.
- Command insertion is deferred: `PlayfieldConfig` (etc.) is NOT available until the frame after `seed_*_config` runs and commands are flushed.

### Full list of seeding operations (14 systems)

| System | Output resource | Input collection field |
|---|---|---|
| `seed_playfield_config` | `PlayfieldConfig` | `collection.playfield` |
| `seed_bolt_config` | `BoltConfig` | `collection.bolt` |
| `seed_breaker_config` | `BreakerConfig` | `collection.breaker` |
| `seed_cell_config` | `CellConfig` | `collection.cell_defaults` |
| `seed_input_config` | `InputConfig` | `collection.input` |
| `seed_main_menu_config` | `MainMenuConfig` | `collection.main_menu` |
| `seed_timer_ui_config` | `TimerUiConfig` | `collection.timer_ui` |
| `seed_chip_select_config` | `ChipSelectConfig` | `collection.chip_select` |
| `seed_difficulty_curve` | `DifficultyCurve` | `collection.difficulty` |
| `seed_cell_type_registry` | `CellTypeRegistry` | `collection.cells` (all handles) |
| `seed_node_layout_registry` | `NodeLayoutRegistry` | `collection.nodes` (depends on `CellTypeRegistry` existing) |
| `seed_archetype_registry` | `ArchetypeRegistry` | `collection.breakers` |
| `seed_chip_registry` | `ChipRegistry` | `collection.chip_templates` (expanded via `expand_template`) + `collection.chips` (non-evolution only) |
| `seed_evolution_registry` | `EvolutionRegistry` | `collection.chips` (evolution-rarity only) |

### Dependency chain

`seed_node_layout_registry` has an explicit dependency: it also takes `Option<Res<CellTypeRegistry>>` and returns `done:0` until `CellTypeRegistry` exists. This means `seed_cell_type_registry` must complete (and flush commands) before `seed_node_layout_registry` can proceed. All other seed systems are independent.

### Registry building (not `*Config` resources)

- `CellTypeRegistry`: `HashMap<char, CellTypeDefinition>`. Validates each definition (hp > 0), asserts alias != '.', asserts no duplicate aliases (panics on dup — data authoring error).
- `NodeLayoutRegistry`: validates each layout against `CellTypeRegistry`; returns `done:0` (non-panic) on invalid layout.
- `ArchetypeRegistry`: asserts no duplicate archetype names (panics on dup).
- `ChipRegistry`: expands `ChipTemplate`s via `expand_template()` → each `RaritySlot` (common/uncommon/rare/legendary) becomes one `ChipDefinition` with name `"{prefix} {template.name}"`. Also absorbs non-evolution `ChipDefinition` assets for backward compatibility.
- `EvolutionRegistry`: wraps evolution `ChipDefinition`s as `EvolutionRecipe { ingredients, result_definition }`.

---

## 6. Progress Tracking and State Transition

**Mechanism:** `iyes_progress::ProgressPlugin<GameState>` + `.track_progress::<GameState>()`.

Every seed system returns `Progress { done: u32, total: u32 }`. The `.track_progress::<GameState>()` wrapper registers this return value into the `ProgressTracker<GameState>` resource.

`ProgressPlugin` checks each frame: when `global.done >= global.total` across all tracked systems, it drives `NextState<GameState>` to `GameState::MainMenu`.

**Loading bar system** (`update_loading_bar`, `Update` while `Loading`):
```rust
let global = progress.get_global_progress();
let ratio = global.done / global.total;
// Sets Node::width = Val::Percent(ratio * 100.0) on LoadingBarFill entity
// Updates Text with "Loading... {done}/{total}" on LoadingProgressText entity
```

The loading bar UI is spawned by `spawn_loading_screen` in `OnEnter(GameState::Loading)` and cleaned up by `cleanup_entities::<LoadingScreen>` in `OnExit(GameState::Loading)`.

**Loading screen UI hierarchy:**
```
(LoadingScreen, Node 100%x100% column center)
  ├── (LoadingProgressText, Text "Loading...")
  └── (Node bar background 720x43px, BackgroundColor dim white)
        └── (LoadingBarFill, Node width=0% to 100%, BackgroundColor blue)
```

---

## 7. Config Resources Available Post-Loading

After `GameState::Loading` exits (state transition to `GameState::MainMenu`), the following resources are guaranteed to exist in the Bevy world:

**`*Config` resources (all derived via `GameConfig` macro):**
- `PlayfieldConfig`, `BoltConfig`, `BreakerConfig`, `CellConfig`
- `InputConfig`, `MainMenuConfig`, `TimerUiConfig`, `ChipSelectConfig`

**Non-macro runtime resource:**
- `DifficultyCurve` (hand-written `From<DifficultyCurveDefaults>`)

**Registry resources:**
- `CellTypeRegistry`, `NodeLayoutRegistry`, `ArchetypeRegistry`
- `ChipRegistry`, `EvolutionRegistry`

**The `DefaultsCollection` resource** remains alive in the world — it is not removed after loading. Hot-reload systems use it to correlate `AssetEvent::Modified` events back to the correct handle.

---

## 8. Configs NOT Loaded Through `DefaultsCollection`

Two `GameConfig`-derived types are **not** in `DefaultsCollection`:

**`TransitionConfig`** (`fx/transition.rs`):
- `FxPlugin` does NOT call `init_resource::<TransitionConfig>()`.
- `TransitionConfig` is used directly via `Res<TransitionConfig>` in `spawn_transition_out` / `spawn_transition_in`.
- Tests insert it manually via `app.insert_resource(TransitionConfig::default())`.
- There is no seed system or hot-reload for `TransitionConfig`. It relies on `Default` (which delegates to `TransitionDefaults::default().into()`).
- **Gap:** `TransitionConfig` is never explicitly inserted at runtime outside tests — it works only because `Default` is implemented and Bevy can init it via `init_resource`.

**`HighlightConfig`** (`run/definition.rs`):
- `RunPlugin::build` calls `app.init_resource::<HighlightConfig>()` — this inserts the default.
- `HighlightDefaults` file `assets/config/defaults.highlights.ron` exists and parses correctly, but there is NO seed system that loads it via `DefaultsCollection`.
- `HighlightConfig` is therefore always at its Rust default values in production (not hot-reloadable and not read from the RON file at runtime). The RON file is only tested via `include_str!` in unit tests.
- **Gap:** `HighlightDefaults` derives `GameConfig` and has a RON file, but the pipeline from file → `DefaultsCollection` → `seed_highlight_config` → `HighlightConfig` does not exist.

---

## 9. Hot-Reload Path (debug only)

```
breaker-game/src/debug/hot_reload/plugin.rs
```

`HotReloadPlugin` runs during `GameState::Playing` only, in two ordered system sets:

**Set 1: `HotReloadSystems::PropagateDefaults`** — watches `AssetEvent<FooDefaults>`, re-seeds `FooConfig` on `Modified`:

| System | Asset type watched | Config resource re-seeded |
|---|---|---|
| `propagate_bolt_defaults` | `BoltDefaults` | `BoltConfig` |
| `propagate_breaker_defaults` | `BreakerDefaults` | `BreakerConfig` |
| `propagate_cell_defaults` | `CellDefaults` | `CellConfig` |
| `propagate_playfield_defaults` | `PlayfieldDefaults` | `PlayfieldConfig` |
| `propagate_input_defaults` | `InputDefaults` | `InputConfig` |
| `propagate_timer_ui_defaults` | `TimerUiDefaults` | `TimerUiConfig` |
| `propagate_main_menu_defaults` | `MainMenuDefaults` | `MainMenuConfig` |
| `propagate_chip_select_defaults` | `ChipSelectDefaults` | `ChipSelectConfig` |
| `propagate_cell_type_changes` | `CellTypeDefinition` | `CellTypeRegistry` (rebuilt) |
| `propagate_node_layout_changes` | `NodeLayout` | `NodeLayoutRegistry` (rebuilt, after cell type changes) |
| `propagate_archetype_changes` | `ArchetypeDefinition` | `ArchetypeRegistry` (rebuilt) |

`propagate_cell_type_changes.before(propagate_node_layout_changes)` — explicit ordering so cell types are updated before layouts re-validate against them.

**Set 2: `HotReloadSystems::PropagateConfig`** — runs after Set 1; propagates config resource changes into existing entity components:

- `propagate_bolt_config.run_if(resource_changed::<BoltConfig>)` — overwrites components on bolt entities
- `propagate_breaker_config.run_if(resource_changed::<BreakerConfig>)` — overwrites components on breaker entities

Hot-reload only watches `AssetEvent::Modified` (not `Added`). The handle ID is verified against `collection.bolt` etc. to avoid reacting to unrelated asset changes.

---

## 10. Key Files

| File | Role |
|---|---|
| `rantzsoft_defaults_derive/src/lib.rs` | Proc-macro: `GameConfig` derive generates `*Config` struct + `From` + `Default` |
| `rantzsoft_defaults/src/lib.rs` | Re-export shim: `pub use rantzsoft_defaults_derive::GameConfig` |
| `breaker-game/src/screen/loading/resources.rs` | `DefaultsCollection` — all 14 asset handles/collections |
| `breaker-game/src/screen/plugin.rs` | Registers `RonAssetPlugin`s, `ProgressPlugin`, `LoadingState::load_collection::<DefaultsCollection>()` |
| `breaker-game/src/screen/loading/plugin.rs` | Registers 14 seed systems with `.track_progress::<GameState>()` |
| `breaker-game/src/screen/loading/systems/seed_*.rs` | One file per seeding operation (14 files) |
| `breaker-game/src/screen/loading/systems/update_loading_bar.rs` | Progress bar UI update from `ProgressTracker<GameState>` |
| `breaker-game/src/debug/hot_reload/plugin.rs` | Hot-reload: `PropagateDefaults` and `PropagateConfig` system sets |
| `breaker-game/src/shared/mod.rs` | `PlayfieldDefaults`, `PlayfieldConfig` (with extra methods), `GameState` |
| `breaker-game/src/run/definition.rs` | `DifficultyCurveDefaults`, `HighlightDefaults` / `HighlightConfig` (gap: highlight not seeded at runtime) |
| `breaker-game/src/fx/transition.rs` | `TransitionDefaults` / `TransitionConfig` (gap: not seeded at runtime, relies on `Default`) |
