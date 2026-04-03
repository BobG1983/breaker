# 5d: Bolt Visuals

## Summary

Replace the bolt's flat `ColorMaterial` circle with SDF energy orb rendering via `EntityGlowMaterial`, glow halo with bloom, and a wake trail entity that scales with speed. The bolt builder's `spawn()` reads the `BoltDefinition` rendering block and directly attaches visual components (Shape, Hue, GlowParams, Trail, `EntityGlowMaterial` handle, trail child entity). No `AttachVisuals` message. A new `sync_bolt_visual_modifiers` system in bolt/ drives runtime visual state (speed-based trail, piercing spikes, serving dimness, lifespan fade) by sending `SetModifier` messages to the visuals domain.

## Context

### Architecture (revised)

The revised architecture (`docs/todos/detail/phase-5-rethink/architecture.md`) eliminates the monolithic `rantzsoft_vfx` crate, recipe system, and `AttachVisuals` god-message. Instead:

- The `visuals/` domain (5e) provides type definitions: `Shape`, `Hue`, `GlowParams`, `Trail`, `EntityVisualConfig`, `EntityGlowMaterial`, `TrailRibbonMaterial`, `ModifierStack`, `VisualModifier`, and the `entity_glow.wgsl` SDF shader.
- Each domain's builder directly attaches visual components at spawn time.
- Runtime visual changes go through `SetModifier` / `AddModifier` / `RemoveModifier` messages owned by visuals/.
- Modifier computation (reading `ModifierStack` and updating material uniforms) is built in 5j, not here.

### Current bolt builder

The bolt builder (`breaker-game/src/bolt/builder/`) uses a typestate pattern with a `V` (visual) dimension:
- `Unvisual` -> `Rendered` (via `.rendered(meshes, materials)` which creates a `Circle` mesh + `ColorMaterial`)
- `Unvisual` -> `Headless` (no visual components)

`Rendered` currently stores `Handle<Mesh>` and `Handle<ColorMaterial>`. The terminal `spawn()` and `build()` methods insert `Mesh2d`, `MeshMaterial2d<ColorMaterial>`, and `GameDrawLayer::Bolt`.

### Current BoltDefinition

`BoltDefinition` in `breaker-game/src/bolt/definition.rs` has gameplay fields only: name, speeds, radius, damage, effects, `color_rgb: [f32; 3]`, angle constraints, min/max radius. No rendering block yet.

### Bolt visual design

From `docs/design/graphics/gameplay-elements.md`:
- **Form**: Energy orb -- bright central sphere + softer glow halo. HDR > 1.0 for bloom.
- **Wake**: Trailing energy wake showing direction and recent path. Length scales with speed.
- **State communication**: Speed -> trail length/brightness. Piercing -> spikes/angular glow. Damage boosted -> hotter color. Serving -> dimmer, no trail, pulsing.
- **Chip modifiers**: Stack additively with diminishing returns on visuals only.

## What to Build

### 1. BoltDefinition Rendering Block

Add an optional `rendering` field to `BoltDefinition` that uses `EntityVisualConfig` from the visuals domain. When present, the builder reads shape, color, glow, and trail from here instead of the legacy `color_rgb` field. When absent (or when individual fields are `None`), sensible defaults produce the current bolt appearance.

The rendering block replaces the flat `color_rgb` field for visual purposes. `color_rgb` remains for backward compatibility and as a fallback but is superseded by `rendering.color` when both exist.

### 2. Bolt Builder Visual Dimension Replacement

Replace the `Rendered` typestate marker's contents. Instead of storing `Handle<Mesh>` + `Handle<ColorMaterial>`, the `Rendered` marker stores the data needed to create an `EntityGlowMaterial`:

- `Handle<EntityGlowMaterial>` -- the SDF material created from the definition's rendering block (shape, color, glow params mapped to `EntityGlowUniforms`)
- `Handle<Mesh>` -- a unit quad (not a `Circle`), since the SDF shader renders the shape
- Optional trail configuration from the definition's `Trail` enum

The `.rendered()` transition method changes signature: instead of taking `&mut Assets<Mesh>, &mut Assets<ColorMaterial>`, it takes `&mut Assets<Mesh>, &mut Assets<EntityGlowMaterial>` and reads visual config from the definition (stored in `OptionalBoltData` during the `.definition()` call).

Headless mode is unchanged -- tests that don't need rendering still use `.headless()`.

### 3. Trail Child Entity Spawning

When the definition's rendering block specifies a `Trail` variant, the builder's `spawn()` creates a trail child entity:

- **ShieldEnergy trail**: Spawns a child entity with `TrailRibbonMaterial`, a ribbon mesh, and a `BoltWakeTrail` marker component. The trail entity tracks the bolt entity.
- **Afterimage trail**: Spawns a pool of sprite entities (based on `copy_count`) with `BoltAfterimagePool` marker. Each sprite fades and repositions to follow the bolt's recent positions.

