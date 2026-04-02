# Data Model — Components vs Resources

How game data flows from RON config files to runtime systems via the ECS.

---

## Pipeline: RON → Registry → Builder → Entity Component

```
assets/breakers/*.breaker.ron
        ↓  (RonAssetLoader via SeedableRegistry)
Res<BreakerRegistry>
        ↓  (spawn_or_reuse_breaker — runs OnEnter(Playing))
        ↓  BreakerRegistry.get(name) → &BreakerDefinition
        ↓  Breaker::builder().definition(&def).rendered(...).primary().spawn(commands)
Entity components: BaseWidth, BaseHeight, BreakerReflectionSpread, …
        ↓  (production systems query entities)
move_breaker, bolt_breaker_collision, …
```

**Registries** (e.g., `Res<BreakerRegistry>`, `Res<BoltRegistry>`, `Res<CellTypeRegistry>`) are the bridge between RON data files and builders. The builder reads from a definition struct and emits all entity components in a single `build()` call.

**`BreakerConfig` was eliminated.** All per-breaker gameplay fields moved into `BreakerDefinition` with `#[serde(default)]`. The RON files specify only overrides from the defaults. `BoltConfig` was similarly eliminated — all fields moved into `BoltDefinition`.

**Only builders read definition structs.** Every other system reads entity components directly. This keeps production systems decoupled from the data pipeline.

---

## Rules

### 1. Components on the owning entity

A component belongs on the entity it conceptually describes:

- `BreakerReflectionSpread` is a breaker surface property → lives on the **breaker** entity, even though `bolt_breaker_collision` also reads it
- `BoltRadius` is a bolt property → lives on the **bolt** entity, even though collision systems on other domains read it
- Cross-entity queries are normal ECS — reading a component from another entity is not coupling

### 2. Config fields follow the same ownership

If a value belongs to a domain, its config field and RON entry live in that domain's `*Defaults`:

- Breaker surface angles (`reflection_spread`), movement, dash, bump params → `BreakerDefinition` / `assets/breakers/*.breaker.ron`
- Bolt speed/radius → `BoltDefinition` / `assets/bolts/*.bolt.ron`

Don't leave empty config resources — if all fields move elsewhere, delete the resource.

### 3. Store full dimensions, provide half helpers

Config files and components store the **full, intuitive value** (width, height). Systems that need halves call helper methods:

```rust
#[derive(Component, Debug)]
pub struct BaseWidth(pub f32);

impl BaseWidth {
    pub fn half_width(&self) -> f32 {
        self.0 / 2.0
    }
}
```

This keeps RON files readable (`width: 120.0` not `half_width: 60.0`) and makes collision code explicit about the division.

### 4. Struct components for tightly coupled fields

When multiple values are **always accessed together** in the same systems, group them into a single struct component:

```rust
#[derive(Component, Debug, Clone)]
pub struct BumpFeedback {
    pub duration: f32,
    pub peak: f32,
    pub peak_fraction: f32,
    pub rise_ease: EaseFunction,
    pub fall_ease: EaseFunction,
}
```

When values are **independently accessed** across different systems, keep them as separate newtypes (`BoltBaseSpeed(f32)`, `BoltRadius(f32)`).

### 5. Builders materialize components from definitions

The builder pattern replaces the old `init_*_params` systems. The `spawn_or_reuse_breaker` system reads from the registry, builds the entity with all components at once, and only does so for new (not persisted) entities:

```rust
pub fn spawn_or_reuse_breaker(
    query: Query<(), With<Breaker>>,
    registry: Res<BreakerRegistry>,
    selected: Res<SelectedBreaker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if query.is_empty() {
        // No existing breaker — spawn fresh
        let def = registry.get(&selected.name).unwrap();
        Breaker::builder()
            .definition(def)
            .rendered(&mut meshes, &mut materials)
            .primary()
            .spawn(&mut commands);
    }
    // Otherwise reuse the existing entity (persisted across nodes via CleanupOnRunEnd)
}
```

All 22+ stat components are now produced by `build()` in a single call. The `Without<MaxSpeed>` guard pattern is no longer needed.

---

## Active Component Pattern

Stat-modifying effects use a single-tier `Active*` stack model — consumers read `Active*` directly via accessor methods:

```
fire_effect(entity, DamageBoost(2.0))
        ↓  (push onto Active stack)
ActiveDamageBoosts(vec![2.0])
        ↓  (consumers call .multiplier() inline)
bolt_cell_collision: effective_damage = BASE_BOLT_DAMAGE * active.multiplier()
```

**Rules:**

- **`Active*` components** (e.g., `ActiveDamageBoosts`, `ActiveSpeedBoosts`, `ActivePiercings`) live in the effect domain (`effect/effects/<name>.rs`). They are plain `Vec` stacks — each applied effect instance pushes one entry; `reverse_effect` removes it.
- **Consumers** (bolt collision, move_breaker, etc.) read `Active*` directly using the `.multiplier()` method (product of all entries, default 1.0) or `.total()` (sum of all entries, for additive stats like piercing). No separate cache component is computed.
- **`PiercingRemaining`** is bolt gameplay state (lives in the bolt domain), not an effect stat. `ActivePiercings::total()` gives the cap that `PiercingRemaining` resets to on wall/breaker contact.
- `Active*` components are inserted lazily by `fire()` when first needed. Consumers handle the absent case via `Option<&Active*>` and map to the identity value (1.0 for multipliers, 0 for sums).

