# Bolt Graphics Migration

How to migrate bolt rendering from the current Mesh2d/MeshMaterial2d/color_rgb system to the `rantzsoft_vfx` architecture described in [index.md](index.md).

**Prerequisite**: The bolt-definitions spec must be implemented first. This document describes the delta from that end state.

---

## What Changes

### 1. BoltDefinition: `color_rgb` → `rendering` substruct

**Before** (post bolt-definitions spec):
```rust
pub struct BoltDefinition {
    pub name: String,
    pub base_speed: f32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub radius: f32,
    pub base_damage: f32,
    pub effects: Vec<RootEffect>,
    pub color_rgb: [f32; 3],
}
```

**After**:
```rust
pub struct BoltDefinition {
    pub name: String,
    pub base_speed: f32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub radius: f32,
    pub base_damage: f32,
    pub effects: Vec<RootEffect>,
    pub rendering: BoltRenderingConfig,
}

pub struct BoltRenderingConfig {
    pub shape: Shape,
    pub color: Hue,             // always resolved by game before sending AttachVisuals
    pub glow: GlowParams,
    pub trail: Option<Trail>,   // Trail enum (ShieldEnergy, Afterimage, PrismaticSplit)
    pub spawn_recipe: String,
    pub death_recipe: String,
    pub expiry_recipe: Option<String>,
}
```

### 2. RON: `color_rgb` → `rendering` block

**Before**:
```ron
(
    name: "Bolt",
    base_speed: 720.0,
    // ...
    color_rgb: (6.0, 5.0, 0.5),
)
```

**After**:
```ron
(
    name: "Bolt",
    base_speed: 720.0,
    // ...
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

### 3. spawn_bolt: Mesh2d/MeshMaterial2d → AttachVisuals

**Before** (post bolt-definitions spec):
```rust
// In spawn_bolt:
let color = Color::linear_rgb(bolt_def.color_rgb[0], bolt_def.color_rgb[1], bolt_def.color_rgb[2]);
commands.spawn((
    // ...
    Mesh2d(render_assets.0.add(Circle::new(1.0))),
    MeshMaterial2d(render_assets.1.add(ColorMaterial::from_color(color))),
));
```

**After**:
```rust
// In spawn_bolt:
let bolt_entity = commands.spawn((
    // ... gameplay components only, NO Mesh2d/MeshMaterial2d
)).id();

let visual_config = EntityVisualConfig {
    shape: bolt_def.rendering.shape,
    color: bolt_def.rendering.color,
    glow: bolt_def.rendering.glow.clone(),
    aura: None,
    trail: bolt_def.rendering.trail.clone(),
};
// AttachVisuals is a rantzsoft_vfx message
attach_writer.send(AttachVisuals { entity: bolt_entity, config: visual_config });
```

Remove `ResMut<Assets<Mesh>>` and `ResMut<Assets<ColorMaterial>>` params from `spawn_bolt`.

### 4. spawn_extra_bolt: same change

Same Mesh2d/MeshMaterial2d → AttachVisuals replacement. Extra bolts read rendering config from the inherited `BoltDefinitionRef`.

### 5. Dynamic visuals: BoltConfig reads → modifier messages

**Before** (post bolt-definitions spec, no BoltRenderState):
Bolt has no dynamic visual communication. Speed, piercing, etc. have no visual effect.

**After**:
New system `sync_bolt_visual_modifiers` in bolt/ sends modifier messages each FixedUpdate:

```rust
// Speed → trail length
set_writer.send(SetModifier { entity, modifier: TrailLength(speed_fraction * 2.0), source: "bolt_speed" });

// Piercing → spike count
set_writer.send(SetModifier { entity, modifier: SpikeCount(piercing_count), source: "bolt_piercing" });

// Serving → dim mode
set_writer.send(SetModifier { entity, modifier: CoreBrightness(0.7), source: "bolt_serving" });
```

Chip effects (SpeedBoost, etc.) send `AddModifier`/`RemoveModifier` in their fire/reverse functions.

### 6. Event VFX: new messages at event time

Bolt spawn, bolt lost, and bolt expiry send recipe-based VFX messages:

```rust
// On bolt spawn:
vfx_writer.send(ExecuteRecipe { recipe: bolt_def.rendering.spawn_recipe.clone(), position: spawn_pos });

// On bolt lost:
vfx_writer.send(ExecuteRecipe { recipe: bolt_def.rendering.death_recipe.clone(), position: exit_pos });

// On bolt lifespan expiry:
if let Some(ref recipe) = bolt_def.rendering.expiry_recipe {
    vfx_writer.send(ExecuteRecipe { recipe: recipe.clone(), position: bolt_pos });
}
```

---

## Summary of Removals

| Removed | Replaced By |
|---------|-------------|
| `color_rgb: [f32; 3]` on BoltDefinition | `rendering: BoltRenderingConfig` |
| `Mesh2d` + `MeshMaterial2d` insertion in spawn_bolt | `AttachVisuals` message |
| `ResMut<Assets<Mesh>>` + `ResMut<Assets<ColorMaterial>>` params | Gone — crate handles mesh/material |
| Direct material creation in spawn_extra_bolt | `AttachVisuals` message |

## Summary of Additions

| Added | Purpose |
|-------|---------|
| `BoltRenderingConfig` struct | shape, color, glow, trail, recipe names |
| `AttachVisuals` message send in spawn_bolt/spawn_extra_bolt | Crate creates mesh/material/shaders |
| `sync_bolt_visual_modifiers` system | Sends SetModifier for speed, piercing, serving |
| `AddModifier`/`RemoveModifier` in effect fire/reverse | Chip visual effects with DR |
| `ExecuteRecipe` messages in bolt_lost, spawn, expiry | Recipe-driven event VFX |