The trail entity is a child of the bolt entity (via Bevy parent-child) so it is automatically despawned when the bolt despawns.

Trail rendering systems (ring buffer position sampling, ribbon mesh updates, afterimage pool repositioning) are built here in bolt/ because they are bolt-specific logic that reads bolt velocity and position. The materials and types come from visuals/.

### 4. Trail Update Systems (bolt-owned)

Two new systems in `bolt/systems/`:

- `update_bolt_wake_trail`: For `ShieldEnergy` and `PrismaticSplit` trail variants. Runs in `Update`. Samples the bolt's position each frame into a ring buffer. Updates the trail ribbon mesh vertices and alpha gradient based on the ring buffer. Trail length scales with the bolt's current speed (faster = longer trail). Trail width and color come from the `Trail` params.

- `update_bolt_afterimage_trail`: For `Afterimage` trail variant. Runs in `Update`. Repositions afterimage sprites to the bolt's N most recent positions (from the ring buffer). Applies fade based on `fade_rate` and `spacing`.

Both systems are bolt-domain code. They read from `visuals::Trail` params but operate on bolt entities. They register in `BoltPlugin`.

### 5. sync_bolt_visual_modifiers System

A new system in `bolt/systems/` that runs in `FixedUpdate` (after physics, before rendering). Reads bolt state and sends `SetModifier` messages to the visuals domain:

| Bolt state | Modifier | Source key | Value |
|------------|----------|------------|-------|
| Speed (velocity magnitude / max_speed) | `TrailLength` | `"bolt_speed"` | `speed_fraction * 2.0` (0.0 at rest, 2.0 at max) |
| Speed | `GlowIntensity` | `"bolt_speed"` | `0.8 + speed_fraction * 0.4` (0.8 at rest, 1.2 at max) |
| Piercing active (from chip effect) | `SpikeCount` | `"bolt_piercing"` | piercing stack count as u32 |
| `BoltServing` present | `CoreBrightness` | `"bolt_serving"` | `0.7` (dimmer while hovering) |
| `BoltServing` present | `TrailLength` | `"bolt_serving"` | `0.0` (no trail while serving) |
| `BoltLifespan` below 30% remaining | `CoreBrightness` | `"bolt_lifespan"` | `fraction * 2.0` (dims as lifespan expires) |
| `BoltLifespan` below 15% remaining | `AlphaOscillation` | `"bolt_lifespan"` | `{ min: 0.4, max: 1.0, frequency: 8.0 }` (flicker) |

This system does NOT read `ModifierStack` or update material uniforms -- that is the modifier computation system in 5j. This system only sends messages to SET the modifiers based on gameplay state.

### 6. ExtraBolt Visual Distinction

Extra bolts (spawned by multi-bolt effects like Split Decision) also go through the builder with the same `BoltDefinition`. Visual distinction is applied at spawn time via initial modifiers:

- `AddModifier(GlowIntensity(0.7))` -- slightly dimmer than primary
- `AddModifier(TrailLength(0.6))` -- shorter trail

These are inserted by the spawning system (in effect/) when it calls the bolt builder, not by the builder itself. The builder just attaches whatever the definition says. The spawning system adds the distinguishing modifiers after spawn.

### 7. PhantomBolt Visual

Phantom bolts (from Phantom Bolt evolution) use a separate `BoltDefinition` RON file with different rendering block values:

- Different `color` (spectral/ghostly hue)
- No separate shader -- same `EntityGlowMaterial` with different uniforms

At spawn time, the phantom bolt spawning system adds modifiers:
- `AddModifier(AlphaOscillation { min: 0.3, max: 0.8, frequency: 3.0 })` -- ghostly fade
- `AddModifier(AfterimageTrail(true))` -- spectral afterimages

### 8. Bolt Trail Position Ring Buffer

A new component `BoltTrailHistory` in bolt/components/ that stores a fixed-size ring buffer of recent positions (e.g., 32 entries). Updated each frame by the trail update system. Used by both wake trail and afterimage trail rendering.

```
BoltTrailHistory {
    positions: [Vec2; 32],
    head: usize,
    count: usize,
}
```

This is a bolt-domain component, not a visuals type. It exists because trail rendering needs positional history, and only the bolt domain knows bolt positions.

### 9. BoltDefinition RON Updates

Update `assets/bolts/default.bolt.ron` with a rendering block:

```ron
rendering: Some((
    shape: Some(Circle),
    color: Some(Gold),
    glow: Some((
        core_brightness: (3.0),
        halo_radius: 2.5,
        halo_falloff: 4.0,
        bloom: (0.8),
    )),
    trail: Some(ShieldEnergy(
        width: 3.0,
        fade_length: 80.0,
        color: Gold,
        intensity: 2.0,
    )),
)),
```

