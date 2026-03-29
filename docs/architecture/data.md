# Data Model — Components vs Resources

How game data flows from RON config files to runtime systems via the ECS.

---

## Pipeline: RON → Config Resource → Init System → Entity Component

```
defaults.breaker.ron
        ↓  (asset loader)
Res<BreakerConfig>
        ↓  (init_breaker_params — runs OnEnter(Playing))
Entity components: BreakerWidth, BreakerHeight, MaxReflectionAngle, …
        ↓  (production systems query entities)
move_breaker, bolt_breaker_collision, …
```

**Config resources** (e.g., `Res<BreakerConfig>`, `Res<BoltConfig>`, `Res<CellConfig>`, `Res<PlayfieldConfig>`, etc.) are the bridge between RON data files and entity components. They exist so the asset pipeline has somewhere to deposit loaded values.

**Only init and spawn systems read config resources.** Every other system reads entity components. This keeps production systems decoupled from the config pipeline — they don't care whether a value came from RON, was overridden by an upgrade, or was injected in a test.

---

## Rules

### 1. Components on the owning entity

A component belongs on the entity it conceptually describes:

- `MaxReflectionAngle` is a breaker surface property → lives on the **breaker** entity, even though `bolt_breaker_collision` and `prepare_bolt_velocity` also read it
- `BoltRadius` is a bolt property → lives on the **bolt** entity, even though collision systems on other domains read it
- Cross-entity queries are normal ECS — reading a component from another entity is not coupling

### 2. Config fields follow the same ownership

If a value belongs to a domain, its config field and RON entry live in that domain's `*Defaults`:

- Breaker surface angles (`max_reflection_angle`, `min_angle_from_horizontal`) → `BreakerDefaults` / `defaults.breaker.ron`
- Bolt speed/radius → `BoltDefaults` / `defaults.bolt.ron`

Don't leave empty config resources — if all fields move elsewhere, delete the resource.

### 3. Store full dimensions, provide half helpers

Config files and components store the **full, intuitive value** (width, height). Systems that need halves call helper methods:

```rust
#[derive(Component, Debug)]
pub struct BreakerWidth(pub f32);

impl BreakerWidth {
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
pub struct BumpVisualParams {
    pub duration: f32,
    pub peak: f32,
    pub peak_fraction: f32,
    pub rise_ease: EaseFunction,
    pub fall_ease: EaseFunction,
}
```

When values are **independently accessed** across different systems, keep them as separate newtypes (`BoltBaseSpeed(f32)`, `BoltRadius(f32)`).

### 5. Init systems materialize components from config

Each domain has an `init_*_params` system that runs `OnEnter(Playing)` after the spawn system. It reads the config resource once and inserts all components:

```rust
pub fn init_breaker_params(
    mut commands: Commands,
    config: Res<BreakerConfig>,           // ← only place this is read
    query: Query<Entity, (With<Breaker>, Without<BreakerMaxSpeed>)>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            BreakerWidth(config.width),
            BreakerHeight(config.height),
            MaxReflectionAngle(config.max_reflection_angle),
            // …
        ));
    }
}
```

The `Without<BreakerMaxSpeed>` filter skips already-initialized entities (persisted across nodes).

---

## Active/Effective Component Pattern

Stat-modifying effects use a two-tier component model instead of accumulating flat deltas:

```
fire_effect(entity, DamageBoost(2.0))
        ↓  (push onto Active stack)
ActiveDamageBoosts(vec![2.0])
        ↓  (recalculate_damage — FixedUpdate, in EffectSystems::Recalculate)
EffectiveDamageMultiplier(2.0)    ← multiplier = product of all entries
        ↓  (consumers: bolt_cell_collision, handle_cell_hit)
```

**Rules:**

- **`Active*` components** (e.g., `ActiveDamageBoosts`, `ActiveSpeedBoosts`, `ActivePiercings`) live in the effect domain (`effect/effects/<name>.rs`). They are plain `Vec` stacks — each applied effect instance pushes one entry; `reverse_effect` removes it.
- **`Effective*` components** (e.g., `EffectiveDamageMultiplier`, `EffectiveSpeedMultiplier`, `EffectivePiercing`) are computed each frame by `recalculate_*` systems in `EffectSystems::Recalculate`. Multiplier stats use the product of all entries; additive stats (piercing) use the sum.
- **Consumers** (bolt collision, move_breaker, etc.) read only `Effective*` — never `Active*`. Consumers run `.after(EffectSystems::Recalculate)`.
- **`PiercingRemaining`** is bolt gameplay state (lives in the bolt domain), not an effect stat. `EffectivePiercing` sets the cap that `PiercingRemaining` resets to on wall/breaker contact.
- Both components are inserted by init systems (`init_bolt_params`, `init_breaker_params`) alongside base stat components. Without them, the entity is unaffected (collision code uses `Option<&EffectiveDamageMultiplier>` and maps to `1.0`).

---

## Testing

- **Init system tests** use `init_resource::<*Config>()` — they test the config→component bridge
- **Production system tests** spawn entities with component values directly — no config resource needed
- Tests may use `*Config::default()` to source component values, but never inject the config as a resource for the system under test

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
    fn extensions() -> &'static [&'static str] { &["bdef.ron"] }

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
| `BreakerRegistry` | `BreakerDefinition` (`bdef.ron`) | `String` (name) | Implements `SeedableRegistry`. Folder: `assets/breakers/`. Re-exported from `effect/` for historical reasons. |
| `ChipTemplateRegistry` | `ChipTemplate` (`chip.ron`) | `String` (name) | Implements `SeedableRegistry`. Folder: `assets/chips/templates/`. Stores `(AssetId, ChipTemplate)` pairs for hot-reload. |
| `EvolutionTemplateRegistry` | `EvolutionTemplate` (`evolution.ron`) | `String` (name) | Implements `SeedableRegistry`. Folder: `assets/chips/evolution/`. Stores `(AssetId, EvolutionTemplate)` pairs. |
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
assets/chips/templates/*.chip.ron
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
