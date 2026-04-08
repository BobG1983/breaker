# Registry and Definition Struct Patterns

Exact code collected from the Bevy 0.18 codebase. No analysis — patterns only.

---

## 1. BreakerRegistry — SeedableRegistry Example

### Struct definition

`breaker-game/src/breaker/registry.rs`

```rust
/// Registry of all loaded breaker definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub struct BreakerRegistry {
    /// Map from breaker name to its definition.
    breakers: HashMap<String, BreakerDefinition>,
}
```

### SeedableRegistry trait impl (all methods)

`breaker-game/src/breaker/registry.rs`

```rust
impl SeedableRegistry for BreakerRegistry {
    type Asset = BreakerDefinition;

    fn asset_dir() -> &'static str {
        "breakers"
    }

    fn extensions() -> &'static [&'static str] {
        &["breaker.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<BreakerDefinition>, BreakerDefinition)]) {
        self.breakers.clear();
        for (_id, def) in assets {
            if self.breakers.contains_key(&def.name) {
                warn!("duplicate breaker name '{}' — skipping", def.name);
                continue;
            }
            self.breakers.insert(def.name.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<BreakerDefinition>, asset: &BreakerDefinition) {
        self.breakers.insert(asset.name.clone(), asset.clone());
    }
}
```

Note: `update_all` is provided by the trait (resets to `Default` then calls `seed`).

### BreakerDefinition asset type (full struct)

`breaker-game/src/breaker/definition/types.rs`

```rust
/// A breaker definition loaded from a RON file.
///
/// All gameplay fields have serde defaults; RON files only need to specify `name`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BreakerDefinition {
    /// Display name of the breaker.
    pub name: String,
    /// Name of the bolt definition this breaker uses.
    #[serde(default = "default_bolt_name")]
    pub bolt: String,
    /// Number of lives, if the breaker uses a life pool. None = infinite.
    #[serde(default)]
    pub life_pool: Option<u32>,
    /// All effect chains for this breaker.
    #[serde(default)]
    pub effects: Vec<RootEffect>,

    // ── Dimensions ──────────────────────────────────────────────────────
    #[serde(default = "default_width")]
    pub width: f32,
    #[serde(default = "default_height")]
    pub height: f32,
    #[serde(default = "default_y_position")]
    pub y_position: f32,
    #[serde(default)]
    pub min_w: Option<f32>,
    #[serde(default)]
    pub max_w: Option<f32>,
    #[serde(default)]
    pub min_h: Option<f32>,
    #[serde(default)]
    pub max_h: Option<f32>,

    // ── Movement ────────────────────────────────────────────────────────
    #[serde(default = "default_max_speed")]
    pub max_speed: f32,
    #[serde(default = "default_acceleration")]
    pub acceleration: f32,
    #[serde(default = "default_deceleration")]
    pub deceleration: f32,
    #[serde(default = "default_decel_ease")]
    pub decel_ease: EaseFunction,
    #[serde(default = "default_decel_ease_strength")]
    pub decel_ease_strength: f32,

    // ── Dash ────────────────────────────────────────────────────────────
    #[serde(default = "default_dash_speed_multiplier")]
    pub dash_speed_multiplier: f32,
    #[serde(default = "default_dash_duration")]
    pub dash_duration: f32,
    #[serde(default = "default_dash_tilt_angle")]
    pub dash_tilt_angle: f32,
    #[serde(default = "default_dash_tilt_ease")]
    pub dash_tilt_ease: EaseFunction,
    #[serde(default = "default_brake_tilt_angle")]
    pub brake_tilt_angle: f32,
    #[serde(default = "default_brake_tilt_duration")]
    pub brake_tilt_duration: f32,
    #[serde(default = "default_brake_tilt_ease")]
    pub brake_tilt_ease: EaseFunction,
    #[serde(default = "default_brake_decel_multiplier")]
    pub brake_decel_multiplier: f32,
    #[serde(default = "default_settle_duration")]
    pub settle_duration: f32,
    #[serde(default = "default_settle_tilt_ease")]
    pub settle_tilt_ease: EaseFunction,

    // ── Bump ────────────────────────────────────────────────────────────
    #[serde(default = "default_perfect_window")]
    pub perfect_window: f32,
    #[serde(default = "default_early_window")]
    pub early_window: f32,
    #[serde(default = "default_late_window")]
    pub late_window: f32,
    #[serde(default = "default_perfect_bump_cooldown")]
    pub perfect_bump_cooldown: f32,
    #[serde(default = "default_weak_bump_cooldown")]
    pub weak_bump_cooldown: f32,
    #[serde(default = "default_bump_visual_duration")]
    pub bump_visual_duration: f32,
    #[serde(default = "default_bump_visual_peak")]
    pub bump_visual_peak: f32,
    #[serde(default = "default_bump_visual_peak_fraction")]
    pub bump_visual_peak_fraction: f32,
    #[serde(default = "default_bump_visual_rise_ease")]
    pub bump_visual_rise_ease: EaseFunction,
    #[serde(default = "default_bump_visual_fall_ease")]
    pub bump_visual_fall_ease: EaseFunction,

    // ── Spread ──────────────────────────────────────────────────────────
    #[serde(default = "default_reflection_spread")]
    pub reflection_spread: f32,

    // ── Visual ──────────────────────────────────────────────────────────
    #[serde(default = "default_color_rgb")]
    pub color_rgb: [f32; 3],
}
```