The exact tuning values are approximate -- they will be adjusted during visual polish. The structure is what matters.

### 10. Quad Mesh for SDF Rendering

The bolt no longer uses a `Circle` mesh. Instead, the builder creates a unit quad (1x1) that the `entity_glow.wgsl` shader renders the SDF shape onto. The quad is scaled by the bolt's `Scale2D` component (which already handles radius). The shader's `half_extents` uniform is set to account for the glow halo extending beyond the core shape.

The quad should be slightly oversized (e.g., 2.5x the core radius) to give the halo room to render without clipping at the mesh edge.

## RON Changes (BoltDefinition rendering block)

### BoltDefinition struct addition

```rust
// In breaker-game/src/bolt/definition.rs
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BoltDefinition {
    // ... existing fields ...

    /// Visual rendering configuration. When present, the bolt builder
    /// uses these values to create EntityGlowMaterial and trail entities.
    /// When absent, falls back to color_rgb for a basic rendered bolt.
    #[serde(default)]
    pub rendering: Option<EntityVisualConfig>,
}
```

### RON file format (updated default.bolt.ron)

```ron
/* @[brickbreaker::bolt::definition::BoltDefinition] */
(
    name: "Bolt",
    base_speed: 720.0,
    min_speed: 360.0,
    max_speed: 1440.0,
    radius: 14.0,
    base_damage: 10.0,
    effects: [],
    color_rgb: (6.0, 5.0, 0.5),
    min_angle_horizontal: 5.0,
    min_angle_vertical: 5.0,
    rendering: Some((
        shape: Some(Circle),
        color: Some(Gold),
        glow: Some((
            core_brightness: (3.0),
            halo_radius: 2.5,
            halo_falloff: 4.0,
            bloom: (0.8),
        )),
        trail: Some(ShieldEnergy(
            width: 3.0,
            fade_length: 80.0,
            color: Gold,
            intensity: 2.0,
        )),
    )),
)
```

The `rendering` field is `Option<EntityVisualConfig>`:
- `None` or omitted: builder falls back to legacy `color_rgb` -> `ColorMaterial` path (backward compatible for tests)
- `Some(config)`: builder creates `EntityGlowMaterial` from shape+color+glow, spawns trail entity from trail config

## Builder Changes (what the bolt builder attaches)

### Visual typestate marker changes

```rust
// Replace current Rendered marker:
pub struct Rendered {
    pub(crate) material: Handle<EntityGlowMaterial>,
    pub(crate) mesh: Handle<Mesh>,  // unit quad, not Circle
    pub(crate) trail_config: Option<Trail>,
}
```

### .rendered() transition changes

The `.rendered()` method changes signature:

```rust
// Before (current):
pub fn rendered(self, meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>) -> ...

// After:
pub fn rendered(self, meshes: &mut Assets<Mesh>, materials: &mut Assets<EntityGlowMaterial>) -> ...
```

Inside `.rendered()`:
1. Read `EntityVisualConfig` from `OptionalBoltData` (set during `.definition()`)
2. If rendering config exists: map Shape -> `shape_type`, Hue -> `color` Vec4, GlowParams -> uniform fields
3. Create `EntityGlowUniforms` with these values (dissolve=0, squash=1/1, alpha=1, rotation=0, spike_count=0)
4. Create `EntityGlowMaterial` and add to `Assets<EntityGlowMaterial>`
5. Create a unit quad mesh (oversized for halo) and add to `Assets<Mesh>`
6. Store trail config for spawn-time child entity creation
7. If no rendering config: create a default material from `color_rgb` fallback

### Terminal spawn() changes

In `spawn()` for `Rendered` variants, after spawning the core entity:
1. Insert `Mesh2d(quad_mesh)`, `MeshMaterial2d(entity_glow_material)`, `GameDrawLayer::Bolt`
2. Insert `ModifierStack::default()` (so the modifier system can apply changes later)
3. If `trail_config` is `Some(trail)`: spawn a trail child entity based on the trail variant
4. Insert `BoltTrailHistory::default()` if any trail is configured

### Callers of .rendered() must update

All systems that call `.rendered(meshes, materials)` must update their system params:
- `ResMut<Assets<ColorMaterial>>` -> `ResMut<Assets<EntityGlowMaterial>>`
- The mesh param stays as `ResMut<Assets<Mesh>>`

Key callers to update:
- `spawn_initial_bolt` (in state/run/node/ or wherever bolt spawning lives)
- Any evolution/effect that spawns bolts with `.rendered()`

### .definition() stores visual config

The `.definition()` method on the builder already reads from `BoltDefinition`. Extend it to also store the `EntityVisualConfig` in `OptionalBoltData`:

