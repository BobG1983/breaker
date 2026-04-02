# Bolt Definitions Architecture

Bolt entities should be fully data-driven with their own RON definitions, registry, effect dispatch, and rendering substruct — matching the existing pattern for breakers and cells.

> **Implementation status**: `BoltDefinition`, `BoltRegistry`, `BoltDefinitionRef`, `BoltBaseDamage`, and `dispatch_bolt_effects` have been implemented (Waves 6-8 of the breaker-builder-pattern feature). `BoltConfig` has been eliminated. The `BoltRenderingConfig` substruct, `AttachVisuals` message, and `sync_bolt_visual_modifiers` system described in the Target State section are **not yet implemented** — they depend on `rantzsoft_vfx` which is a future phase.

## Prior State (Before Implementation)

### BoltConfig (GameConfig pattern — eliminated)

`BoltConfig` was a `Resource` in `bolt/resources.rs`, loaded from `assets/config/defaults.bolt.ron` via the `GameConfig` derive macro.

**Fields that become BoltDefinition fields:**
- `base_speed: f32` (720.0) — base speed in world units/second
- `min_speed: f32` (360.0) — minimum speed cap
- `max_speed: f32` (1440.0) — maximum speed cap
- `radius: f32` (14.0) — bolt radius in world units
- `color_rgb: [f32; 3]` ([6.0, 5.0, 0.5]) — HDR color (replaced by rendering substruct)

Additionally, `BASE_BOLT_DAMAGE: f32 = 10.0` is a constant in `bolt/resources.rs`. This becomes `base_damage: f32` in the definition and a `BoltBaseDamage(f32)` component on the bolt entity. Systems that need damage read the component.

**Fields that stay as components (initialized from constants for now):**
- `spawn_offset_y: f32` (54.0) — currently always 54, but remains a `BoltSpawnOffsetY` component so future bolt types or effects can override it
- `angle_spread: f32` (0.524) — currently always ~30°, but remains a `BoltAngleSpread` component. Used for both initial launch and respawn angle randomization. (Replaces both `initial_angle` and `respawn_angle_spread` which were always the same value.)
- `respawn_offset_y` is eliminated — always same as `spawn_offset_y`, so `BoltSpawnOffsetY` covers both

**Target:** `BoltConfig` will be eliminated entirely. Per-bolt physics fields move to `BoltDefinition`. Spawn offset and angle spread remain as components inserted by the builder from `BoltConfig` fields until the full `BoltDefinition` migration is complete.

### Current Spawn Flow (Implemented)

1. `spawn_bolt` runs on `OnEnter(GameState::Playing)`. If a `Bolt` entity exists (persists across nodes via `CleanupOnRunEnd`), it just fires `BoltSpawned` and returns.
2. Looks up `BoltDefinition` from `BoltRegistry` via `SelectedBreaker` → `BreakerRegistry` chain. Falls back to `BreakerDefinition::y_position` for spawn position.
3. Calls `Bolt::builder()` with `.definition(&bolt_def)`, `.at_position()`, `.rendered(...)` or `.headless()`, optionally `.serving()`, and `.primary()`. The builder inserts all components in a single `.spawn(world)` call: `Bolt`, `PrimaryBolt`, `Velocity2D`, `GameDrawLayer::Bolt`, `Position2D`, `PreviousPosition`, `Scale2D`, `PreviousScale`, `Aabb2D`, `CollisionLayers`, `BaseRadius`, `MinRadius`, `MaxRadius`, `BoltSpawnOffsetY`, `BoltAngleSpread`, `BoltBaseDamage`, `BoltDefinitionRef`, `BaseSpeed`, `MinSpeed`, `MaxSpeed`, `MinAngleH`, `MinAngleV`, `CleanupOnRunEnd`. Conditionally: `BoltServing` if serving.
4. `apply_node_scale_to_bolt` adds `NodeScalingFactor` from `ActiveNodeLayout.entity_scale`.
5. `dispatch_bolt_effects` runs in FixedUpdate, not OnEnter. It processes `Added<BoltDefinitionRef>` each tick and dispatches effects from the definition. Effects are dispatched on the first FixedUpdate tick after spawning, not synchronously during OnEnter.