---

## Testing

- **Builder tests** use `Breaker::builder().definition(&def).headless().primary().build()` — they test the definition→component bridge
- **Production system tests** spawn entities with component values directly — no registry or definition needed
- Tests that need a breaker entity use `Breaker::builder()` directly with test values, not the registry

---

## Registry Pattern

Domain registries hold definitions loaded from RON during the loading screen. Registries that load a **folder** of RON assets implement the `SeedableRegistry` trait from `rantzsoft_defaults`. Registries that are built from templates at runtime (e.g., `ChipCatalog`) use a custom internal pattern.

### SeedableRegistry (folder-based loading)

Registries that load an entire directory of RON files implement `SeedableRegistry` from `rantzsoft_defaults::prelude`. The `RantzDefaultsPluginBuilder::add_registry::<R>()` call wires all loading, seeding, and (with `hot-reload` feature) hot-reload propagation automatically.

```rust
#[derive(Resource, Debug, Default)]
pub struct BreakerRegistry {
    breakers: HashMap<String, BreakerDefinition>,
}

impl SeedableRegistry for BreakerRegistry {
    type Asset = BreakerDefinition;

    fn asset_dir() -> &'static str { "breakers" }
    fn extensions() -> &'static [&'static str] { &["breaker.ron"] }

    fn seed(&mut self, assets: &[(AssetId<BreakerDefinition>, BreakerDefinition)]) {
        self.breakers.clear();
        for (_id, def) in assets { self.breakers.insert(def.name.clone(), def.clone()); }
    }

    fn update_single(&mut self, _id: AssetId<BreakerDefinition>, asset: &BreakerDefinition) {
        self.breakers.insert(asset.name.clone(), asset.clone());
    }
}
```

The `update_all` method is provided by the trait (resets to default then calls `seed`). The `RegistryHandles<A>` resource tracks the folder handle and typed asset handles; the `seed_registry` system resolves the folder and seeds the registry during the loading state.

Fields are **private** — all access goes through methods. This lets internals change (e.g., adding ordering) without breaking callers.

### Key Types

| Registry | Asset type | Key | Notes |
|----------|-----------|-----|-------|
| `BreakerRegistry` | `BreakerDefinition` (`breaker.ron`) | `String` (name) | Implements `SeedableRegistry`. Folder: `assets/breakers/`. Re-exported from `breaker/`. |
| `ChipTemplateRegistry` | `ChipTemplate` (`chip.ron`) | `String` (name) | Implements `SeedableRegistry`. Folder: `assets/chips/standard/`. Stores `(AssetId, ChipTemplate)` pairs for hot-reload. |
| `EvolutionTemplateRegistry` | `EvolutionTemplate` (`evolution.ron`) | `String` (name) | Implements `SeedableRegistry`. Folder: `assets/chips/evolutions/`. Stores `(AssetId, EvolutionTemplate)` pairs. |
| `ChipCatalog` | *(built from templates)* | `String` (name) | NOT a `SeedableRegistry` — built at runtime by expanding `ChipTemplate`s and `EvolutionTemplate`s via `populate_catalog`. Paired `Vec<String>` preserves insertion order for deterministic chip offers. Also holds `Vec<Recipe>` for in-catalog evolution recipes. |
| `NodeLayoutRegistry` | `NodeLayout` | `String` (name) | Paired `Vec<String>` preserves insertion order for index-based node progression. |
| `CellTypeRegistry` | `CellTypeDefinition` | `char` (alias) | Exception: keyed by grid alias char, not name. `CellTypeDefinition.hp` is `f32`. Has optional `behavior: CellBehavior` field (locked, regen_rate). |

### Pipeline

For `SeedableRegistry` types:

```
assets/<dir>/*.ron
    ↓  (RonAssetLoader — registered by add_registry())
Assets<FooDefinition>
    ↓  (seed_registry — runs in loading state via add_registry())
Res<FooRegistry>
    ↓  (production systems read via methods)
    ↓  (propagate_registry — reruns on AssetEvent::Modified when hot-reload feature active)
```

For `ChipCatalog` (template-expanded):

```
assets/chips/standard/*.chip.ron
    ↓  (ChipTemplateRegistry seeded via SeedableRegistry)
Res<ChipTemplateRegistry>
    ↓  (expand_template — called at load time to expand each template into ChipDefinitions)
Res<ChipCatalog>
    ↓  (production systems read via ordered_values(), get(), eligible_recipes())
```

### When to Add Ordered Access

If a registry needs both name lookup and index-based access, pair the `HashMap` with a `Vec<String>` for insertion-order keys:

```rust
pub struct ChipCatalog {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,  // insertion order
}
```

Provide `ordered_values()` alongside `get(&str)`.