```rust
optional.visual_config = def.rendering.clone();
```

Add `visual_config: Option<EntityVisualConfig>` to `OptionalBoltData`.

### build() vs spawn() for Rendered

`build()` returns a bundle but cannot spawn child entities (trails). For rendered bolts with trails, `spawn()` is required. `build()` attaches the material and mesh but not the trail. This is acceptable because `build()` is primarily used in tests where trails are not needed.

## What NOT to Do

- Do NOT create an `AttachVisuals` message -- builders attach directly
- Do NOT create a recipe system or `ExecuteRecipe` message -- event VFX (spawn flash, death streak, expiry implosion) are deferred to 5k (bump grade and failure VFX)
- Do NOT implement the modifier computation system that reads `ModifierStack` and updates `EntityGlowMaterial` uniforms -- that is 5j (dynamic visuals)
- Do NOT implement temperature palette shifts on the bolt -- that is 5j
- Do NOT implement spawn/death/expiry VFX particles -- those are 5k (bump/failure VFX) and require 5c (particle crate)
- Do NOT modify the visuals/ domain types -- those are built in 5e and consumed here
- Do NOT implement aura on bolt -- bolts do not have auras (that is breaker-specific, 5g)
- Do NOT implement the `entity_glow.wgsl` shader -- that is built in 5e
- Do NOT modify `EntityGlowMaterial` or `TrailRibbonMaterial` structs -- those are defined in 5e
- Do NOT implement damage boost visual state -- that is driven by chip effects sending `AddModifier`, not by this phase
- Do NOT change headless bolt behavior -- tests using `.headless()` must continue to work unchanged

## Dependencies

### Requires (must be complete before starting)

- **5e (visuals/ domain + entity shader)**: Provides `Shape`, `Hue`, `GlowParams`, `Trail`, `EntityVisualConfig`, `EntityGlowMaterial`, `EntityGlowUniforms`, `TrailRibbonMaterial`, `ModifierStack`, `VisualModifier`, `ModifierKind`, `SetModifier`/`AddModifier`/`RemoveModifier` messages, `entity_glow.wgsl` shader, `trail_ribbon.wgsl` shader, additive blend `specialize()` helper, `VisualsPlugin`

### Independent of (can run in parallel)

- **5c (rantzsoft_particles2d)**: Bolt visuals do not use particles (particle-based bolt VFX are in 5k)
- **5d (rantzsoft_postprocess)**: Bolt visuals do not use screen effects
- **5g (breaker visuals)**: Different domain, no dependency
- **5h (cell visuals)**: Different domain, no dependency
- **5i (walls & background)**: Different domain, no dependency

### Required by (blocks these phases)

- **5j (dynamic visuals)**: Needs bolt entities with `ModifierStack` and `EntityGlowMaterial` to drive modifier computation
- **5k (bump grade VFX)**: Needs bolt rendered as SDF orb for spawn/death/expiry VFX to layer onto
- **5l (combat effect VFX)**: Needs bolt visual state for effect-specific VFX (piercing beam, shockwave origin, etc.)

## Verification

### Functional

- Bolt renders as SDF energy orb (bright core + softer halo) via `EntityGlowMaterial` and `entity_glow.wgsl`
- Bolt core blooms (HDR > 1.0 values visible through Bevy's bloom)
- Wake trail child entity spawns when definition has `trail` configured
- Wake trail follows the bolt and its length scales with bolt speed
- Trail despawns automatically when bolt despawns (parent-child relationship)
- Serving bolt has dimmer core and no trail (via `sync_bolt_visual_modifiers` sending `SetModifier`)
- Piercing bolt gains spike count modifier when piercing effect is active
- Lifespan bolt dims and flickers as timer expires
- ExtraBolt and PhantomBolt have distinguishing modifiers applied at spawn
- `BoltTrailHistory` ring buffer correctly records recent positions

### Backward compatibility

- All existing bolt tests pass (headless bolts unchanged)
- `BoltDefinition` parses with and without `rendering` field (serde default)
- Existing `default.bolt.ron` parses correctly with the new `rendering` field added
- Builder typestate pattern still enforces the same compile-time constraints
- `color_rgb` field still works as fallback when `rendering` is absent

### Code quality

- `cargo all-dclippy` clean
- `cargo all-dtest` clean
- No new warnings
- Trail systems and `sync_bolt_visual_modifiers` have unit tests with concrete values
- `BoltTrailHistory` ring buffer has unit tests (insert, wrap-around, iteration)
- RON deserialization tests for `BoltDefinition` with rendering block

## Status
`[NEEDS DETAIL]` — blocked on 5e API design (how builders consume EntityVisualConfig, how trail entities are spawned, how modifiers are sent)