There is no separate `init_bolt_params` step. The builder handles all parameter insertion at spawn time.

### Current Extra Bolt Spawn

Effect modules that spawn extra bolts (`SpawnBolts`, `SpawnPhantom`, `ChainBolt`, `MirrorProtocol`) each call `Bolt::builder()` directly — there is no shared `spawn_extra_bolt` helper function. The builder handles component insertion uniformly. Extra bolts use `.extra()` instead of `.primary()` and carry `ExtraBolt` + `CleanupOnNodeExit`.

### Current Bolt Lost (Implemented)

`bolt_lost` runs in `FixedUpdate`. For each bolt below playfield bottom:
- **Shield active**: Flips Y-velocity, calls `apply_velocity_formula`, clamps position. No `BoltLost` message.
- **Extra bolt**: Sends `BoltLost`, writes `RequestBoltDestroyed`. Entity stays alive one frame for `OnDeath` effect evaluation, then `cleanup_destroyed_bolts` despawns.
- **Baseline bolt**: Sends `BoltLost`. Respawns above breaker: reads `BoltSpawnOffsetY` and `BoltAngleSpread` from the bolt entity (constants-initialized components), calls `apply_velocity_formula`. Entity persists (no despawn/respawn cycle). `BoltRespawnOffsetY` was eliminated — `BoltSpawnOffsetY` covers both.

### Current Breaker → Bolt Relationship

The breaker does NOT spawn the bolt. They spawn independently. The breaker:
- Provides position anchor for bolt spawn/respawn
- Tracks serving bolt position (`hover_bolt`)
- Launches bolt on `GameAction::Bump` (`launch_bolt`)
- Reflects bolt on collision (`bolt_breaker_collision`)

Angle constraints (`min_angle_horizontal`, `min_angle_vertical`) come from `BoltConfig` fields embedded by the builder into the bolt entity's spatial components (`MinAngleH`, `MinAngleV`). The `apply_velocity_formula` function enforces these constraints at every velocity-modification site.

---

## Target State (What To Build)

### BoltDefinition

A new `BoltDefinition` struct, parallel to `BreakerDefinition`. Loaded from `.bolt.ron` files in `assets/bolts/`.

```rust
// bolt/definition.rs

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BoltDefinition {
    /// Display name of the bolt type.
    pub name: String,

    /// Base speed in world units per second.
    pub base_speed: f32,

    /// Minimum speed cap.
    pub min_speed: f32,

    /// Maximum speed cap.
    pub max_speed: f32,

    /// Bolt radius in world units.
    pub radius: f32,

    /// Base damage dealt per hit. Becomes a BoltBaseDamage component.
    /// Systems that need damage read the component, not a global constant.
    pub base_damage: f32,

    /// All effect chains for this bolt, each scoped to a target entity.
    /// Dispatched onto the bolt entity at spawn, identical to breaker dispatch.
    pub effects: Vec<RootEffect>,

    /// Visual rendering configuration.
    pub rendering: BoltRenderingConfig,
}
```

**Not in BoltDefinition** (components initialized from default constants by the builder):
- `BoltSpawnOffsetY(54.0)` — component on the bolt entity, inserted by the builder from a default constant. Future bolt types or effects could override it.
- `BoltAngleSpread(0.524)` — component on the bolt entity (~30°), used for both initial launch and respawn angle randomization. Inserted by the builder from a default constant, overridable.
- `BoltRespawnOffsetY` is eliminated — `BoltSpawnOffsetY` covers both spawn and respawn.

### BoltRenderingConfig

Inline rendering substruct. Maps to `EntityVisualConfig` for the `AttachVisuals` message, plus recipe names for event-time VFX.

