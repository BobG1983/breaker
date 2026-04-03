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
    // ── Screen effects ──
    pub shake_multiplier: f32,             // 0.0 = disabled, 1.0 = default, 2.0 = max
    pub chromatic_multiplier: f32,         // chromatic aberration scaling

    // ── Bloom ──
    pub bloom_intensity: f32,              // Bloom.intensity — overall bloom strength
    pub bloom_low_frequency_boost: f32,    // Bloom.low_frequency_boost — wide glow vs tight halo
    pub bloom_composite_mode: BloomCompositeMode, // Bloom.composite_mode — Additive or EnergyConserving
    pub bloom_prefilter_threshold: f32,    // Bloom.prefilter.threshold — HDR cutoff for bloom

    // ── CRT overlay ──
    pub crt_enabled: bool,                 // CRT overlay toggle
    pub crt_intensity: f32,                // CRT scanline strength

    // ── Grid ──
    pub grid_line_spacing: f32,            // pixels between grid lines
    pub grid_line_thickness: f32,          // pixel width
    pub grid_glow_intensity: f32,          // overall grid brightness
}
```

**Bloom application**: The crate owns a system (in Update) that reads `VfxConfig` and applies bloom settings to the camera's `Bloom` component:

```rust
fn sync_bloom_config(
    config: Res<VfxConfig>,
    mut query: Query<&mut Bloom, With<Camera2d>>,
) {
    if !config.is_changed() { return; }
    for mut bloom in &mut query {
        bloom.intensity = config.bloom_intensity;
        bloom.low_frequency_boost = config.bloom_low_frequency_boost;
        bloom.composite_mode = config.bloom_composite_mode;
        bloom.prefilter.threshold = config.bloom_prefilter_threshold;
    }
}
```

This means the game controls bloom entirely through `VfxConfig` — no direct `Bloom` component manipulation needed.

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

**Game-side implementation**: The game implements `VfxLayer` on the same `GameDrawLayer` enum that already implements `DrawLayer` for `rantzsoft_spatial2d`. The `GameDrawLayer` enum will be expanded with VFX-specific variants (particles, primitives, trails, HUD overlay) as part of Phase 5. The concrete z-values are a game-side concern — not specified by the crate architecture. The crate only requires that the game provides values via the trait; the game decides the full draw order.

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

## Trail Ring Buffer — Dynamic Capacity

ShieldEnergy trails use a ring buffer of world-space positions, sampled every frame in PostUpdate (after `TransformSystems::Propagate`).

**Buffer capacity is dynamic**, driven by the `TrailLength` modifier:

```rust
#[derive(Component)]
pub struct TrailRingBuffer {
    pub positions: VecDeque<Vec2>,
    pub capacity: usize,           // current max samples (changes with modifier)
    pub base_capacity: usize,      // default capacity from Trail config (e.g., 32)
}
```

- `TrailLength(1.0)` = base capacity (e.g., 32 samples)
- `TrailLength(2.0)` = 64 samples (doubled)
- `TrailLength(0.5)` = 16 samples (halved)
- Capacity changes are applied by resizing the `VecDeque` — if shrinking, oldest samples are dropped. If growing, new capacity is available for future samples.
- Minimum capacity: 2 (need at least 2 points for a line). Maximum: 256 (hard cap to prevent unbounded growth).

**Vertex generation**: 2 vertices per sample (left/right of velocity tangent, offset by half trail width). At 32 samples = 64 vertices per trail. At max 256 samples = 512 vertices — still trivial for modern GPUs.

**Alpha curve**: vertex alpha = `(index as f32 / visible_count as f32)`. Head = 1.0, tail = 0.0. The `TrailLength` modifier changes how many samples exist, which naturally changes the visual length.

**Afterimage trails** have similar dynamic pool sizing — `TrailLength` modifier scales the number of active sprite copies in the pool (base = `copy_count` from config, scaled by modifier).

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

**Lifecycle:** SquashStretch is a brief effect. The game sends `SetModifier { modifier: SquashStretch { ... }, source: "bump_pop", duration: Some(0.05) }`. The crate auto-removes it after 0.05s (~3 frames). No game-side frame counting needed.

## Aura Material Consolidation

Aura rendering uses a **single `AuraMaterial`** type with a `variant: u32` uniform, rather than separate Material2d types per Aura variant. One `Material2dPlugin::<AuraMaterial>` registration. The WGSL shader uses a `switch` on `variant` to select the rendering algorithm (ShieldShimmer=0, TimeDistortion=1, PrismaticSplit=2). The material carries a union of all variant parameters — unused params are ignored per-variant.

Same approach as entity_glow's `shape_type` uniform. At 3 variants, GPU branch divergence is negligible.

## Trail Rendering — Three Distinct Techniques

Unlike auras, trail variants use **different rendering techniques** and cannot share a single material:

| Trail Variant | Technique | Material |
|--------------|-----------|----------|
| ShieldEnergy | Mesh ribbon (TriangleStrip), ring buffer of positions | Custom `TrailRibbonMaterial` (premultiplied-alpha additive) |
| Afterimage | Pre-spawned sprite entity pool, repositioned each frame | Standard `Sprite` with alpha fade |
| PrismaticSplit | 3 overlapping ShieldEnergy-style ribbons with RGB tint | Same `TrailRibbonMaterial` × 3 |

Each trail type has its own spawning and update logic. The `Trail` enum selects which technique to use at `AttachVisuals` time.

### TrailRibbonMaterial — Alpha-Weighted Additive

Same blend mode as `ParticleMaterial`: `src_factor: SrcAlpha, dst_factor: One`. The blend unit does the alpha multiply — no manual premultiplication in the shader.

```
Output = src.rgb * src.a + dst.rgb
```

- `alpha = 1.0` → full light contribution (additive, overlapping trails brighten)
- `alpha = 0.0` → zero contribution (invisible, not black)
- HDR values > 1.0 produce bloom naturally

Vertex alpha along the ribbon controls fade: head = 1.0, tail = 0.0. The shader just outputs color + alpha normally.

## Deferred Despawn for Death VFX

Entities with destruction VFX (Disintegrate, Split, Fracture) must remain alive for the death recipe duration. On entity death:

1. **Remove physics/interaction components** immediately — `CollisionLayers`, `Aabb2D`, health components, any component that causes the entity to participate in collisions or take damage. The entity becomes inert.
2. **Keep spatial + visual components** — `Position2D`, `Scale2D`, `Transform`, mesh, material, and any rendering components. The death recipe needs these to render the destruction effect at the correct position and scale.
3. **Fire the death recipe** via `ExecuteRecipe` with `source: Some(entity)`.
4. **Listen for `RecipeComplete`** and despawn the entity when it arrives.

No `DeferredDespawn` component or crate-side timer needed — the recipe lifecycle messages handle this. The game domain strips physics components and fires the recipe; the crate emits `RecipeComplete` when the destruction VFX finishes; the game domain despawns the husk. See Recipes doc — Recipe Lifecycle Messages.

## SDF Quad Oversizing

The SDF quad must be large enough to contain the full glow halo. Oversizing multiplier: **2.0× the entity's `Scale2D`** in each dimension. This means a 60×20 entity gets a 120×40 quad. The extra space is where the exponential halo falloff renders — fragments far from the shape boundary output near-zero alpha (additive black = no contribution).

At 2.0×, the halo has ~3 falloff-lengths to decay. If a specific entity needs a wider halo (e.g., heavy glow modifier), the multiplier can be increased via a material uniform override.

## Anchored Primitive Retraction

When one entity of a two-entity anchored primitive (AnchoredBeam, AnchoredArc) despawns while the other still exists, the primitive **retracts** rather than vanishing instantly.

**Retraction behavior:**
1. The endpoint tracking the despawned entity begins lerping toward the surviving entity's position over ~0.1s.
2. The lerp uses `Time<Virtual>` (slows with slow-motion).
3. When the retracted endpoint reaches the surviving entity (or 0.1s elapses), the anchored primitive despawns.
4. If both entities despawn in the same frame, the anchored primitive despawns immediately — no retraction.

**Implementation:**

```rust
#[derive(Component)]
pub struct AnchoredEndpoints {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub retraction: Option<RetractionState>,
}