Key: derives `Asset, TypePath, Deserialize, Clone, Debug`. No `Resource` — this is the asset, not the registry.
The file also implements `Default for BreakerDefinition` by wiring each field to its serde-default function.

### Example .breaker.ron file

`breaker-game/assets/breakers/aegis.breaker.ron`

```ron
/* @[brickbreaker::breaker::definition::BreakerDefinition] */
(
    name: "Aegis",
    life_pool: Some(3),
    effects: [
        On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(LoseLife)])]),
        On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
        On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
    ],
)
```

Only `name` is required. All other fields fall back to serde defaults (e.g. `bolt: "Bolt"`, `width: 120.0`).

### System reading BreakerRegistry

`breaker-game/src/state/run/systems/setup_run/system.rs`

```rust
pub(crate) fn setup_run(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    breaker_reg: Res<BreakerRegistry>,
    bolt_reg: Res<BoltRegistry>,
    run_state: Res<NodeOutcome>,
    mut rng: ResMut<GameRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_breakers: Query<(), With<Breaker>>,
    mut breaker_spawned: MessageWriter<BreakerSpawned>,
    mut bolt_spawned: MessageWriter<BoltSpawned>,
) {
    // ...
    let Some(breaker_def) = breaker_reg.get(&selected.0).cloned() else {
        warn!("Breaker '{}' not found in BreakerRegistry", selected.0);
        return;
    };

    Breaker::builder()
        .definition(&breaker_def)
        .rendered(&mut meshes, &mut materials)
        .primary()
        .spawn(&mut commands);
    // ...
}
```

The system takes `Res<BreakerRegistry>` directly. Lookup is `registry.get(name) -> Option<&BreakerDefinition>`.

---

## 2. CellTypeRegistry — Another SeedableRegistry

### Struct definition and impl

`breaker-game/src/cells/resources/data.rs`

```rust
/// Registry mapping alias strings to cell type definitions. Built at boot from all loaded RONs.
#[derive(Resource, Debug, Default)]
pub(crate) struct CellTypeRegistry {
    /// Map from alias string to cell type definition.
    types: HashMap<String, CellTypeDefinition>,
}

impl SeedableRegistry for CellTypeRegistry {
    type Asset = CellTypeDefinition;

    fn asset_dir() -> &'static str {
        "cells"
    }

    fn extensions() -> &'static [&'static str] {
        &["cell.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<CellTypeDefinition>, CellTypeDefinition)]) {
        self.types.clear();
        for (_id, def) in assets {
            // Reserved alias is a programming error — panic before validate() filters it.
            assert!(def.alias != ".", "reserved alias '{}'", def.alias);
            if def.validate().is_err() {
                warn!(
                    "skipping cell type '{}' (alias '{}'): validation failed",
                    def.id, def.alias
                );
                continue;
            }
            assert!(
                !self.types.contains_key(&def.alias),
                "duplicate cell type alias '{}'",
                def.alias
            );
            self.types.insert(def.alias.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<CellTypeDefinition>, asset: &CellTypeDefinition) {
        assert!(asset.alias != ".", "reserved alias '{}'", asset.alias);
        if asset.validate().is_err() {
            warn!(
                "ignoring invalid cell type update '{}' (alias '{}')",
                asset.id, asset.alias
            );
            return;
        }
        self.types.insert(asset.alias.clone(), asset.clone());
    }
}
```

