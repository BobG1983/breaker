# Entity Visual Attachment

## AttachVisuals Message

When a gameplay domain spawns an entity, it builds an `EntityVisualConfig` from its RON `rendering` substruct and sends `AttachVisuals`:

```rust
#[derive(Message, Clone)]
pub struct AttachVisuals {
    pub entity: Entity,
    pub config: EntityVisualConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EntityVisualConfig {
    pub shape: Shape,
    pub color: Hue,             // always resolved by the game before sending (temperature lookup is game-side)
    pub glow: GlowParams,
    pub aura: Option<Aura>,     // breakers have auras
    pub trail: Option<Trail>,   // breakers and bolts have trails
}
```

The crate handler:
1. Generates mesh from `Shape` variant
2. Creates Material2d with entity_glow shader parameterized by `GlowParams` and `Hue`
3. Spawns aura shader child entity (if `aura` is Some)
4. Spawns trail emitter child entity (if `trail` is Some)
5. Registers the entity for modifier tracking

## Event VFX

Damage, death, hit, etc. are NOT part of `AttachVisuals`. They fire at event time via `ExecuteRecipe`:

```rust
// Cell destroyed:
world.send(ExecuteRecipe { recipe: "shatter_sparks", position: cell_pos, camera: Some(cam) });

// Bolt lost:
world.send(ExecuteRecipe { recipe: "bolt_lost_streak", position: exit_pos, camera: Some(cam) });
```

## Rendering Substruct in Entity RON

Entity RON files contain a `rendering` block. The gameplay domain reads it, sends `AttachVisuals` at spawn, and stores recipe names for later event dispatch.

```ron
// assets/cells/tough.cell.ron
(
    cell_type: Tough,
    hp: 3,
    rendering: (
        shape: Hexagon,
        color: MediumSeaGreen,
        glow: (core_brightness: HdrBrightness(0.9), halo_radius: 2.0, halo_falloff: 1.5, bloom: BloomIntensity(0.6)),
        damage_recipe: "fracture_damage",
        death_recipe: "tough_death_single",
        death_recipe_combo: "tough_death_combo",       // default None → falls back to death_recipe
        death_recipe_chain: "tough_death_chain",       // default None → falls back to death_recipe
        hit_recipe: "cell_hit_tough",
    ),
)

// assets/breakers/chrono.breaker.ron
(
    name: "Chrono",
    bolt: "Bolt",
    stat_overrides: (),
    life_pool: Some(3),
    effects: [ ... ],
    rendering: (
        shape: Angular,
        color: Gold,
        glow: (core_brightness: HdrBrightness(1.0), halo_radius: 3.0, halo_falloff: 2.0, bloom: BloomIntensity(0.8)),
        aura: TimeDistortion(ripple_frequency: 2.0, echo_count: 3, intensity: 0.6, color: Amber),
        trail: Afterimage(copy_count: 4, fade_rate: 0.8, color: Amber, spacing: 2.0),
        perfect_bump_recipe: "bump_chrono_perfect",
        early_bump_recipe: "bump_chrono_early",
        late_bump_recipe: "bump_chrono_late",
        // whiff = no recipe fired (silence IS the feedback)
    ),
)

// assets/bolts/default.bolt.ron
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
        color: White,
        glow: (core_brightness: HdrBrightness(1.2), halo_radius: 3.0, halo_falloff: 2.0, bloom: BloomIntensity(1.0)),
        trail: ShieldEnergy(width: 1.5, fade_length: 40.0, color: White, intensity: 0.8),
        spawn_recipe: "bolt_spawn_flash",
        death_recipe: "bolt_lost_streak",
        expiry_recipe: "bolt_expiry_implosion",
    ),
)
```

Note: The `rendering` block fields that are entity-type-specific (damage_recipe for cells, bump_recipe for breakers, spawn/death/expiry_recipe for bolts) are NOT part of `EntityVisualConfig`. They are stored by the owning gameplay domain for event-time VFX dispatch. Only `shape`, `color`, `glow`, `aura`, `trail` go into `EntityVisualConfig` for `AttachVisuals`.

## RON Migration

Existing RON files gain the `rendering` block with `#[serde(default)]` for incremental migration. Until the graphics overhaul is complete, the rendering block can coexist with the current `color_rgb` fields.
