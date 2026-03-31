# rantzsoft_vfx Crate

A new game-agnostic workspace crate at `rantzsoft_vfx/` providing `RantzVfxPlugin`. Zero game vocabulary, usable by any 2D Bevy game.

## What Lives in rantzsoft_vfx

**Primitives & Rendering:**
- Particle system: CPU particle simulation, emitter components, Material2d with additive blending, HDR color support (see [particles.md](particles.md))
- Geometric primitives: ExpandingRing, Beam, EnergyRing — message types + spawn/tick/cleanup systems
- Particle primitives: SparkBurst, ShardBurst, GlowMotes, ElectricArc, TrailBurst
- Screen effects: ScreenShake, ScreenFlash, RadialDistortion, ChromaticAberration, Desaturation, SlowMotion — with camera-targeting API
- Shape rendering: Shape enum → mesh generation + entity_glow shader attachment
- Aura rendering: Aura enum → per-variant aura shader, RON-driven params
- Trail rendering: Trail enum → per-variant trail shader, RON-driven params
- All concept shaders (.wgsl files)

**Systems:**
- `AttachVisuals` message handler: receives entity + `EntityVisualConfig`, attaches mesh/material/shaders/aura/trail
- Modifier system: receives `SetModifier`/`AddModifier`/`RemoveModifier` messages, maintains computed visual state per entity with diminishing returns
- Recipe system: `RecipeStore` resource, RON recipe loading pipeline, phased timeline dispatch with `PhaseGroup` entity tracking
- Post-processing pipeline: bloom tuning, distortion buffer (16-source fixed array), chromatic aberration, desaturation, flash, CRT, vignette

**Types:**
- `Hue` enum: CSS named colors (~148 variants + Custom)
- `Shape` enum: geometric shapes (+ Custom)
- `Aura` enum: aura shader + params merged (ShieldShimmer, TimeDistortion, PrismaticSplit)
- `Trail` enum: trail shader + params merged (ShieldEnergy, Afterimage, PrismaticSplit)
- `GlowParams`, `BloomIntensity`, `HdrBrightness`, `EmissiveStrength`: typed visual parameters
- `EntityVisualConfig`: the struct `AttachVisuals` carries
- `VisualModifier` enum, `ModifierKind`, `ModifierConfig`
- `Recipe`, `Phase`, `PhaseTrigger`, `PrimitiveStep`, `RecipeStore`
- `ShakeTier`, `Direction`, `EntityRef` enums
- `VfxConfig`: runtime-mutable crate configuration resource

## VfxConfig Resource

```rust
/// Global VFX configuration. Inserted by the game, mutable at runtime.
#[derive(Resource, Clone, Debug)]
pub struct VfxConfig {
    pub shake_multiplier: f32,      // 0.0 = disabled, 1.0 = default, 2.0 = max
    pub bloom_intensity: f32,       // bloom strength
    pub crt_enabled: bool,          // CRT overlay toggle
    pub crt_intensity: f32,         // CRT scanline strength
    pub chromatic_multiplier: f32,  // chromatic aberration scaling
    // ... extensible
}
```

**Ownership split**: The crate defines the type and reads it every frame. The game inserts it at startup (from `GraphicsDefaults` RON via `rantzsoft_defaults`) and owns all mutations — loading, debug menu sliders, settings screen, accessibility overrides. The crate never writes to `VfxConfig`; it is a read-only input from the game's perspective.

## What Lives in the Game (NOT in the crate)