Key difference from BreakerRegistry: CellTypeRegistry keys on `alias` (the grid token), not `name`. It also calls `def.validate()` before inserting and skips rather than panics on invalid data. It panics on the reserved `"."` alias (which is the empty-cell sentinel) because that is always a programming error.

### CellTypeDefinition asset type

`breaker-game/src/cells/definition.rs`

```rust
/// Behavioral variants that can be attached to a cell type.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum CellBehavior {
    /// Cell regenerates HP at the given rate per second.
    Regen { rate: f32 },
}

/// Configuration for a shield cell's orbiting children.
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ShieldBehavior {
    pub count: u32,
    pub radius: f32,
    pub speed: f32,
    pub hp: f32,
    pub color_rgb: [f32; 3],
}

/// A cell type definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct CellTypeDefinition {
    /// Unique identifier.
    pub id: String,
    /// Alias used in node layout grids — may be multi-character.
    pub alias: String,
    /// Hit points for this cell type.
    pub hp: f32,
    /// HDR RGB color.
    pub color_rgb: [f32; 3],
    /// Whether this cell counts toward node completion.
    pub required_to_clear: bool,
    /// HDR intensity multiplier for damaged cells at full health.
    pub damage_hdr_base: f32,
    /// Minimum green channel value for damage color feedback.
    pub damage_green_min: f32,
    /// Blue channel range added based on health fraction.
    pub damage_blue_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base: f32,
    /// Optional behavior list (regen, etc.). Defaults to `None`.
    #[serde(default)]
    pub behaviors: Option<Vec<CellBehavior>>,
    /// Optional shield configuration.
    #[serde(default)]
    pub shield: Option<ShieldBehavior>,
    /// Optional effect chains for this cell type. Defaults to `None`.
    #[serde(default)]
    pub effects: Option<Vec<RootEffect>>,
}
```

Key: derives `Asset, TypePath, Deserialize, Clone, Debug` (same pattern as `BreakerDefinition`). No serde-default functions for required fields — those must appear in every RON file. Optional fields use `#[serde(default)]`.

### Example .cell.ron files

`breaker-game/assets/cells/standard.cell.ron`
```ron
/* @[brickbreaker::cells::resources::CellTypeDefinition] */
(
    id: "standard",
    alias: "S",
    hp: 10.0,
    color_rgb: (4.0, 0.2, 0.5),
    required_to_clear: true,
    damage_hdr_base: 4.0,
    damage_green_min: 0.2,
    damage_blue_range: 0.4,
    damage_blue_base: 0.2,
)
```

`breaker-game/assets/cells/regen.cell.ron`
```ron
(
    id: "regen",
    alias: "R",
    hp: 20.0,
    color_rgb: (0.3, 4.0, 0.3),
    required_to_clear: true,
    damage_hdr_base: 4.0,
    damage_green_min: 0.4,
    damage_blue_range: 0.3,
    damage_blue_base: 0.1,
    behaviors: Some([Regen(rate: 2.0)]),
)
```

Note: `color_rgb` uses tuple syntax `(r, g, b)` rather than array syntax `[r, g, b]` — RON treats both as valid for a `[f32; 3]`.

---

## 3. ChipDefinition — the runtime type

### Full struct with all fields

`breaker-game/src/chips/definition/types.rs`

```rust
/// A fully resolved chip definition used at runtime.
///
/// Never deserialized directly — constructed from [`ChipTemplate`] via
/// [`expand_chip_template`] or from [`EvolutionTemplate`] via [`expand_evolution_template`].
#[derive(Clone, Debug)]
pub struct ChipDefinition {
    /// Display name shown on the chip card.
    pub name: String,
    /// Flavor text shown below the name.
    pub description: String,
    /// How rare this chip is.
    pub rarity: Rarity,
    /// Maximum number of times this chip can be stacked.
    pub max_stacks: u32,
    /// The effects applied when this chip is selected.
    pub effects: Vec<crate::effect::RootEffect>,
    /// Evolution ingredients. `None` for non-evolution chips.
    pub ingredients: Option<Vec<EvolutionIngredient>>,
    /// Template this chip was expanded from, if any.
    pub template_name: Option<String>,
}
```