```rust
// bolt/definition.rs

#[derive(Deserialize, Clone, Debug)]
pub struct BoltRenderingConfig {
    /// Shape of the bolt mesh.
    pub shape: Shape,

    /// Base color. None = game determines color (e.g., temperature).
    pub color: Option<Hue>,

    /// Entity glow shader parameters.
    pub glow: GlowParams,

    /// Trail configuration (shader type + RON-driven params).
    pub trail: Option<TrailConfig>,

    /// Named recipe for bolt spawn VFX.
    pub spawn_recipe: String,

    /// Named recipe for bolt-lost VFX.
    pub death_recipe: String,

    /// Named recipe for bolt lifespan expiry VFX (for time-limited bolts).
    pub expiry_recipe: Option<String>,
}
```

`GlowParams`, `TrailConfig`, `Shape`, `Hue` are types from `rantzsoft_vfx`. See [rendering.md](rendering.md) for definitions.

### Default Bolt RON File

```ron
// assets/bolts/default.bolt.ron
/* @[brickbreaker::bolt::definition::BoltDefinition] */
(
    name: "Bolt",
    base_speed: 720.0,
    min_speed: 360.0,
    max_speed: 1440.0,
    radius: 14.0,
    base_damage: 10.0,
    effects: [],
    rendering: (
        shape: Circle,
        color: Some(White),
        glow: (
            core_brightness: HdrBrightness(1.2),
            halo_radius: 3.0,
            halo_falloff: 2.0,
            bloom: BloomIntensity(1.0),
        ),
        trail: Some((
            shader: ShieldEnergy,
            params: (width: 1.5, fade_length: 40.0, intensity: 0.8),
        )),
        spawn_recipe: "bolt_spawn_flash",
        death_recipe: "bolt_lost_streak",
        expiry_recipe: Some("bolt_expiry_implosion"),
    ),
)
```

### BoltRegistry

Follows the exact `SeedableRegistry` pattern from `BreakerRegistry`:

```rust
// bolt/registry.rs

#[derive(Resource, Debug, Default)]
pub struct BoltRegistry {
    bolts: HashMap<String, BoltDefinition>,
}

impl SeedableRegistry for BoltRegistry {
    type Asset = BoltDefinition;

    fn asset_dir() -> &'static str {
        "bolts"
    }

    fn extensions() -> &'static [&'static str] {
        &["bolt.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<BoltDefinition>, BoltDefinition)]) {
        self.bolts.clear();
        for (_id, def) in assets {
            assert!(!self.bolts.contains_key(&def.name), "duplicate bolt name '{}'", def.name);
            self.bolts.insert(def.name.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<BoltDefinition>, asset: &BoltDefinition) {
        self.bolts.insert(asset.name.clone(), asset.clone());
    }
}
```

Methods: `get`, `contains`, `insert`, `names`, `iter`, `values`, `clear`, `len`, `is_empty` — identical API surface to `BreakerRegistry`.

### Breaker Definition Changes (Implemented)

`BreakerDefinition` was expanded with all gameplay fields (previously in `BreakerConfig`) and a `bolt` field linking to the bolt archetype. `BreakerStatOverrides` and `BreakerConfig` were eliminated — the definition is the single source of truth.

```rust
// breaker/definition.rs — current shape (abbreviated)

#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BreakerDefinition {
    pub name: String,                    // required, no default
    #[serde(default = "default_bolt_name")]
    pub bolt: String,                    // default: "Bolt"
    #[serde(default)]
    pub life_pool: Option<u32>,          // default: None (infinite)
    #[serde(default)]
    pub effects: Vec<RootEffect>,        // default: []
    // 29+ gameplay fields with #[serde(default)]:
    pub width: f32, pub height: f32, pub y_position: f32,
    pub min_w: Option<f32>, pub max_w: Option<f32>,
    pub min_h: Option<f32>, pub max_h: Option<f32>,
    pub max_speed: f32, pub acceleration: f32, pub deceleration: f32,
    // ... dash, brake, settle, bump timing, spread, color_rgb
}
```

RON files use `.breaker.ron` extension and only specify overrides (defaults apply to omitted fields):

```ron
// assets/breakers/aegis.breaker.ron
(
    name: "Aegis",
    life_pool: Some(3),
    effects: [ ... ],
)
```

See `breaker-game/assets/breakers/breaker.example.ron` for the full annotated field list.

---

## System Changes Required

### 1. New Files to Create

