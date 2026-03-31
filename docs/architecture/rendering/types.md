# Types

All types defined in `rantzsoft_vfx` unless noted otherwise.

## Hue Enum

Hand-written CSS named colors (~148 widely-known, web-standard colors) + `Custom(f32, f32, f32, f32)`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum Hue {
    // CSS named colors (~148 standard)
    AliceBlue, AntiqueWhite, Aqua, Aquamarine, Azure,
    // ... (full set)
    White, WhiteSmoke, Yellow, YellowGreen,

    Custom(f32, f32, f32, f32),  // RGBA linear
}
```

- `impl From<Hue> for bevy::color::Color`
- `impl From<bevy::color::Color> for Hue` (Custom for non-matching)
- `impl From<Hue> for LinearRgba` (shader uniforms)

**RON files use CSS color names directly** (e.g., `color: CadetBlue`, `color: Gold`, `color: MediumSeaGreen`). No semantic aliases in RON. Games may define `const` aliases in Rust code for readability: `const AEGIS_ACCENT: Hue = Hue::CadetBlue;`

## Shape Enum

```rust
pub enum Shape {
    Rectangle, RoundedRectangle, Hexagon, Octagon,
    Circle, Diamond, Shield, Angular, Crystalline,
    Custom(CustomShape),
}

/// Custom convex polygon defined by vertices. Centered at origin, CCW winding.
pub struct CustomShape {
    pub vertices: Vec<Vec2>,
}
```

**Research**: See [research/mesh-generation.md](research/mesh-generation.md) for full Bevy 0.18.1 API details and vertex coordinates.

### SDF Rendering (no mesh generation for visual shape)

Entities are rendered as **SDF-on-quad**: one oversized quad per entity, the fragment shader computes the SDF distance from the shape boundary to determine core fill vs halo glow. The Shape enum selects which SDF function to evaluate — no mesh vertex generation needed for rendering.

| Variant | SDF Function | Notes |
|---------|-------------|-------|
| Rectangle | `sdBox(uv, half_extents)` | Standard box SDF |
| RoundedRectangle | `sdRoundedBox(uv, half_extents, corner_radius)` | Box SDF with rounded corners |
| Hexagon | `sdHexagon(uv, radius)` | 6-sided regular polygon SDF |
| Octagon | `sdRegularPolygon(uv, radius, 8)` | 8-sided regular polygon SDF |
| Circle | `sdCircle(uv, radius)` | Standard circle SDF |
| Diamond | `sdRhombus(uv, half_diags)` | Rhombus SDF |
| Shield | Custom SDF (wide convex top, tapered bottom) | Approximate with sdBox + smoothstep or custom polygon SDF |
| Angular | Custom SDF (chevron/arrowhead) | Polygon SDF from vertex list |
| Crystalline | Custom SDF (irregular facets) | Polygon SDF from vertex list |
| Custom | `sdPolygon(uv, vertices)` | Arbitrary convex polygon SDF |

**SDF advantages**: one draw call per entity, infinitely smooth at any scale, all modifiers (SpikeCount, CoreBrightness, HaloRadius) are uniform changes — no vertex regeneration. SDF math is trivial at our entity sizes (30-80px).

**Shape is just "which SDF function"** — it carries no size/dimension parameters. SDF dimensions come from the entity's `Scale2D` component (already used by the spatial system). The entity_glow shader reads scale from the mesh transform and normalizes UV coordinates accordingly. A `Shape::Rectangle` on a 60x20 entity uses `sdBox(uv, vec2(0.5, 0.167))` — half-extents derived from the aspect ratio.

**SDF function selection**: single `entity_glow.wgsl` shader with an integer `shape_type` uniform. WGSL `switch` statement selects the SDF function. At 50-100 entities with branch divergence only at entity boundaries, GPU performance impact is negligible.

SDF primitives from the [munrocket WGSL gist](https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4), included in `rantzsoft_vfx/assets/shaders/sdf.wgsl`.

**Mesh generation is still used for**: collision AABBs (already handled by physics), trail ribbon meshes (TriangleStrip), fracture shard quads, particle quads. Just not for entity visual shapes — those are SDF.

## Aura Enum

Ambient visual effect around an entity. Shader + params merged — each variant maps to a `.wgsl` shader.

```rust
pub enum Aura {
    ShieldShimmer { pulse_speed: f32, intensity: f32, color: Hue, radius: f32 },
    TimeDistortion { ripple_frequency: f32, echo_count: u32, intensity: f32, color: Hue },
    PrismaticSplit { refraction_intensity: f32, spectral_spread: f32, color: Hue },
}
```

**Research**: See [research/aura-rendering.md](research/aura-rendering.md) for full Bevy 0.18.1 API details and shader techniques.

### Aura Rendering Technique

Auras are **child mesh entities** of the gameplay entity. Each is a `Circle` mesh slightly larger than the parent (1.4x radius), with a custom `Material2d` implementing additive blending via `specialize()`. Placed at `Transform::from_xyz(0.0, 0.0, -0.5)` (behind parent).

All aura types use a **single `AuraMaterial`** type with a `variant: u32` uniform. One `Material2dPlugin::<AuraMaterial>` registration. The WGSL shader switches on variant to select the rendering algorithm. The material carries a union of all variant parameters (unused params ignored per-variant). Fragment shaders use SDF distance from entity center + time-based animation:

- **ShieldShimmer**: sine-wave edge ripple + noise-driven pulsation + optional scanline sweep
- **TimeDistortion**: concentric radial wave rings with phase-offset echoes, earlier echoes brighter
- **PrismaticSplit**: per-channel UV offset simulating wavelength-dependent refraction (R bends least, B most)

**WGSL import gotcha**: Use `#import bevy_sprite::mesh2d_view_bindings::globals` (NOT `bevy_pbr::mesh_view_bindings::globals`) for `globals.time` in 2D shaders. Material uniforms are `@group(2)`.

