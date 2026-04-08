# RON Catalog Patterns Research

**Bevy version**: 0.18 (confirmed from `Cargo.toml` workspace)

---

## Summary

There are two distinct catalog patterns in this codebase, both powered by `rantzsoft_defaults`:

1. **Config pattern** (`SeedableConfig`): a single RON file → one `Resource`. Used for tuning values (playfield width, cell size, UI colors).
2. **Registry pattern** (`SeedableRegistry`): a folder of RON files → one `Resource` (a `HashMap`-based registry). Used for named content catalogs (chips, breakers, bolts, evolutions, cell types, node layouts, walls).

`ChipCatalog` is a third-layer structure built on top of two registries. `EvolutionCatalog` does not exist as a separate resource — evolutions are merged into `ChipCatalog` during a build step.

---

## 1. ChipCatalog — Definition and Loading

### Struct

`ChipCatalog` lives in `breaker-game/src/chips/resources/data.rs` and is a `Resource`:

```rust
#[derive(Resource, Debug, Default)]
pub struct ChipCatalog {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,       // preserves insertion order for deterministic display
    recipes: Vec<Recipe>,
}
```

It is NOT a `SeedableRegistry` and NOT a `SeedableConfig`. It is a derived/computed resource built from two registries.

### Two-Layer Architecture

The chip system uses a two-layer design:

**Layer 1 — Raw registries** (loaded from disk via `SeedableRegistry`):
- `ChipTemplateRegistry` — loaded from `assets/chips/standard/*.chip.ron`
- `EvolutionTemplateRegistry` — loaded from `assets/chips/evolutions/*.evolution.ron`

Both implement `SeedableRegistry` with:
- `asset_dir()` returning the folder path
- `extensions()` returning the RON extension suffix
- `seed()` replacing all entries
- `update_single()` upserting one entry on hot-reload

**Layer 2 — ChipCatalog** (computed from layers 1):
Built by `build_chip_catalog` in `Update`, gated on `AppState::Loading`, tracked by `iyes_progress`. Once both registries have `RegistryHandles.loaded == true`, the system:
1. Iterates `ChipTemplateRegistry`, calls `expand_chip_template()` on each — producing 1–4 `ChipDefinition`s per template (one per rarity slot)
2. Iterates `EvolutionTemplateRegistry`, calls `expand_evolution_template()` — producing 1 `ChipDefinition` per evolution plus extracting a `Recipe`
3. Inserts the resulting `ChipCatalog` resource via `commands.insert_resource()`
4. Sets `Local<bool>` to prevent rebuilding on subsequent ticks

### EvolutionCatalog

There is no `EvolutionCatalog` resource. Evolutions are:
- Loaded as raw `EvolutionTemplate` assets via `EvolutionTemplateRegistry`
- Merged into `ChipCatalog` during the build step above
- Evolutions appear in `ChipCatalog.chips` as `ChipDefinition { rarity: Rarity::Evolution, ... }`
- Evolution recipes are stored in `ChipCatalog.recipes`

---

## 2. rantzsoft_defaults — How It Works

### Core Types

| Type | Role |
|------|------|
| `SeedableConfig` | Trait for single-file configs (`asset_path()`, `extensions()`, `Config` assoc type) |
| `SeedableRegistry` | Trait for folder-based registries (`asset_dir()`, `extensions()`, `seed()`, `update_single()`) |
| `RonAssetLoader<T>` | Generic `AssetLoader` that deserializes RON bytes into any `T: DeserializeOwned + Asset` |
| `DefaultsHandle<D>` | `Resource` wrapping a `Handle<D>` — stored so seed/propagate systems can locate it |
| `RegistryHandles<A>` | `Resource` storing folder handle + typed handles + `loaded` flag |
| `GameConfig` (derive macro) | Generates either a paired Config (legacy) or a paired Defaults struct (reversed) |
| `RantzDefaultsPlugin` | Bevy `Plugin` built by `RantzDefaultsPluginBuilder` |
| `DefaultsSystems` | `SystemSet` enum: `Seed` and `PropagateDefaults` |

### Plugin Builder Pattern

`RantzDefaultsPluginBuilder::new(AppState::Loading)` accumulates type-erased closures, then `.build()` returns the plugin:

```
.add_config::<D>() wires up:
  - asset init + RON loader registration
  - Startup: init_defaults_handle (loads single file via AssetServer)
  - Update (run_if Loading, track_progress): seed_config (one-shot, inserts Config resource)
  - Update (#[cfg(hot-reload)]): propagate_defaults (watches AssetEvent::Modified)

.add_registry::<R>() wires up:
  - asset init + RON loader registration
  - Startup: init_registry_handles (loads_folder via AssetServer)
  - init_resource::<R> (inserts default registry resource)
  - Update (run_if Loading, track_progress): seed_registry (one-shot, populates registry)
  - Update (#[cfg(hot-reload)]): propagate_registry (watches AssetEvent::Modified)
```