| File | Contents |
|------|----------|
| `bolt/definition.rs` | `BoltDefinition`, `BoltRenderingConfig` structs |
| `bolt/registry.rs` | `BoltRegistry` implementing `SeedableRegistry` |
| `assets/bolts/default.bolt.ron` | Default bolt definition |

### 2. Plugin Registration Changes

**bolt/plugin.rs:**
- Register `BoltDefinition` as an asset: `app.init_asset::<BoltDefinition>()`
- Register `BoltRegistry` via `RantzDefaultsPluginBuilder::add_registry::<BoltRegistry>()`
- This follows the pattern in `breaker/plugin.rs` and `chips/plugin.rs`

**bolt/mod.rs:**
- Add `pub mod definition;` and `pub mod registry;`
- Re-export `BoltDefinition`, `BoltRegistry`

### 3. Eliminate BoltConfig

`BoltConfig` is removed entirely. All per-bolt fields have moved to `BoltDefinition`. The remaining values become constants:

```rust
// bolt/constants.rs (or bolt/resources.rs)

/// Default vertical offset above breaker for bolt spawn and respawn.
/// Used to initialize BoltSpawnOffsetY component. The component is the runtime authority.
pub const DEFAULT_BOLT_SPAWN_OFFSET_Y: f32 = 54.0;

/// Default angle spread from vertical for launch and respawn (radians, ~30°).
/// Used to initialize BoltAngleSpread component. The component is the runtime authority.
pub const DEFAULT_BOLT_ANGLE_SPREAD: f32 = 0.524;
```

**Files affected by BoltConfig removal:**
- `bolt/resources.rs` — delete `BoltConfig` struct and its `Default` impl, delete `BASE_BOLT_DAMAGE` constant
- `bolt/plugin.rs` — remove `add_config::<BoltConfig>()` registration
- `assets/config/defaults.bolt.ron` — delete file
- `bolt/systems/spawn_bolt/` — change from `Res<BoltConfig>` to `Res<BoltRegistry>` + `BoltDefinitionRef`
- `bolt/systems/init_bolt_params.rs` — change from `Res<BoltConfig>` to `Res<BoltRegistry>` + `BoltDefinitionRef`
- `bolt/systems/bolt_lost/` — change from `Res<BoltConfig>` to constant + `Res<BoltRegistry>`
- `bolt/systems/launch_bolt.rs` — reads `BoltBaseSpeed` and angle spread constant instead of config
- `bolt/systems/hover_bolt.rs` — reads spawn offset constant instead of config
- `effect/effects/fire_helpers.rs` — reads `Res<BoltRegistry>` + `BoltDefinitionRef` instead of `Res<BoltConfig>`
- `debug/hot_reload/` — remove `propagate_bolt_config_changes`, add `propagate_bolt_definition_changes`
- All tests that inject `BoltConfig` as a resource

### 4. New BoltBaseDamage Component

```rust
// bolt/components/definitions.rs — ADD

/// Base damage dealt by this bolt per hit.
/// Read by collision systems instead of the former BASE_BOLT_DAMAGE constant.
#[derive(Component, Debug, Clone, Copy)]
pub struct BoltBaseDamage(pub f32);
```

Inserted by `init_bolt_params` from `BoltDefinition.base_damage`. Systems that currently reference `BASE_BOLT_DAMAGE` (bolt_cell_collision, and any effect that applies damage) instead query for `&BoltBaseDamage` on the bolt entity.

**Files affected:**
- `bolt/systems/bolt_cell_collision/` — query `&BoltBaseDamage` instead of using the constant
- Any effect damage system that hardcodes `BASE_BOLT_DAMAGE`

### 5. spawn_bolt System Changes

Current: reads `BoltConfig` for all parameters.

After:

```rust
fn spawn_bolt(
    bolt_registry: Res<BoltRegistry>,
    breaker_registry: Res<BreakerRegistry>,
    selected_breaker: Res<SelectedBreaker>,
    // ... other params
) {
    // 1. Look up the selected breaker's bolt name
    let breaker_def = breaker_registry.get(&selected_breaker.name).unwrap();
    let bolt_def = bolt_registry.get(&breaker_def.bolt).unwrap();

    // 2. Compute spawn position (using default constant — component set later by init_bolt_params)
    let spawn_pos = Vec2::new(breaker_x, breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    // 3. Compute initial velocity (zero if serving, else random within default angle spread)
    let velocity = if node_index == 0 {
        Velocity2D::ZERO
    } else {
        let angle = rng.gen_range(-DEFAULT_BOLT_ANGLE_SPREAD..DEFAULT_BOLT_ANGLE_SPREAD);
        Velocity2D(Vec2::new(
            bolt_def.base_speed * angle.sin(),
            bolt_def.base_speed * angle.cos(),
        ))
    };

    // 4. Spawn entity with gameplay components (NO Mesh2d/MeshMaterial2d)
    let bolt_entity = commands.spawn((
        Bolt,
        BoltDefinitionRef(bolt_def.name.clone()),
        velocity,
        Position2D::new(spawn_pos),
        Aabb2D::new(Vec2::ZERO, Vec2::new(bolt_def.radius, bolt_def.radius)),
        CollisionLayers::bolt(),
        GameDrawLayer::Bolt,
        CleanupOnRunEnd,
        // Conditionally: BoltServing
    )).id();

    // 5. Build EntityVisualConfig from bolt_def.rendering and send AttachVisuals
    let visual_config = EntityVisualConfig {
        shape: bolt_def.rendering.shape,
        color: bolt_def.rendering.color,
        glow: bolt_def.rendering.glow.clone(),
        aura: None,  // bolts don't have auras
        trail: bolt_def.rendering.trail.clone(),
    };
    world.send(AttachVisuals { entity: bolt_entity, config: visual_config });

    // 6. Send BoltSpawned message
    world.send(BoltSpawned);
}
```

### 6. init_bolt_params System Changes

Reads from `BoltDefinitionRef` + registry instead of `BoltConfig`.

```rust
fn init_bolt_params(
    query: Query<(Entity, &BoltDefinitionRef), Added<Bolt>>,
    bolt_registry: Res<BoltRegistry>,
    mut commands: Commands,
) {
    for (entity, def_ref) in &query {
        let def = bolt_registry.get(&def_ref.0).unwrap();
        commands.entity(entity).insert((
            BoltBaseSpeed(def.base_speed),
            BoltMinSpeed(def.min_speed),
            BoltMaxSpeed(def.max_speed),
            BoltRadius(def.radius),
            BoltBaseDamage(def.base_damage),
            BoltSpawnOffsetY(DEFAULT_BOLT_SPAWN_OFFSET_Y),  // 54.0 — component, overridable by effects
            BoltAngleSpread(DEFAULT_BOLT_ANGLE_SPREAD),      // 0.524 (~30°) — component, overridable
        ));
    }
}
```

**Components kept:** `BoltSpawnOffsetY` (initialized from default constant, overridable), `BoltAngleSpread` (replaces both `BoltInitialAngle` and `BoltRespawnAngleSpread` — they were always the same value).

**Components removed:** `BoltRespawnOffsetY` (redundant — same as spawn offset), `BoltRespawnAngleSpread` (merged into `BoltAngleSpread`), `BoltInitialAngle` (merged into `BoltAngleSpread`).

### 7. Bolt Effect Dispatch (NEW)

New system: `dispatch_bolt_effects`, parallel to `dispatch_breaker_effects`.

When a bolt spawns with a `BoltDefinitionRef`, the dispatch system reads the definition's `effects: Vec<RootEffect>` and dispatches them onto the bolt entity using the same pattern as breaker dispatch:

```rust
fn dispatch_bolt_effects(
    query: Query<(Entity, &BoltDefinitionRef), Added<BoltDefinitionRef>>,
    bolt_registry: Res<BoltRegistry>,
    mut commands: Commands,
) {
    for (entity, def_ref) in &query {
        let def = bolt_registry.get(&def_ref.0).unwrap();
        for root_effect in &def.effects {
            let RootEffect::On { target, then } = root_effect;
            // Same dispatch logic as breaker: resolve target, fire bare Do, push non-Do to BoundEffects
            // ...
        }
    }
}
```