**Performance**: ~6 aura entities max (1 breaker + up to 5 bolts). ~0.0002ms per aura on mid-range hardware — negligible.

## Trail Enum

Motion trail behind a moving entity. Shader + params merged.

```rust
pub enum Trail {
    ShieldEnergy { width: f32, fade_length: f32, color: Hue, intensity: f32 },
    Afterimage { copy_count: u32, fade_rate: f32, color: Hue, spacing: f32 },
    PrismaticSplit { spectral_spread: f32, fade_length: f32 },
}
```

**Research**: See [research/trail-rendering.md](research/trail-rendering.md) for full Bevy 0.18.1 API details.

### Trail Rendering Techniques

**Critical**: Trail entities are **top-level, NOT children** of the moving entity. Trail vertices are in world space (historical positions). Making a trail a child would require inverse-transforming every vertex through the parent's transform each frame. Sample the entity's `GlobalTransform::translation()` in PostUpdate (after TransformSystems::Propagate).

| Variant | Technique | Implementation |
|---------|-----------|----------------|
| ShieldEnergy | **Mesh ribbon** (TriangleStrip) | Ring buffer of N world-space positions. Each frame: push new position, generate left/right vertex pairs with tapering width and fading alpha. Update mesh via `meshes.get_mut(&handle).insert_attribute(...)`. Use `RenderAssetUsages::default()` (MAIN_WORLD + RENDER_WORLD) for per-frame dynamic meshes. |
| Afterimage | **Pre-spawned entity pool** | N sprite entities (6-10 sufficient) at historical world-space positions, each with decreasing alpha via `sprite.color`. NOT instanced rendering — pre-spawn the pool, reposition + re-tint each frame. Avoids archetype churn. |
| PrismaticSplit | **3 overlapping ribbons** | Three ShieldEnergy-style mesh ribbons, each tinted a different spectral color (R/G/B), with slight perpendicular offset per channel. Three draw calls but zero shader work. (Or: single ribbon with custom Material2d shader mapping UV.x to spectral gradient.) |

**Bevy 0.18 gotcha**: `Mesh::new()` requires `RenderAssetUsages::default()` for meshes updated per-frame. Using `RENDER_WORLD` only causes panics on `insert_attribute()`.

## Typed Visual Parameters

Newtypes enforce correctness — can't mix bloom with brightness.

```rust
pub struct HdrBrightness(pub f32);   // values > 1.0 produce bloom
pub struct BloomIntensity(pub f32);
pub struct EmissiveStrength(pub f32);

pub struct GlowParams {
    pub core_brightness: HdrBrightness,
    pub halo_radius: f32,
    pub halo_falloff: f32,
    pub bloom: BloomIntensity,
}
```

## EntityRef Enum

Used by anchored and shape-destruction primitives in recipes to reference entities from the `ExecuteRecipe` message.

```rust
pub enum EntityRef {
    Source,  // from ExecuteRecipe.source
    Target,  // from ExecuteRecipe.target
}
```

RON uses `entity: Source` or `entity_a: Source, entity_b: Target`. Resolved at dispatch time.

## Direction Enum

Used by Beam primitive for recipe-authored beam directions.

```rust
pub enum Direction {
    N, S, E, W, NE, NW, SE, SW,
    Forward,   // in the entity's facing direction
    Backward,  // opposite facing direction
}
```

**Forward/Backward resolution**: derived from the source entity's `Velocity2D` component direction (normalized). If the entity has no velocity or zero velocity, Forward defaults to N (up). This is primarily used by bolts (beams fire along bolt trajectory). Breakers don't use directional beam recipes.

## ShakeTier Enum

```rust
pub enum ShakeTier {
    Micro,   // 1-2px, 1-2 frames
    Small,   // 3-5px, 3-4 frames
    Medium,  // 6-10px, 4-6 frames
    Heavy,   // 12-20px, 6-10 frames
}
```

## VisualModifier Enum

```rust
pub enum VisualModifier {
    // Multipliers (stacks with DR via AddModifier, absolute via SetModifier)
    TrailLength(f32),
    GlowIntensity(f32),
    CoreBrightness(f32),
    HaloRadius(f32),
    ShapeScale(f32),
    SpikeCount(u32),           // angular spikes on glow (piercing)

    // Color
    ColorShift(Hue),           // shift toward this hue
    ColorCycle { colors: [Hue; 4], speed: f32 },  // prismatic shimmer: cycle through up to 4 colors at speed (Entropy Engine). Fixed array avoids heap allocation in messages.

    // Transparency
    AlphaOscillation { min: f32, max: f32, frequency: f32 },  // pulsing transparency (Phantom Bolt)

    // Entity deformation (shader uniform, NOT Transform — doesn't affect collision AABB)
    SquashStretch { x_scale: f32, y_scale: f32 },  // brief scale distortion (Quick Stop, bump pop)

    // Trail variants
    AfterimageTrail(bool),     // enable afterimage copies at previous positions (Phantom Bolt, FlashStep)

    // Dynamic ring
    RotationSpeed(f32),        // orbital ring spin speed (Ramping Damage)
}
```

## ModifierKind Enum

Derived from `VisualModifier` variants — used as HashMap key in `ModifierConfig`.

```rust
pub enum ModifierKind {
    TrailLength, GlowIntensity, CoreBrightness, HaloRadius, ShapeScale, SpikeCount,
    ColorShift, ColorCycle, AlphaOscillation, SquashStretch, AfterimageTrail, RotationSpeed,
}
```