### Config Load Path (SeedableConfig)

1. `Startup`: `init_defaults_handle` calls `asset_server.load::<D>(D::asset_path())`, inserts `DefaultsHandle<D>`
2. `Update` while `AppState::Loading`: `seed_config` polls `Assets<D>`, converts to `Config` via `From`, inserts via `commands.insert_resource::<D::Config>()`
3. `iyes_progress` tracks `Progress { done: 0/1, total: 1 }` — app stays in `Loading` until all configs are done
4. Hot-reload: `propagate_defaults` watches `AssetEvent<D>::Modified`, re-inserts updated `Config`

### Registry Load Path (SeedableRegistry)

1. `Startup`: `init_registry_handles` calls `asset_server.load_folder(R::asset_dir())`, inserts `RegistryHandles<R::Asset>`
2. `init_resource::<R>()` inserts the empty registry (accessible immediately, before seeding)
3. `Update` while `AppState::Loading`: `seed_registry` polls `Assets<LoadedFolder>`, resolves typed handles, then polls each `Assets<R::Asset>` — when all handles are ready, calls `registry.seed(&collected)` and sets `Local<bool>`
4. `iyes_progress` tracks progress — app stays in `Loading` until all registries are done
5. Hot-reload: `propagate_registry` watches `AssetEvent<R::Asset>::Modified`, calls `registry.update_all()`

### Progress Gating

`AppState` transitions `Loading → Game` when ALL registered configs and registries report `Progress { done: 1, total: 1 }`. This is wired in `StatePlugin` via `ProgressPlugin::<AppState>::new().with_state_transition(AppState::Loading, AppState::Game)`.

---

## 3. GameConfig Derive Macro — Two Directions

### Reversed path (current convention for new types)

Used when the `Config` resource is the primary type (has gameplay logic):

```rust
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "CellDefaults",
    path = "config/defaults.cells.ron",
    ext = "cells.ron"
)]
pub(crate) struct CellConfig { ... }
```

The macro generates:
- `CellDefaults` struct with `#[derive(Asset, TypePath, Deserialize, Debug, Clone, PartialEq)]`
- `From<CellDefaults> for CellConfig`
- `From<CellConfig> for CellDefaults`
- `Default for CellDefaults` (delegates to `CellConfig::default()`)
- `CellConfig::merge_from_defaults(&mut self, defaults: &CellDefaults)`
- `impl SeedableConfig for CellDefaults` (because `path` and `ext` are present)

### Legacy path (older types)

Used when the Defaults struct was written first:
```rust
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "BreakerConfig")]
pub struct BreakerDefaults { ... }
```
Generates only the `Config` struct, no `SeedableConfig` impl (legacy types wire `SeedableConfig` manually or via a separate impl).

---

## 4. How Tuning Values Are Accessed at Runtime

Systems read from `Res<ConfigResource>` (the generated `Config` type). For example:

```rust
fn my_system(cell_config: Res<CellConfig>, ...) {
    let width = cell_config.width;
}
```

**Config resources are plain Bevy resources** — no special wrapper. They are inserted during `AppState::Loading` and live for the rest of the app lifetime. Hot-reload re-inserts them (replaces the resource) when the RON file changes on disk.

Registry-based catalogs are also plain `Resource` (e.g., `Res<BreakerRegistry>`, `Res<ChipTemplateRegistry>`). Systems look up entries by key at runtime.

`ChipCatalog` is accessed as `Res<ChipCatalog>`. It is `pub` (publicly exported from `breaker-game`) because the offering system and UI systems read it.

---

## 5. RON File Structure Convention

### Naming pattern

| Category | Folder | Extension | Example |
|----------|--------|-----------|---------|
| Single-value config | `assets/config/` | `defaults.<domain>.ron` | `defaults.cells.ron` |
| Breaker definitions | `assets/breakers/` | `<name>.breaker.ron` | `aegis.breaker.ron` |
| Bolt definitions | `assets/bolts/` | `<name>.bolt.ron` | `default.bolt.ron` |
| Chip templates | `assets/chips/standard/` | `<name>.chip.ron` | `amp.chip.ron` |
| Evolution templates | `assets/chips/evolutions/` | `<name>.evolution.ron` | `anchor.evolution.ron` |
| Cell type definitions | `assets/cells/` | (TODO: confirm extension) | — |
| Node layouts | (inside node layout asset dir) | — | — |
| Wall definitions | `assets/walls/` | `<name>.wall.ron` | `wall.wall.ron` |

### RON format