This lives in `bolt/systems/dispatch_bolt_effects.rs`. Follow the exact pattern from `breaker/systems/dispatch_breaker_effects/`.

### 8. spawn_extra_bolt Changes

`spawn_extra_bolt` in `effect/effects/fire_helpers.rs` currently reads `BoltConfig` for radius and speed. After:

```rust
pub fn spawn_extra_bolt(
    source_entity: Entity,
    world: &mut World,
) -> Entity {
    // Read source bolt's definition ref
    let def_ref = world.get::<BoltDefinitionRef>(source_entity)
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "Bolt".to_owned());

    let bolt_def = world.resource::<BoltRegistry>().get(&def_ref).unwrap().clone();

    // Random angle — read from source bolt's BoltAngleSpread component (falls back to default)
    let angle_spread = world.get::<BoltAngleSpread>(source_entity)
        .map(|a| a.0)
        .unwrap_or(DEFAULT_BOLT_ANGLE_SPREAD);
    let angle = rng.gen_range(-angle_spread..angle_spread);
    let velocity = Vec2::new(
        bolt_def.base_speed * angle.sin(),
        bolt_def.base_speed * angle.cos(),
    );

    // Spawn with BoltDefinitionRef so the extra bolt also has a definition
    let extra = world.spawn((
        ExtraBolt,
        BoltDefinitionRef(def_ref),
        Velocity2D(velocity),
        Position2D::new(source_position),
        Aabb2D::new(Vec2::ZERO, Vec2::new(bolt_def.radius, bolt_def.radius)),
        CollisionLayers::bolt(),
        GameDrawLayer::Bolt,
        CleanupOnNodeExit,
    )).id();

    // Build EntityVisualConfig from bolt_def.rendering and send AttachVisuals
    // (This fixes the current bug where extra bolts are invisible)
    let visual_config = EntityVisualConfig {
        shape: bolt_def.rendering.shape,
        color: bolt_def.rendering.color,
        glow: bolt_def.rendering.glow.clone(),
        aura: None,
        trail: bolt_def.rendering.trail.clone(),
    };
    world.send(AttachVisuals { entity: extra, config: visual_config });

    extra
}
```

### 9. Dynamic Visuals via Modifier System (replaces BoltRenderState)

There is **no BoltRenderState component**. All dynamic visual state is communicated through the modifier system from `rantzsoft_vfx`. See [rendering.md](rendering.md) for the full modifier API.

Bolt/ domain sends modifier messages each FixedUpdate:

```rust
// bolt/systems/sync_bolt_visual_modifiers.rs

fn sync_bolt_visual_modifiers(
    query: Query<(Entity, &Velocity2D, &BoltBaseSpeed, &BoltMaxSpeed,
                  Option<&ActivePiercings>, Option<&EffectiveDamageMultiplier>,
                  Option<&ShieldActive>, Option<&BoltServing>), With<Bolt>>,
    mut set_writer: MessageWriter<SetModifier>,
) {
    for (entity, velocity, base_speed, max_speed, piercing, damage_mult, shield, serving) in &query {
        let speed_fraction = velocity.0.length() / max_speed.0;

        // Speed → trail length (overwrites each frame)
        set_writer.send(SetModifier {
            entity,
            modifier: VisualModifier::TrailLength(speed_fraction * 2.0),
            source: "bolt_speed".into(),
        });

        // Speed → core brightness
        set_writer.send(SetModifier {
            entity,
            modifier: VisualModifier::CoreBrightness(0.8 + speed_fraction * 0.4),
            source: "bolt_speed_glow".into(),
        });

        // Piercing → spike count
        if let Some(piercing) = piercing {
            set_writer.send(SetModifier {
                entity,
                modifier: VisualModifier::SpikeCount(piercing.remaining().min(6)),
                source: "bolt_piercing".into(),
            });
        }

        // Serving → dim pulsing mode
        if serving.is_some() {
            set_writer.send(SetModifier {
                entity,
                modifier: VisualModifier::CoreBrightness(0.7),
                source: "bolt_serving".into(),
            });
            set_writer.send(SetModifier {
                entity,
                modifier: VisualModifier::TrailLength(0.0),
                source: "bolt_serving_trail".into(),
            });
        }
    }
}
```