pub struct RetractionState {
    pub dead_endpoint: Vec2,        // last known position of despawned entity
    pub surviving_entity: Entity,   // which entity is still alive
    pub progress: f32,              // 0.0 → 1.0 over retraction_duration
    pub retraction_duration: f32,   // 0.1s default
}
```

The update system in PostUpdate checks:
- If both entities exist → normal tracking (read `GlobalTransform` from both)
- If one is missing and `retraction` is None → begin retraction (snapshot last known position)
- If retracting → lerp `dead_endpoint` toward surviving entity position, advance progress
- If progress >= 1.0 or surviving entity also despawns → despawn primitive

**Single-entity anchors** (AnchoredRing, AnchoredDistortion, AnchoredGlowMotes) still despawn immediately when their tracked entity despawns — no retraction needed for single-entity effects.

## ElectricArc Rendering

`ElectricArc` is **not a particle system** despite being listed under particle primitives. It is a **segmented line mesh** with per-frame jitter:

1. Compute N line segments between start and end positions (N ≈ distance / segment_length, typically 8-12 segments)
2. Offset each interior vertex perpendicular to the line by `random(-jitter, jitter)` — regenerated each frame for flicker
3. Render as a `Mesh2d` with `TriangleStrip` topology (thin ribbon along the jagged path)
4. Custom `ArcMaterial` (additive blend) with HDR color

The `ElectricArc` PrimitiveStep/message specifies `jitter`, `flicker_rate`, `hdr`, `color`, `lifetime`. The emitter system spawns one arc entity (not multiple particles). Flicker via regenerating vertex offsets at `flicker_rate` Hz.

## GlitchText Overlay Sizing

The GlitchText child overlay entity must match the parent Text2d bounds. On spawn:

1. Spawn Text2d entity with the text content
2. Wait one frame for Bevy's text layout to compute `TextLayoutInfo`
3. Read `TextLayoutInfo.size` from the Text2d entity to get the computed text bounds
4. Spawn the overlay child entity with a quad mesh sized to match

Since we use a monospace font (Space Mono Bold), estimate bounds from `text.len() * char_width` using a known character width for the font size. No frame delay needed. The char_width for Space Mono Bold at a given font size is a fixed constant (measure once, use everywhere).

## Recipe Direction Resolution

When a recipe contains `Beam(direction: Forward)` and `ExecuteRecipe` provides `direction: Some(vec2)`:

1. `Forward` resolves from the **source entity's `Velocity2D`** direction (normalized), not from `ExecuteRecipe.direction`.
2. `ExecuteRecipe.direction` is used for **particle spray angle** (e.g., `SparkBurst` emit direction) and any step that doesn't specify its own direction.
3. If both a step direction (`Forward`) and a message direction exist, the **step direction wins** — it's more specific.
4. `Forward` fallback when source has no velocity: uses `ExecuteRecipe.direction` if Some, else defaults to `Vec2::Y` (up).

## Multiple ActiveRecipe Convention

Multiple recipes can run on the same entity simultaneously without conflict. The convention:

- **Fire-and-forget recipes** (bump flash, cell hit, spark bursts): Don't check `ActiveRecipe`. Let them run independently. They're short-lived and overlap naturally via additive blending.
- **Persistent recipes** (gravity well, tether beam, ramping ring): These are the ones tracked by `ActiveRecipe`. Game should `CancelRecipe` the old one before firing a new persistent recipe on the same entity.

Rule of thumb: if the recipe has anchored primitives or repeating phases, it's persistent — cancel before replacing. If it's a one-shot effect (<0.5s), fire freely.

## expected_nodes_per_act

Value: `5` (4 normal nodes + 1 boss per act). Temperature goes from 0.0 to 1.0 over 5 nodes. For infinite runs, temperature cycles: `temperature = ((node_index % nodes_per_act) as f32 / nodes_per_act as f32)`. The act length and cycling behavior are tuned in Phase 7 with the run progression system.

## Absorbed from fx/

The current `fx/` domain is eliminated:
- Transitions → `screen/transition/`
- Fade-out, punch scale → `rantzsoft_vfx` (generic animation primitives)
- `FxPlugin` removed from `game.rs`