All RON files use the `IMPLICIT_SOME` extension enabled in `RonAssetLoader`:
```rust
ron::Options::default()
    .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
    .from_bytes(bytes)
```

This means `Option<T>` fields can be written as `value: 42` instead of `value: Some(42)`.

Config files carry a comment header identifying their struct type:
```ron
/* @[brickbreaker::cells::resources::CellDefaults] */
(
    width: 126.0,
    ...
)
```

### Folder-based vs single-file

- `assets/config/defaults.*.ron` — one file per config domain (SeedableConfig)
- All other asset dirs — one file per named entity (SeedableRegistry folder load)

### Example reference files

`assets/examples/*.example.ron` — documentation-only files listing all available fields with comments. They are NOT loaded by the game (not in a watched folder). Naming convention: `<domain>.example.ron`.

---

## 6. Wiring Location

All config and registry registrations happen in a single function in `StatePlugin`:

```
breaker-game/src/state/plugin/system.rs — defaults_plugin() fn
```

```rust
RantzDefaultsPluginBuilder::<AppState>::new(AppState::Loading)
    .add_config::<PlayfieldDefaults>()
    .add_config::<CellDefaults>()
    // ... all configs ...
    .add_registry::<CellTypeRegistry>()
    .add_registry::<BreakerRegistry>()
    .add_registry::<BoltRegistry>()
    .add_registry::<NodeLayoutRegistry>()
    .add_registry::<ChipTemplateRegistry>()
    .add_registry::<EvolutionTemplateRegistry>()
    .add_registry::<WallRegistry>()
    .build()
```

The `build_chip_catalog` system is wired separately in `LoadingPlugin` because it is a computed step:

```
breaker-game/src/state/app/loading/plugin.rs — LoadingPlugin::build()
```

---

## 7. Hot-Reload

Hot-reload is a `#[cfg(feature = "dev")]` / `#[cfg(feature = "hot-reload")]` concern.

**For configs** (`SeedableConfig`): `propagate_defaults::<D>` is added to `Update` in `DefaultsSystems::PropagateDefaults` when `hot-reload` feature is active. It watches `AssetEvent<D>::Modified` and re-inserts the `Config` resource.

**For registries** (`SeedableRegistry`): `propagate_registry::<R>` watches `AssetEvent<R::Asset>::Modified` and calls `registry.update_all()`.

**For ChipCatalog** (computed resource): `propagate_chip_catalog` is registered in `HotReloadPlugin` (`debug/hot_reload/plugin.rs`). It checks `is_changed()` on either source registry and rebuilds `ChipCatalog` from scratch. Runs only in `NodeState::Playing` (gated by `HotReloadSystems::PropagateDefaults` set).

---

## 8. Pattern for New ProtocolCatalog / HazardCatalog

Based on the existing patterns, here are the viable approaches:

### Option A: Pure SeedableRegistry (like BreakerRegistry)

**When to use**: Protocols/hazards have flat tuning values in RON, no computed expansion step needed.

Structure:
```
assets/protocols/<name>.protocol.ron   → ProtocolDefinition (Asset + Deserialize)
assets/hazards/<name>.hazard.ron       → HazardDefinition (Asset + Deserialize)
```

Registry:
```rust
#[derive(Resource, Debug, Default)]
pub struct ProtocolRegistry {
    protocols: HashMap<String, ProtocolDefinition>,
}
impl SeedableRegistry for ProtocolRegistry { ... }
```

Wiring: add to `defaults_plugin()` in `state/plugin/system.rs`:
```rust
.add_registry::<ProtocolRegistry>()
.add_registry::<HazardRegistry>()
```

Systems read `Res<ProtocolRegistry>` and `Res<HazardRegistry>` directly.

### Option B: Two-tier with computed Catalog (like ChipCatalog)

**When to use**: Protocol/hazard definitions need expansion or merging (e.g., multiple protocol pools combine into one catalog, or evolution-like computed fields).

Structure: raw registry → computed catalog (like `ChipTemplateRegistry` → `ChipCatalog`).

Requires: a `build_protocol_catalog` system registered in `LoadingPlugin` gated on `track_progress`.

### Recommendation for this design

Given that:
- Protocols are "one per run, RON-tuned but code-implemented"
- Hazards are "flat pool, RON-tuned, code-implemented systems"
- Neither requires rarity expansion or recipe building

**Option A** (pure `SeedableRegistry`) matches the existing `BreakerRegistry` pattern exactly. No computed catalog step is needed unless you add protocol meta-progression gating at load time.

If meta-progression unlock state needs to be filtered at offering time (not at load time), do that filtering in the offering system rather than in the catalog build step. Keep the registry as the raw complete pool.