Key: derives only `Clone, Debug`. No `Asset`, no `Deserialize`, no `Resource`. This is a pure runtime type constructed by `expand_chip_template` / `expand_evolution_template` at catalog-build time.

### Rarity enum

```rust
/// How rare a chip is — controls appearance weight in the selection pool.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
    /// Evolution-tier chips — produced by combining maxed ingredient chips.
    Evolution,
}
```

### ChipInventory — how it tracks taken chips

`breaker-game/src/chips/inventory/data.rs`

```rust
/// A single entry in the chip inventory, tracking stacks and metadata.
#[derive(Debug, Clone)]
pub struct ChipEntry {
    pub stacks: u32,
    pub max_stacks: u32,
    pub rarity: Rarity,
    pub template_name: Option<String>,
}

/// Tracks chips the player has acquired during a run.
#[derive(Resource, Debug, Default)]
pub struct ChipInventory {
    /// Map from chip display name to its entry (stacks, max, rarity, template).
    held: HashMap<String, ChipEntry>,
    /// Accumulated offering decay per chip name (multiplied by 0.8 each time offered).
    decay_weights: HashMap<String, f32>,
    /// Current count of chips taken per template name.
    template_taken: HashMap<String, u32>,
    /// Maximum allowed per template name.
    template_maxes: HashMap<String, u32>,
}
```

Key structure: `held` is `HashMap<String, ChipEntry>` where the key is the chip's display name (e.g. `"Minor Splinter"`). `template_taken` uses the template base name as key (e.g. `"Splinter"`), not the expanded name. The inventory enforces two distinct caps: per-chip-name via `held[name].stacks >= held[name].max_stacks`, and per-template via `template_taken[tname] >= def.max_stacks`.

### The loaded template type (what gets deserialized from .chip.ron)

```rust
/// A chip template loaded from RON (`.chip.ron`).
///
/// Each template defines up to four rarity variants. At load time,
/// [`expand_chip_template`] converts each non-`None` slot into a [`ChipDefinition`].
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ChipTemplate {
    pub name: String,
    pub max_taken: u32,
    #[serde(default)]
    pub common: Option<RaritySlot>,
    #[serde(default)]
    pub uncommon: Option<RaritySlot>,
    #[serde(default)]
    pub rare: Option<RaritySlot>,
    #[serde(default)]
    pub legendary: Option<RaritySlot>,
}

/// A rarity slot within a [`ChipTemplate`].
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct RaritySlot {
    pub prefix: String,
    pub effects: Vec<crate::effect::RootEffect>,
}
```

Example `.chip.ron` — `breaker-game/assets/chips/standard/splinter.chip.ron`:
```ron
(
    name: "Splinter",
    max_taken: 2,
    common: (
        prefix: "Minor",
        effects: [
            On(target: Bolt, then: [
                When(trigger: CellDestroyed, then: [
                    Do(SpawnBolts(count: 1, lifespan: Some(2.0), inherit: false)),
                    Do(SizeBoost(0.5)),
                ]),
            ]),
        ],
    ),
    uncommon: (
        prefix: "Spreading",
        effects: [ ... ],
    ),
    rare: (
        prefix: "Devastating",
        effects: [ ... ],
    ),
)
```

Example `.evolution.ron` — `breaker-game/assets/chips/evolutions/anchor.evolution.ron`:
```ron
(
    name: "Anchor",
    description: "Plant the breaker for doubled bump force and a wider perfect window",
    effects: [
        On(target: Breaker, then: [
            Do(Anchor(
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            )),
        ]),
    ],
    ingredients: [
        (chip_name: "Quick Stop", stacks_required: 2),
        (chip_name: "Bump Force", stacks_required: 2),
    ],
)
```

Note: `ingredients` uses template-level names (the base `name` field in a `.chip.ron`), not expanded names.

### The ChipTemplateRegistry (the SeedableRegistry for chips)

`breaker-game/src/chips/resources/data.rs`

