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

## Testing

- **Init system tests** use `init_resource::<*Config>()` — they test the config→component bridge
- **Production system tests** spawn entities with component values directly — no config resource needed
- Tests may use `*Config::default()` to source component values, but never inject the config as a resource for the system under test

---

## Registry Pattern

Domain registries hold definitions loaded from RON during the loading screen. All registries follow a standard encapsulated pattern.

### Standard Shape

```rust
#[derive(Resource, Debug, Default)]
pub(crate) struct FooRegistry {
    items: HashMap<String, FooDefinition>,
}

impl FooRegistry {
    pub(crate) fn get(&self, name: &str) -> Option<&FooDefinition> { ... }
    pub(crate) fn insert(&mut self, def: FooDefinition) { ... }
    pub(crate) fn values(&self) -> impl Iterator<Item = &FooDefinition> { ... }
    pub(crate) fn len(&self) -> usize { ... }
    pub(crate) fn is_empty(&self) -> bool { ... }
}
```

Fields are **private** — all access goes through methods. This lets internals change (e.g., adding ordering) without breaking callers.

### Key Types

| Registry | Key | Value | Notes |
|----------|-----|-------|-------|
| `ChipRegistry` | `String` (name) | `ChipDefinition` | Paired `Vec<String>` preserves insertion order for deterministic chip offers |
| `NodeLayoutRegistry` | `String` (name) | `NodeLayout` | Paired `Vec<String>` preserves insertion order for index-based node progression |
| `ArchetypeRegistry` | `String` (name) | `ArchetypeDefinition` | Unsorted — callers sort at call site for UI display |
| `CellTypeRegistry` | `char` (alias) | `CellTypeDefinition` | Exception: keyed by grid alias char, not name |

### Pipeline

```
RON asset files
    ↓  (asset loader)
Assets<FooDefinition>
    ↓  (seed_foo_registry — loading screen system)
Res<FooRegistry>
    ↓  (production systems read via methods)
```

### When to Add Ordered Access

If a registry needs both name lookup and index-based access, pair the `HashMap` with a `Vec<String>` for insertion-order keys:

```rust
pub struct NodeLayoutRegistry {
    layouts: HashMap<String, NodeLayout>,
    order: Vec<String>,  // insertion order
}
```

Provide `get_by_index(usize)` alongside `get_by_name(&str)`.