Chip effects (SpeedBoost, DamageBoost, etc.) use `AddModifier`/`RemoveModifier` in their fire/reverse functions for stacking modifiers with diminishing returns. See [rendering.md — Modifier System](rendering.md#modifier-system).

### 10. bolt_lost Changes

Currently reads `BoltConfig` for respawn params. After:
- Spawn offset reads `BoltSpawnOffsetY` component on the bolt entity
- Respawn angle reads `BoltAngleSpread` component on the bolt entity
- Base speed reads `BoltBaseSpeed` component on the bolt entity
- No `BoltConfig` dependency

```rust
fn bolt_lost(
    // ... No Res<BoltConfig>
    query: Query<(Entity, &Position2D, &BoltBaseSpeed, &BoltSpawnOffsetY, &BoltAngleSpread, ...), With<Bolt>>,
) {
    // Baseline bolt respawn:
    let angle = rng.gen_range(-angle_spread.0..angle_spread.0);
    velocity.0 = Vec2::new(
        base_speed.0 * angle.sin(),
        base_speed.0 * angle.cos(),
    );
    position.0 = Vec2::new(breaker_x, breaker_y + spawn_offset.0);
}
```

### 11. Visual Attachment Changes

Currently `spawn_bolt` inserts `Mesh2d(Circle::new(1.0))` and `MeshMaterial2d` directly. After:
- `spawn_bolt` does NOT insert `Mesh2d` or `MeshMaterial2d`
- Instead, it builds `EntityVisualConfig` from `bolt_def.rendering` and sends `AttachVisuals { entity, config }`
- `rantzsoft_vfx` receives the message and attaches mesh + material + shaders + trail emitter
- Same pattern for `spawn_extra_bolt` — fixes the current bug where extra bolts are invisible

### 12. Breaker Definition Changes

Add `bolt: String` field with `#[serde(default = "default_bolt_name")]` to `BreakerDefinition`. Update all three breaker RON files to include `bolt: "Bolt"`. The default ensures backwards compatibility.

### 13. Run Setup Changes

The run setup screen (breaker select) currently only selects a breaker. The bolt comes from the breaker definition's `bolt` field. No separate bolt selection UI for now — the breaker choice implies the bolt.

Future: bolt selection could be a separate UI step, or a meta-progression unlock.

### 14. Scenario Runner Changes

The scenario runner (`breaker-scenario-runner`) accesses bolt components for invariant checking. Changes needed:
- Import `BoltDefinition`, `BoltRegistry`, `BoltDefinitionRef`
- Update `menu_bypass.rs` to handle the new bolt selection (look up bolt name from breaker def)
- Ensure `BoltRegistry` is populated in test worlds

### 15. Hot-Reload Changes

`debug/hot_reload/` currently propagates `BoltConfig` changes. After:
- Remove `propagate_bolt_config_changes` (BoltConfig no longer exists)
- Add `propagate_bolt_definition_changes` (same pattern as `propagate_breaker_changes`)
- When a `.bolt.ron` file changes, update the registry and re-apply params to live bolt entities via `BoltDefinitionRef`

### 16. Test Changes

- All tests that construct `BoltConfig` for physics params must be updated to use `BoltDefinition` instead
- All tests that reference `BASE_BOLT_DAMAGE` must use `BoltBaseDamage` component
- Tests that call `spawn_extra_bolt` must have a `BoltRegistry` resource in the test world
- Remove tests for eliminated components (`BoltSpawnOffsetY`, `BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, `BoltInitialAngle`)
- New tests needed for:
  - `BoltDefinition` RON parsing (including rendering substruct)
  - `BoltRegistry` seed/update/lookup (follow `BreakerRegistry` test pattern)
  - `dispatch_bolt_effects` (follow `dispatch_breaker_effects` test pattern)
  - `sync_bolt_visual_modifiers` (sends correct SetModifier messages for each state)
  - `spawn_bolt` using definition instead of config
  - `spawn_extra_bolt` inheriting parent's definition and getting AttachVisuals
  - Breaker definition's `bolt` field lookup
  - `BoltBaseDamage` component is read correctly by collision systems

---

## RON File Inventory

### New Files

| File | Contents |
|------|----------|
| `assets/bolts/default.bolt.ron` | Default bolt definition (physics + rendering + empty effects) |

### Modified Files

| File | Change |
|------|--------|
| `assets/breakers/aegis.breaker.ron` | Has `bolt` field (defaulted to "Bolt") |
| `assets/breakers/chrono.breaker.ron` | Has `bolt` field (defaulted to "Bolt") |
| `assets/breakers/prism.breaker.ron` | Has `bolt` field (defaulted to "Bolt") |

### Deleted Files

| File | Reason |
|------|--------|
| `assets/config/defaults.bolt.ron` | BoltConfig eliminated, all fields in BoltDefinition |

### Future Files (not needed for initial implementation)

| File | When |
|------|------|
| `assets/bolts/heavy.bolt.ron` | When a second bolt type is designed |
| `assets/bolts/spectral.bolt.ron` | If phantom bolts get their own definition |

---

## Migration Checklist

This is the implementation order. Each step should compile and pass tests before proceeding.

1. **Create `BoltDefinition` and `BoltRenderingConfig`** — definition.rs with all fields, derives Deserialize. Include `base_damage` field.
2. **Create `BoltRegistry`** — registry.rs implementing SeedableRegistry, follow BreakerRegistry pattern exactly.
3. **Create `default.bolt.ron`** — values matching current BoltConfig + BASE_BOLT_DAMAGE. No `initial_angle` or `respawn_angle_spread` fields.
4. **Register in plugin** — init_asset, add_registry in BoltPlugin.
5. **Add `BoltDefinitionRef(String)` component** — in bolt/components.
6. **Add `BoltBaseDamage(f32)` component** — in bolt/components.
7. **Add `bolt` field to BreakerDefinition** — with serde default, update all three breaker RON files.
8. **Create bolt domain default constants** — `DEFAULT_BOLT_SPAWN_OFFSET_Y: f32 = 54.0`, `DEFAULT_BOLT_ANGLE_SPREAD: f32 = 0.524`. Used by `init_bolt_params` to initialize components. Components are the runtime authority; constants are just defaults.
9. **Update `spawn_bolt`** — read from BoltRegistry via breaker's bolt field, insert BoltDefinitionRef, build EntityVisualConfig, send AttachVisuals. Stop inserting Mesh2d/MeshMaterial2d.
10. **Update `init_bolt_params`** — read from BoltDefinitionRef + registry, insert BoltBaseDamage, remove eliminated component insertions.
11. **Create `dispatch_bolt_effects`** — system parallel to dispatch_breaker_effects, fires on Added<BoltDefinitionRef>.
12. **Update `spawn_extra_bolt`** — read from parent's BoltDefinitionRef, inherit definition, send AttachVisuals (fixes invisible extra bolts).
13. **Create `sync_bolt_visual_modifiers`** — new system sending SetModifier messages each FixedUpdate for speed, piercing, serving state, etc.
14. **Update `bolt_lost`** — use constants for spawn offset and angle spread, read BoltBaseSpeed from entity.
15. **Update `bolt_cell_collision`** — read `BoltBaseDamage` component instead of `BASE_BOLT_DAMAGE` constant.
16. **Eliminate `BoltConfig`** — delete struct, delete defaults.bolt.ron, remove all `Res<BoltConfig>` references.
17. **Rename/consolidate components** — rename `BoltInitialAngle` + `BoltRespawnAngleSpread` → `BoltAngleSpread` (single component, same value for both). Delete `BoltRespawnOffsetY` (redundant — `BoltSpawnOffsetY` covers both). Keep `BoltSpawnOffsetY` as-is.
18. **Update tests** — all tests that reference removed types, add new tests for all new systems.
19. **Update hot-reload** — remove bolt config propagation, add bolt definition propagation.
20. **Update scenario runner** — menu_bypass, imports, BoltRegistry in test worlds.