**Naming convention** to match existing patterns:
- `ProtocolDefinition` (the `Asset` type, loaded from RON)
- `ProtocolRegistry` (the `Resource`, implements `SeedableRegistry`)
- `HazardDefinition` (the `Asset` type)
- `HazardRegistry` (the `Resource`, implements `SeedableRegistry`)

No separate `ProtocolCatalog` / `HazardCatalog` types are needed unless computation is required.

---

## 9. System Chain Summary

```
Startup
  init_registry_handles::<ChipTemplateRegistry>
    → inserts RegistryHandles<ChipTemplate> { folder: handle, loaded: false }
  init_registry_handles::<EvolutionTemplateRegistry>
    → inserts RegistryHandles<EvolutionTemplate> { folder: handle, loaded: false }

Update (AppState::Loading, track_progress)
  seed_registry::<ChipTemplateRegistry>
    → polls LoadedFolder, resolves handles, calls registry.seed()
    → sets RegistryHandles.loaded = true
  seed_registry::<EvolutionTemplateRegistry>
    → same pattern

  build_chip_catalog (chips domain, LoadingPlugin)
    reads: ChipTemplateRegistry, EvolutionTemplateRegistry, RegistryHandles (both)
    waits: until both RegistryHandles.loaded == true
    writes: commands.insert_resource(ChipCatalog)
    guard: Local<bool> — runs exactly once

Progress gating
  ProgressPlugin tracks all seed_* and build_chip_catalog Progress values
  AppState transitions Loading → Game only when all report done: 1

Runtime (Playing)
  Systems read Res<ChipCatalog>        — for chip offering, recipe checking
  Systems read Res<ChipTemplateRegistry> — for template-name lookup (offering dedup)
  Systems read Res<EvolutionTemplateRegistry> — (not directly; absorbed into ChipCatalog)

Hot-reload (dev only)
  propagate_registry::<ChipTemplateRegistry> (DefaultsSystems::PropagateDefaults)
  propagate_registry::<EvolutionTemplateRegistry>
  propagate_chip_catalog (HotReloadPlugin, NodeState::Playing only)
    → rebuilds ChipCatalog from updated registries
```

---

## Key Files

| File | Relevance |
|------|-----------|
| `rantzsoft_defaults/src/seedable.rs` | `SeedableConfig` trait definition |
| `rantzsoft_defaults/src/registry.rs` | `SeedableRegistry` trait + `RegistryHandles` type |
| `rantzsoft_defaults/src/loader.rs` | `RonAssetLoader<T>` — generic RON deserialization |
| `rantzsoft_defaults/src/systems/fns.rs` | `seed_config`, `seed_registry`, `propagate_defaults`, `propagate_registry`, `init_*_handles` |
| `rantzsoft_defaults/src/plugin/definition.rs` | `RantzDefaultsPluginBuilder` — where `.add_config` and `.add_registry` are defined |
| `rantzsoft_defaults_derive/src/lib.rs` | `GameConfig` proc macro — generates paired Config/Defaults structs and `SeedableConfig` impl |
| `breaker-game/src/chips/resources/data.rs` | `ChipCatalog`, `ChipTemplateRegistry`, `EvolutionTemplateRegistry` definitions |
| `breaker-game/src/chips/definition/types.rs` | `ChipTemplate`, `EvolutionTemplate`, `ChipDefinition`, `expand_chip_template`, `expand_evolution_template` |
| `breaker-game/src/chips/systems/build_chip_catalog/system.rs` | `build_chip_catalog` system — two-registry → computed catalog pattern |
| `breaker-game/src/state/plugin/system.rs` | `defaults_plugin()` — the single wiring location for all configs and registries |
| `breaker-game/src/state/app/loading/plugin.rs` | `LoadingPlugin` — where `build_chip_catalog` is wired with `track_progress` |
| `breaker-game/src/debug/hot_reload/plugin.rs` | `HotReloadPlugin` — where `propagate_chip_catalog` is registered |
| `breaker-game/src/breaker/registry.rs` | `BreakerRegistry` — simplest `SeedableRegistry` example (no computed step) |
| `breaker-game/src/cells/resources/data.rs` | `CellConfig` + `CellTypeRegistry` — shows both patterns in one file |
| `breaker-game/src/shared/playfield.rs` | `PlayfieldConfig` — canonical `GameConfig` reversed-path example |
| `breaker-game/assets/chips/standard/amp.chip.ron` | Canonical multi-rarity chip template RON |
| `breaker-game/assets/chips/evolutions/anchor.evolution.ron` | Canonical evolution RON with ingredients |
| `breaker-game/assets/config/defaults.cells.ron` | Canonical single-file config RON |
| `breaker-game/assets/examples/*.example.ron` | Documentation-only reference files (not loaded) |