```rust
/// Registry of chip templates loaded from `.chip.ron` files.
#[derive(Resource, Debug, Default)]
pub(crate) struct ChipTemplateRegistry {
    templates: HashMap<String, (AssetId<ChipTemplate>, ChipTemplate)>,
}

impl SeedableRegistry for ChipTemplateRegistry {
    type Asset = ChipTemplate;

    fn asset_dir() -> &'static str {
        "chips/standard"
    }

    fn extensions() -> &'static [&'static str] {
        &["chip.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<ChipTemplate>, ChipTemplate)]) {
        self.templates.clear();
        for (id, template) in assets {
            self.templates
                .insert(template.name.clone(), (*id, template.clone()));
        }
    }

    fn update_single(&mut self, id: AssetId<ChipTemplate>, asset: &ChipTemplate) {
        self.templates
            .insert(asset.name.clone(), (id, asset.clone()));
    }
}
```

Key difference: the map value is `(AssetId<ChipTemplate>, ChipTemplate)` — it stores the `AssetId` alongside the data, unlike `BreakerRegistry` and `CellTypeRegistry` which discard the id. This enables `update_single` to avoid re-inserting under the same id in a hot-reload scenario.

---

## 4. SeedableRegistry trait — full definition

`rantzsoft_defaults/src/registry.rs`

```rust
/// A [`Resource`] populated from a folder of RON assets at boot time.
pub trait SeedableRegistry: Resource + Default + Send + Sync + 'static {
    /// The asset type loaded from each RON file in the registry folder.
    type Asset: Asset + DeserializeOwned + Clone + Send + Sync + 'static;

    /// Path to the folder containing registry assets (relative to `assets/`).
    fn asset_dir() -> &'static str;

    /// File extensions recognized for this asset type.
    fn extensions() -> &'static [&'static str];

    /// Populate the registry from loaded assets. Destructive — replaces all
    /// existing entries.
    fn seed(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)]);

    /// Rebuild the registry from all assets. Default: reset to default then
    /// seed.
    fn update_all(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)]) {
        *self = Self::default();
        self.seed(assets);
    }

    /// Update a single asset entry. Required — implementor defines upsert logic.
    fn update_single(&mut self, id: AssetId<Self::Asset>, asset: &Self::Asset);
}
```

### RegistryHandles struct

```rust
/// Stores the folder handle and typed handles for a registry's assets.
#[derive(Resource)]
pub struct RegistryHandles<A: Asset> {
    pub folder: Handle<LoadedFolder>,
    pub handles: Vec<Handle<A>>,
    pub loaded: bool,
}

impl<A: Asset> RegistryHandles<A> {
    #[must_use]
    pub const fn new(folder: Handle<LoadedFolder>) -> Self {
        Self {
            folder,
            handles: Vec::new(),
            loaded: false,
        }
    }
}
```

`RegistryHandles<A>` is inserted at `Startup` by `init_registry_handles`. The `seed_registry` system resolves the folder, populates `handles`, and calls `SeedableRegistry::seed`. It is generic — one `RegistryHandles<BreakerDefinition>`, one `RegistryHandles<CellTypeDefinition>`, etc. all coexist as separate resources.

### How registries are wired into the app

`rantzsoft_defaults/src/plugin/definition.rs` — inside `RantzDefaultsPluginBuilder::add_registry`:

```rust
pub fn add_registry<R: SeedableRegistry + 'static>(mut self) -> Self
where
    R::Asset: serde::de::DeserializeOwned,
{
    let loading_state = self.loading_state.clone();
    self.registrations.push(Box::new(move |app: &mut App| {
        app.init_asset::<R::Asset>();
        app.register_asset_loader(RonAssetLoader::<R::Asset>::new(R::extensions()));
        app.add_systems(Startup, init_registry_handles::<R>);
        app.init_resource::<R>();
        // (feature-gated: seed_registry runs in Update while in loading state)
        // (feature-gated: propagate_registry runs in Update for hot-reload)
    }));
    self
}
```

The seed system (`seed_registry`) runs in `Update` gated on `in_state(loading_state)` — not at `Startup`. It waits for the asset folder to finish loading.

---

## 5. GameConfig derive macro — CellConfig example

### The Config struct (reversed path)

`breaker-game/src/cells/resources/data.rs`

```rust
/// Cell configuration resource — shared grid layout properties.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "CellDefaults",
    path = "config/defaults.cells.ron",
    ext = "cells.ron"
)]
pub(crate) struct CellConfig {
    pub width: f32,
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
}

impl Default for CellConfig {
    fn default() -> Self {
        Self {
            width: 70.0,
            height: 24.0,
            padding_x: 4.0,
            padding_y: 4.0,
        }
    }
}
```

### What the macro generates