There is no `rendering/` or `graphics/` game domain. Game-specific visual concerns are dispersed to the domains that own the relevant game state. See [communication.md — Domain Restructuring](communication.md#domain-restructuring) for the full map.

| Concern | Game Domain | Why There |
|---------|-------------|-----------|
| `GraphicsConfig` resource (CRT, bloom, shake multiplier) | `shared/` | Game-wide configuration |
| `RunTemperature` palette | `run/` | Reads RunState.node_index |
| Danger vignette logic | `run/` | Reads timer + lives, sends VFX messages |
| Diegetic HUD (timer wall, life orbs, node progress) | `screen/playing/hud/` | Gameplay screen UI |
| Transitions (Flash, Sweep, Glitch, Collapse/Rebuild) | `screen/transition/` | Screen lifecycle |
| Per-screen UI (chip select, menus, pause, run-end) | `screen/<screen_name>/` | Per-screen systems |
| Fade-out, punch scale animation | `rantzsoft_vfx` | Generic animation primitives |

## Camera-Targeting API

Screen effect messages carry the camera `Entity` explicitly. The game passes it when sending. When `ExecuteRecipe` dispatches screen effect steps from a recipe, the camera from the `ExecuteRecipe` message is passed through.

This enables:
- Main game camera gets gameplay screen effects
- A sub-render from another camera could have its own effects
- Camera can be changed at runtime

## Headless Mode

`RantzVfxPlugin::headless()` registers message types and fires completion callbacks but skips all rendering systems. Messages are accepted but not processed visually. `TransitionComplete` still fires. See [headless.md](headless.md).

## VfxLayer Trait (Z-Ordering)

The crate needs z-layer values for spawned VFX entities but can't know the game's draw layer scheme. Solution: a trait the game implements.

```rust
/// Game implements this to tell the crate where to place VFX entities in z-space.
pub trait VfxLayer: Resource + Send + Sync + 'static {
    fn particles_z(&self) -> f32;
    fn primitives_z(&self) -> f32;       // expanding rings, beams, discs
    fn trail_z(&self) -> f32;
    fn aura_z_offset(&self) -> f32;      // relative to parent entity
    fn hud_overlay_z(&self) -> f32;      // timer wall, etc.
}
```

The game implements `VfxLayer` on a type and passes it to `RantzVfxPlugin::new(layer_resource)`. The crate reads it when spawning VFX entities. Same pattern as `rantzsoft_spatial2d`'s `DrawLayer` trait.

## Scale → Shader (SDF Aspect Ratio)

The entity_glow SDF shader needs the entity's aspect ratio to compute correct SDF dimensions. The `AttachVisuals` handler reads the entity's `Transform.scale` (set by the game from `Scale2D`) and bakes `half_extents` into the material uniform. The shader normalizes UV coordinates using these half-extents:

```wgsl
let uv = (in.uv * 2.0 - 1.0) * material.half_extents;
let d = sdf_shape(uv, shape_type);
```

The crate doesn't reference `Scale2D` directly (game-agnostic). It reads standard Bevy `Transform.scale` which the game's spatial plugin already synchronizes from `Scale2D`.

## Trail Entity Cleanup

Trail entities are top-level (not children of the tracked entity). They need explicit cleanup when the source entity despawns.

Each trail entity stores a `TrailSource(Entity)` component. A crate system runs each tick: if `world.get_entity(source)` returns None, despawn the trail. Same polling pattern as anchored primitives.

## Recipe Timing

Recipe phase timing (`Delay(f32)`, `RepeatConfig.interval`) uses `Time<Virtual>`. VFX slows down with gameplay during slow-motion. This is correct — slow-motion should affect visual timing, not just physics.

Exception: `DilationRamp` (the slow-motion ramp itself) uses `Time<Real>` to avoid recursive slowdown. See [slow_motion.md](slow_motion.md).

## Recipe Position vs Entity Position

For recipes with both a `position` field and `source`/`target` entities:
- **Non-anchored steps** use `message.position + phase.offset` — the position from `ExecuteRecipe`.
- **Anchored steps** ignore `message.position` — they read `GlobalTransform` from the Source/Target entities each tick.

Both can coexist in the same recipe. A recipe can have non-anchored expanding rings at the message position AND an anchored beam tracking entity positions.

## ActiveRecipe Lifecycle

`ActiveRecipe(Entity)` is a **crate-owned component** inserted on the source entity (if `ExecuteRecipe.source` is Some). It points to the `RecipeExecution` entity.

- On `CancelRecipe`: the crate despawns the `RecipeExecution` and removes `ActiveRecipe` from the source entity.
- On source entity despawn: the `RecipeExecution` self-despawns (it polls `world.get_entity(source)` each tick). `ActiveRecipe` is cleaned up by Bevy's standard entity despawn.
- **Multiple recipes per entity**: `ActiveRecipe` holds the *most recent* recipe. If a new recipe fires on the same source, it overwrites `ActiveRecipe`. The old `RecipeExecution` continues to completion independently (it's not cancelled — only the component pointer updates). If cancellation of the old recipe is needed, the game must cancel it before firing the new one.

## SquashStretch Modifier

`SquashStretch { x_scale, y_scale }` is applied as a **shader uniform**, NOT a `Transform.scale` change. The SDF shader multiplies UV coordinates by the squash/stretch factors before computing the SDF distance. This changes the visual shape without affecting the entity's collision AABB or spatial footprint.

## Aura Material Consolidation

Aura rendering uses a **single `AuraMaterial`** type with a `variant: u32` uniform, rather than separate Material2d types per Aura variant. One `Material2dPlugin::<AuraMaterial>` registration. The WGSL shader uses a `switch` on `variant` to select the rendering algorithm (ShieldShimmer=0, TimeDistortion=1, PrismaticSplit=2). The material carries a union of all variant parameters — unused params are ignored per-variant.

Same approach as entity_glow's `shape_type` uniform. At 3 variants, GPU branch divergence is negligible. Similarly, trail rendering uses a single `TrailMaterial` where applicable.

## Absorbed from fx/

The current `fx/` domain is eliminated:
- Transitions → `screen/transition/`
- Fade-out, punch scale → `rantzsoft_vfx` (generic animation primitives)
- `FxPlugin` removed from `game.rs`