From `rantzsoft_defaults_derive/src/lib.rs` (reversed path, `has_seedable = true`):

```rust
// 1. The Defaults asset struct (with Asset + Deserialize derives, matching visibility)
#[derive(Asset, TypePath, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CellDefaults {
    pub width: f32,
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
}

// 2. From<CellDefaults> for CellConfig
impl From<CellDefaults> for CellConfig {
    fn from(d: CellDefaults) -> Self {
        Self { width: d.width, height: d.height, padding_x: d.padding_x, padding_y: d.padding_y }
    }
}

// 3. From<CellConfig> for CellDefaults
impl From<CellConfig> for CellDefaults {
    fn from(c: CellConfig) -> Self {
        Self { width: c.width, height: c.height, padding_x: c.padding_x, padding_y: c.padding_y }
    }
}

// 4. Default for CellDefaults (delegates to CellConfig::default())
impl Default for CellDefaults {
    fn default() -> Self {
        CellConfig::default().into()
    }
}

// 5. merge_from_defaults method on CellConfig
impl CellConfig {
    pub fn merge_from_defaults(&mut self, defaults: &CellDefaults) {
        self.width = defaults.width.clone();
        self.height = defaults.height.clone();
        self.padding_x = defaults.padding_x.clone();
        self.padding_y = defaults.padding_y.clone();
    }
}

// 6. SeedableConfig impl for CellDefaults (because path + ext are provided)
impl SeedableConfig for CellDefaults {
    type Config = CellConfig;

    fn asset_path() -> &'static str {
        "config/defaults.cells.ron"
    }

    fn extensions() -> &'static [&'static str] {
        &["cells.ron"]
    }
}
```

The derive always produces a paired struct. When `path` and `ext` are provided, it also produces the `SeedableConfig` impl so the pipeline can load the RON file and seed the `Resource`.

### The RON file

`breaker-game/assets/config/defaults.cells.ron`

```ron
/* @[brickbreaker::cells::resources::CellDefaults] */
(
    width: 126.0,
    height: 43.0,
    padding_x: 7.0,
    padding_y: 7.0,
)
```

### SeedableConfig trait (the single-file version — contrast with SeedableRegistry)

`rantzsoft_defaults/src/seedable.rs`

```rust
pub trait SeedableConfig: Asset + Clone + Send + Sync + 'static {
    /// The `Resource` type seeded from this defaults asset.
    type Config: Resource + From<Self> + Send + Sync + 'static;

    fn asset_path() -> &'static str;

    fn extensions() -> &'static [&'static str];
}
```

`SeedableConfig` loads a single file. `SeedableRegistry` loads an entire folder. Both are registered through `RantzDefaultsPluginBuilder` (`.add_config::<D>()` vs `.add_registry::<R>()`).

---

## Pattern Summary

| Type | Derives | Lives in | Asset? | Key field |
|---|---|---|---|---|
| `BreakerDefinition` | `Asset, TypePath, Deserialize, Clone, Debug` | `breaker/definition/types.rs` | yes | `name: String` (also registry key) |
| `CellTypeDefinition` | `Asset, TypePath, Deserialize, Clone, Debug` | `cells/definition.rs` | yes | `alias: String` (registry key), `id: String` (display) |
| `ChipTemplate` | `Asset, TypePath, Deserialize, Clone, Debug` | `chips/definition/types.rs` | yes | `name: String` (registry key) |
| `ChipDefinition` | `Clone, Debug` | `chips/definition/types.rs` | no | runtime-only, never deserialized |
| `BreakerRegistry` | `Resource, Debug, Default` | `breaker/registry.rs` | no | `breakers: HashMap<String, BreakerDefinition>` |
| `CellTypeRegistry` | `Resource, Debug, Default` | `cells/resources/data.rs` | no | `types: HashMap<String, CellTypeDefinition>` |
| `ChipTemplateRegistry` | `Resource, Debug, Default` | `chips/resources/data.rs` | no | `templates: HashMap<String, (AssetId, ChipTemplate)>` |
| `CellConfig` | `Resource, Debug, Clone, PartialEq, GameConfig` | `cells/resources/data.rs` | no | generated `CellDefaults` is the asset |
| `ChipInventory` | `Resource, Debug, Default` | `chips/inventory/data.rs` | no | `held: HashMap<String, ChipEntry>` |
