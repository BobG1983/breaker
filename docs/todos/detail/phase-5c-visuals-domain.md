# 5c: visuals/ Domain + Entity Shader

## Summary

Create a new top-level domain `breaker-game/src/visuals/` with `VisualsPlugin`. This domain provides the game's visual vocabulary: type definitions (Hue, Shape, Aura, Trail, GlowParams, VisualModifier, ModifierStack, EntityVisualConfig), shaders (entity_glow SDF, additive blend Material2d, glitch_text overlay, holographic card), and shared WGSL utilities (SDF primitives, noise). Other domains (bolt/, cells/, breaker/, walls/) consume these types via their builders. The visuals domain does NOT know about Bolt, Breaker, Cell, or Wall — it only knows its own types, same relationship as effect/ provides infrastructure while other domains define their effects.

This phase delivers types and shaders only. No modifier computation system, no temperature palette application system, no runtime rendering consumers. Those come in 5j (dynamic visuals).

This phase also eliminates the old `fx/` domain. FadeOut and PunchScale move into visuals/.

## Context (why a game-side domain, not a rantzsoft crate)

The original plan (old 5f) placed Hue, Shape, Aura, Trail, VisualModifier, etc. in the `rantzsoft_vfx` monolithic crate. The revised architecture (see `docs/todos/detail/phase-5-rethink/architecture.md`) splits that crate into two focused game-agnostic crates (`rantzsoft_particles2d`, `rantzsoft_postprocess`) and puts all game-specific visual composition types in a game-side domain.

Why game-side:
- These types encode game design decisions: which shapes exist, which aura variants exist, which modifier variants exist. They change with the game, not with a reusable engine.
- RON data files reference these enums by variant name. The enums are part of the game's content vocabulary.
- Shaders like entity_glow embed game-specific SDF shape definitions (Shield, Angular, Crystalline are designed for specific breaker archetypes). They are not generic.
- The rantzsoft crates remain game-agnostic (zero game vocabulary) and reusable.

## What to Build

### 1. Hue Enum

~148 CSS named colors + `Custom(f32, f32, f32, f32)`. RON files reference CSS color names directly (e.g., `color: CadetBlue`, `color: Gold`).

Implements:
- `From<Hue> for bevy::color::Color`
- `From<bevy::color::Color> for Hue` (maps to `Custom` for non-matching values)
- `From<Hue> for LinearRgba` (for shader uniform conversion)

Unit tests:
- Round-trip conversion for all named variants
- `Custom` preserves exact values
- RON deserialization for representative named colors and Custom

### 2. Shape Enum

Geometric SDF selection. Each variant maps to an integer `shape_type` uniform in the entity_glow shader.

Unit tests:
- `shape_type_index()` method returns correct integer for each variant
- RON deserialization including `RoundedRectangle` with `corner_radius` field
- `Custom(CustomShape)` vertex count validation (max 16)

### 3. Aura Enum

Ambient visual effect variants, each with params. Rendered via single `AuraMaterial` with `variant` uniform selecting the algorithm.

Unit tests:
- RON deserialization with all param fields
- Default param values via `#[serde(default)]`

### 4. Trail Enum

Motion trail variants, each with params. Three distinct rendering techniques (ribbon, sprite pool, triple ribbon).

Unit tests:
- RON deserialization with all param fields
- Default param values

### 5. GlowParams and Typed Visual Parameters

Newtypes enforce correctness — cannot mix bloom with brightness.

Unit tests:
- Construction and field access
- `From<GlowParams>` conversions to shader uniform fields

### 6. EntityVisualConfig Struct

The composition struct that brings together shape, color, glow, aura, trail. Used in RON rendering blocks of entity definitions.

Unit tests:
- RON deserialization with all fields
- RON deserialization with optional fields omitted (serde defaults)

### 7. VisualModifier Enum and ModifierKind

12 modifier variants for runtime visual changes from chip effects. `ModifierKind` is the discriminant enum used as a key.

Unit tests:
- `ModifierKind::from(&VisualModifier)` mapping
- RON deserialization for each variant

### 8. ModifierStack Component

Component that tracks stacked modifiers with diminishing returns. Type definition and basic stack operations only — the computation system that applies these to materials is built in 5j.

Unit tests:
- Push/remove operations
- Stack count queries
- Diminishing returns curve computation (the math, not the system that applies it)

### 9. RunTemperature Resource

`RunTemperature(f32)` resource: 0.0 (cool) to 1.0 (hot). Type definition only — the system that updates it on node transitions and the systems that read it for palette application are built in 5j.

Unit tests:
- Construction, clamping to 0.0-1.0 range

### 10. TemperaturePalette Struct

Palette endpoints for grid, bloom, walls. Defined in RON via `GraphicsDefaults`. Type definition only.

Unit tests:
- RON deserialization
- Lerp between cool and hot endpoints at a given temperature value

### 11. Entity Glow Shader (entity_glow.wgsl)

The core entity rendering shader. SDF-on-quad: computes distance from shape boundary, exponential falloff for halo, HDR values > 1.0 for bloom.

Contents:
- SDF primitive functions (sdCircle, sdBox, sdRoundedBox, sdHexagon, sdRegularPolygon, sdRhombus, sdPolygon)
- Shape type switch statement (0=Rectangle through 9=Custom)
- Core fill + halo glow computation
- Spike modulation (for piercing modifier)
- Dissolve threshold (for death VFX — threshold 0.0 = off, no cost when disabled)
- SquashStretch UV distortion
- Alpha override
- Rotation angle

### 12. EntityGlowMaterial (Material2d)

Rust-side material struct with `AsBindGroup`, `ShaderType` uniforms, and `specialize()` for additive blending (`SrcAlpha + One`).

Unit tests:
- Material compiles (type assertion)
- Uniform alignment (ShaderType derives)

### 13. Additive Blend specialize() Pattern

Shared `specialize()` implementation for additive blending. Used by EntityGlowMaterial, AuraMaterial, TrailRibbonMaterial. Extract as a helper function to avoid duplication.

### 14. AuraMaterial

Single material type for all aura variants. Variant-switched in WGSL. Union of all variant params.

### 15. TrailRibbonMaterial

Material for ShieldEnergy and PrismaticSplit trail variants. Simple color + HDR intensity + alpha.

### 16. Glitch Text Shader (glitch_text.wgsl)

Material2d overlay for highlight moment typography. Scanlines, chromatic split, jitter blocks, additive composite.

Contents:
- Scanline band modulation
- RGB channel UV offset (chromatic split)
- Hash-based block displacement (jitter)
- Additive blend composite

### 17. GlitchMaterial

Rust-side material with uniforms: time, scanline_density, scanline_speed, chromatic_offset, jitter_intensity.

### 18. Holographic Shader (holographic.wgsl)

Material2d for Evolution-rarity chip card backgrounds. Prismatic foil shimmer based on UV + time.

Contents:
- Base color rendering
- Hue-shifting spectral overlay (UV position + time driven)
- Additive shimmer layer

### 19. HolographicMaterial

Rust-side material with uniforms: base_color, shimmer_speed, spectral_intensity, scan_line_frequency.

### 20. Shared WGSL Utilities

- `sdf.wgsl` — SDF primitive functions (imported by entity_glow.wgsl). Sources: munrocket WGSL gist + Inigo Quilez.
- `noise.wgsl` — Simplex noise 2D/3D (imported by entity_glow.wgsl for dissolve). Source: munrocket MIT-licensed WGSL gist.

### 21. FadeOut and PunchScale Migration

Move `FadeOut` component and `animate_fade_out` system from fx/ to visuals/. Move `PunchScale` component and `animate_punch_scale` system from fx/ to visuals/. Update all imports across the codebase (bolt/, breaker/, state/run/node/). Delete the fx/ domain entirely. Remove `FxPlugin` from `game.rs` and `lib.rs`. These systems register in `VisualsPlugin`.

### 22. VisualsPlugin

Plugin registration:
- Register `Material2dPlugin::<EntityGlowMaterial>`
- Register `Material2dPlugin::<AuraMaterial>`
- Register `Material2dPlugin::<TrailRibbonMaterial>`
- Register `Material2dPlugin::<GlitchMaterial>`
- Register `Material2dPlugin::<HolographicMaterial>`
- Register `animate_fade_out` and `animate_punch_scale` systems (migrated from FxPlugin)
- Init `RunTemperature` resource (default 0.0)
- No modifier computation systems yet (5j)
- No temperature palette application systems yet (5j)

## Type Definitions

### Hue (~148 CSS named colors + Custom)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub enum Hue {
    // Reds
    IndianRed, LightCoral, Salmon, DarkSalmon, LightSalmon, Crimson, Red,
    FireBrick, DarkRed,
    // Pinks
    Pink, LightPink, HotPink, DeepPink, MediumVioletRed, PaleVioletRed,
    // Oranges
    Coral, Tomato, OrangeRed, DarkOrange, Orange,
    // Yellows
    Gold, Yellow, LightYellow, LemonChiffon, LightGoldenrodYellow,
    PapayaWhip, Moccasin, PeachPuff, PaleGoldenrod, Khaki, DarkKhaki,
    // Purples
    Lavender, Thistle, Plum, Violet, Orchid, Fuchsia, Magenta,
    MediumOrchid, MediumPurple, RebeccaPurple, BlueViolet, DarkViolet,
    DarkOrchid, DarkMagenta, Purple, Indigo, SlateBlue, DarkSlateBlue,
    MediumSlateBlue,
    // Greens
    GreenYellow, Chartreuse, LawnGreen, Lime, LimeGreen, PaleGreen,
    LightGreen, MediumSpringGreen, SpringGreen, MediumSeaGreen, SeaGreen,
    ForestGreen, Green, DarkGreen, YellowGreen, OliveDrab, Olive,
    DarkOliveGreen, MediumAquamarine, DarkSeaGreen, LightSeaGreen,
    DarkCyan, Teal,
    // Blues
    Aqua, Cyan, LightCyan, PaleTurquoise, Aquamarine, Turquoise,
    MediumTurquoise, DarkTurquoise, CadetBlue, SteelBlue, LightSteelBlue,
    PowderBlue, LightBlue, SkyBlue, LightSkyBlue, DeepSkyBlue,
    DodgerBlue, CornflowerBlue, RoyalBlue, Blue, MediumBlue, DarkBlue,
    Navy, MidnightBlue,
    // Browns
    Cornsilk, BlanchedAlmond, Bisque, NavajoWhite, Wheat, BurlyWood, Tan,
    RosyBrown, SandyBrown, Goldenrod, DarkGoldenrod, Peru, Chocolate,
    SaddleBrown, Sienna, Brown, Maroon,
    // Whites
    White, Snow, Honeydew, MintCream, Azure, AliceBlue, GhostWhite,
    WhiteSmoke, Seashell, Beige, OldLace, FloralWhite, Ivory,
    AntiqueWhite, Linen, LavenderBlush, MistyRose,
    // Grays
    Gainsboro, LightGray, Silver, DarkGray, Gray, DimGray, LightSlateGray,
    SlateGray, DarkSlateGray, Black,

    /// Arbitrary RGBA linear color for values not in the CSS palette.
    Custom(f32, f32, f32, f32),
}
```

### Shape

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub enum Shape {
    Rectangle,
    RoundedRectangle { corner_radius: f32 },
    Hexagon,
    Octagon,
    Circle,
    Diamond,
    Shield,
    Angular,
    Crystalline,
    Custom(CustomShape),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub struct CustomShape {
    pub vertices: Vec<Vec2>,  // max 16, centered at origin, CCW winding
}
```

Shape type integer mapping:

| Variant | `shape_type` | SDF Function |
|---------|-------------|--------------|
| Rectangle | 0 | `sdBox` |
| RoundedRectangle | 1 | `sdRoundedBox` (uses `corner_radius`) |
| Hexagon | 2 | `sdHexagon` |
| Octagon | 3 | `sdRegularPolygon(n=8)` |
| Circle | 4 | `sdCircle` |
| Diamond | 5 | `sdRhombus` |
| Shield | 6 | `sdPolygon` (hardcoded WGSL verts) |
| Angular | 7 | `sdPolygon` (hardcoded WGSL verts) |
| Crystalline | 8 | `sdPolygon` (hardcoded WGSL verts) |
| Custom | 9 | `sdPolygon` (vertices from uniform array, max 16) |

### Aura

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub enum Aura {
    ShieldShimmer {
        pulse_speed: f32,
        intensity: f32,
        color: Hue,
        radius: f32,
    },
    TimeDistortion {
        ripple_frequency: f32,
        echo_count: u32,
        intensity: f32,
        color: Hue,
    },
    PrismaticSplit {
        refraction_intensity: f32,
        spectral_spread: f32,
        color: Hue,
    },
}
```

### Trail

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub enum Trail {
    ShieldEnergy {
        width: f32,
        fade_length: f32,
        color: Hue,
        intensity: f32,
    },
    Afterimage {
        copy_count: u32,
        fade_rate: f32,
        color: Hue,
        spacing: f32,
    },
    PrismaticSplit {
        spectral_spread: f32,
        fade_length: f32,
    },
}
```

### GlowParams and Newtypes

```rust
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub struct HdrBrightness(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub struct BloomIntensity(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub struct EmissiveStrength(pub f32);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub struct GlowParams {
    pub core_brightness: HdrBrightness,
    pub halo_radius: f32,
    pub halo_falloff: f32,
    pub bloom: BloomIntensity,
}
```

### EntityVisualConfig

```rust
#[derive(Clone, Debug, Default, Deserialize, Serialize, Reflect)]
pub struct EntityVisualConfig {
    #[serde(default)]
    pub shape: Option<Shape>,
    #[serde(default)]
    pub color: Option<Hue>,
    #[serde(default)]
    pub glow: Option<GlowParams>,
    #[serde(default)]
    pub aura: Option<Aura>,
    #[serde(default)]
    pub trail: Option<Trail>,
}
```

### VisualModifier (12 variants)

```rust
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Reflect)]
pub enum VisualModifier {
    // Multipliers (stacks with DR via AddModifier, absolute via SetModifier)
    TrailLength(f32),
    GlowIntensity(f32),
    CoreBrightness(f32),
    HaloRadius(f32),
    ShapeScale(f32),
    SpikeCount(u32),

    // Color
    ColorShift(Hue),
    ColorCycle { colors: [Hue; 4], speed: f32 },

    // Transparency
    AlphaOscillation { min: f32, max: f32, frequency: f32 },

    // Entity deformation (shader uniform, NOT Transform)
    SquashStretch { x_scale: f32, y_scale: f32 },

    // Trail variants
    AfterimageTrail(bool),

    // Dynamic
    RotationSpeed(f32),
}
```

### ModifierKind (discriminant key)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect)]
pub enum ModifierKind {
    TrailLength,
    GlowIntensity,
    CoreBrightness,
    HaloRadius,
    ShapeScale,
    SpikeCount,
    ColorShift,
    ColorCycle,
    AlphaOscillation,
    SquashStretch,
    AfterimageTrail,
    RotationSpeed,
}
```

### ModifierStack

```rust
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct ModifierStack {
    /// Active modifiers keyed by source identifier.
    /// Source string identifies who applied the modifier (e.g., "speed_boost_chip").
    entries: HashMap<(ModifierKind, String), ModifierEntry>,
}

#[derive(Clone, Debug, Reflect)]
pub struct ModifierEntry {
    pub modifier: VisualModifier,
    pub source: String,
    pub duration: Option<f32>,  // None = permanent until removed
}
```

### RunTemperature

```rust
#[derive(Resource, Clone, Debug, Default, Reflect)]
pub struct RunTemperature(pub f32);  // 0.0 = cool, 1.0 = hot
```

### TemperaturePalette

```rust
#[derive(Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct TemperaturePalette {
    pub cool_grid: Hue,
    pub hot_grid: Hue,
    pub cool_bloom: Hue,
    pub hot_bloom: Hue,
    pub cool_wall: Hue,
    pub hot_wall: Hue,
}
```

### EntityGlowMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct EntityGlowMaterial {
    #[uniform(0)]
    pub uniforms: EntityGlowUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct EntityGlowUniforms {
    pub color: Vec4,
    pub core_brightness: f32,
    pub halo_radius: f32,
    pub halo_falloff: f32,
    pub bloom_intensity: f32,
    pub half_extents: Vec2,
    pub shape_type: u32,
    pub corner_radius: f32,
    pub spike_count: u32,
    pub dissolve_threshold: f32,
    pub squash_x: f32,
    pub squash_y: f32,
    pub alpha_override: f32,
    pub rotation_angle: f32,
    pub _padding: Vec2,
}
```

### AuraMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct AuraMaterial {
    #[uniform(0)]
    pub uniforms: AuraUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct AuraUniforms {
    pub color: Vec4,
    pub variant: u32,           // 0=ShieldShimmer, 1=TimeDistortion, 2=PrismaticSplit
    pub intensity: f32,
    pub _pad0: Vec2,
    pub pulse_speed: f32,
    pub radius: f32,
    pub _pad1: Vec2,
    pub ripple_frequency: f32,
    pub echo_count: u32,
    pub _pad2: Vec2,
    pub refraction_intensity: f32,
    pub spectral_spread: f32,
    pub _pad3: Vec2,
}
```

### TrailRibbonMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct TrailRibbonMaterial {
    #[uniform(0)]
    pub uniforms: TrailRibbonUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TrailRibbonUniforms {
    pub color: Vec4,
    pub hdr_intensity: f32,
    pub alpha: f32,
    pub _padding: Vec2,
}
```

### GlitchMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct GlitchMaterial {
    #[uniform(0)]
    pub uniforms: GlitchUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct GlitchUniforms {
    pub time: f32,
    pub scanline_density: f32,
    pub scanline_speed: f32,
    pub chromatic_offset: f32,
    pub jitter_intensity: f32,
    pub _padding: Vec3,
}
```

### HolographicMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct HolographicMaterial {
    #[uniform(0)]
    pub uniforms: HolographicUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct HolographicUniforms {
    pub base_color: Vec4,
    pub shimmer_speed: f32,
    pub spectral_intensity: f32,
    pub scan_line_frequency: f32,
    pub _padding: f32,
}
```

## Shader Details

### entity_glow.wgsl

The core entity rendering shader. Every gameplay entity (bolt, breaker, cell, wall segment) is rendered as an SDF-on-quad with this shader.

**Algorithm per fragment:**
1. Convert UV to centered coordinates: `let uv = (in.uv * 2.0 - 1.0) * material.half_extents`
2. Apply squash/stretch: `uv.x *= material.squash_x; uv.y *= material.squash_y`
3. Apply rotation: rotate UV by `material.rotation_angle`
4. Evaluate SDF via `shape_type` switch (0-9)
5. Compute core fill: `step(0.0, -d)` (1.0 inside shape, 0.0 outside)
6. Compute halo glow: `exp(-max(d, 0.0) * material.halo_falloff)`
7. Apply spike modulation if `material.spike_count > 0`: sinusoidal angle-based glow push
8. Apply dissolve if `material.dissolve_threshold > 0.0`: noise-based discard with burning edge
9. Final color: `material.color.rgb * (core * material.core_brightness + halo)` with alpha `max(core, halo) * material.alpha_override`

**SDF functions** (in sdf.wgsl, imported):
- `sdCircle(p: vec2<f32>, r: f32) -> f32`
- `sdBox(p: vec2<f32>, b: vec2<f32>) -> f32`
- `sdRoundedBox(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32`
- `sdHexagon(p: vec2<f32>, r: f32) -> f32`
- `sdRegularPolygon(p: vec2<f32>, r: f32, n: u32) -> f32`
- `sdRhombus(p: vec2<f32>, b: vec2<f32>) -> f32`
- `sdPolygon(p: vec2<f32>, verts: array<vec2<f32>, 16>, count: u32) -> f32`

**Built-in polygon shapes** (Shield, Angular, Crystalline): vertex lists are `const` arrays hardcoded in WGSL within the switch cases. No uniform data needed. Custom shapes pass vertices as a separate uniform buffer at `@group(2)` binding.

**Sources**: SDF primitives from munrocket WGSL gist + Inigo Quilez SDF reference. Simplex noise from munrocket MIT-licensed WGSL gist. All included directly, no external shader dependencies.

### Additive Blend Material2d Pattern

All entity and effect materials use additive blending via `Material2d::specialize()`:

```rust
fn specialize(
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayoutRef,
    _key: Material2dKey<Self>,
) -> Result<(), SpecializedMeshPipelineError> {
    if let Some(fragment) = &mut descriptor.fragment {
        for target in fragment.targets.iter_mut().flatten() {
            target.blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
        }
    }
    Ok(())
}
```

Black (0,0,0) = no contribution. Bright values = light on top. HDR values > 1.0 produce bloom. This is required because Bevy 0.18 `AlphaMode2d` has NO `Add` variant.

Extract this as a shared helper function `apply_additive_blend(descriptor)` to avoid duplication across EntityGlowMaterial, AuraMaterial, TrailRibbonMaterial, GlitchMaterial, HolographicMaterial.

### glitch_text.wgsl

Child overlay Material2d for highlight moment typography. Applied to a quad mesh sitting on top of a Text2d entity.

**Algorithm per fragment:**
1. Scanlines: `sin(uv.y * scanline_density + time * scanline_speed)` modulates alpha in horizontal bands
2. Chromatic split: sample at `uv`, `uv + vec2(chromatic_offset, 0.0)`, `uv - vec2(chromatic_offset, 0.0)` for R, G, B channels
3. Jitter: hash-based block displacement — divide UV into blocks, offset some horizontally based on `hash(block_id + floor(time * jitter_rate))`
4. Additive composite

Punch scale is NOT a shader effect — it uses `PunchScale` component (Transform animation), which is already part of this domain.

### holographic.wgsl

Material2d for Evolution-rarity chip card backgrounds. Simulates holographic foil shimmer.

**Algorithm per fragment:**
1. Render base_color
2. Compute spectral overlay: hue shift based on `uv.x + uv.y + time * shimmer_speed`
3. Additive blend shimmer layer at `spectral_intensity`
4. Optional scan lines at `scan_line_frequency`

### noise.wgsl

Custom simplex noise 2D/3D. Imported by entity_glow.wgsl for dissolve effect. Based on munrocket MIT-licensed WGSL gist. No external dependencies.

### sdf.wgsl

2D SDF primitive functions. Imported by entity_glow.wgsl. Based on munrocket WGSL gist and Inigo Quilez SDF reference.

## Module Structure

```
breaker-game/src/visuals/
    mod.rs                          // pub(crate) mod + re-exports
    plugin.rs                       // VisualsPlugin
    types/
        mod.rs                      // pub(crate) mod + re-exports
        hue.rs                      // Hue enum (~148 CSS colors + Custom), From impls
        shape.rs                    // Shape enum, CustomShape, shape_type_index()
        aura.rs                     // Aura enum with params
        trail.rs                    // Trail enum with params
        glow_params.rs              // GlowParams, HdrBrightness, BloomIntensity, EmissiveStrength
        entity_visual_config.rs     // EntityVisualConfig struct
        visual_modifier.rs          // VisualModifier enum (12 variants)
        modifier_kind.rs            // ModifierKind enum (discriminant key)
    components/
        mod.rs                      // pub(crate) mod + re-exports
        modifier_stack.rs           // ModifierStack component, ModifierEntry
        fade_out.rs                 // FadeOut component (migrated from fx/)
        punch_scale.rs              // PunchScale component (migrated from fx/)
    resources/
        mod.rs                      // pub(crate) mod + re-exports
        run_temperature.rs          // RunTemperature resource
        temperature_palette.rs      // TemperaturePalette struct
    materials/
        mod.rs                      // pub(crate) mod + re-exports, additive blend helper
        entity_glow.rs              // EntityGlowMaterial, EntityGlowUniforms
        aura.rs                     // AuraMaterial, AuraUniforms
        trail_ribbon.rs             // TrailRibbonMaterial, TrailRibbonUniforms
        glitch.rs                   // GlitchMaterial, GlitchUniforms
        holographic.rs              // HolographicMaterial, HolographicUniforms
    systems/
        mod.rs                      // pub(crate) mod + re-exports
        animate_fade_out.rs         // animate_fade_out system (migrated from fx/)
        animate_punch_scale.rs      // animate_punch_scale system (migrated from fx/)

breaker-game/assets/shaders/
    entity_glow.wgsl                // Core entity SDF shader
    aura.wgsl                       // Aura variant shader (placeholder — rendering in 5g)
    trail_ribbon.wgsl               // Trail ribbon shader (placeholder — rendering in 5f)
    glitch_text.wgsl                // Glitch text overlay shader
    holographic.wgsl                // Holographic card shader
    sdf.wgsl                        // Shared SDF primitive functions
    noise.wgsl                      // Shared simplex noise functions
```

Notes on file sizes:
- `hue.rs` will be large (~400+ lines for 148 variants + From impls) but this is acceptable — it is a single enum definition with mechanical conversion implementations, not complex logic. Splitting it would fragment a single concept. Tests for Hue should be in a separate `tests/` directory under `types/hue/` if the file exceeds 400 lines with tests included.
- All other files should stay well under 400 lines.

## visuals/ Domain Rules

From the revised architecture (`docs/todos/detail/phase-5-rethink/architecture.md`):

1. **visuals/ provides the visual vocabulary**: types (Shape, Hue, Aura, Trail, GlowParams, VisualModifier, ModifierStack, EntityVisualConfig), shaders (entity_glow, aura, trail_ribbon, glitch_text, holographic), materials, and animation systems (FadeOut, PunchScale).
2. **visuals/ has its own plugin**: `VisualsPlugin`, registered in `game.rs`.
3. **Domains use visuals types via their builders**: bolt/, cells/, breaker/, walls/ choose which Shape, Hue, Aura, Trail to use based on their entity definitions. They import from `crate::visuals::`.
4. **visuals/ does NOT know about bolt, breaker, cell, or wall**: it only knows its own types. No game entity vocabulary in visuals/ code.
5. **Same relationship as effect/**: effect domain provides infrastructure (trigger→effect pipeline), other domains define their effects. visuals/ provides visual infrastructure, other domains use it.
6. **No messages for visual attachment**: builders attach visual components directly at spawn time. No `AttachVisuals` god-message. This is a key simplification from the old architecture.
7. **Modifier messages are visuals-owned**: `SetModifier` / `AddModifier` / `RemoveModifier` messages live in visuals/ and are consumed by the modifier computation system (built in 5j).

## What NOT to Do

- Do NOT implement the modifier computation system that reads ModifierStack and updates material uniforms — that is 5j (dynamic visuals).
- Do NOT implement the temperature palette application system that reads RunTemperature and updates grid/bloom/wall colors — that is 5j.
- Do NOT implement any rendering consumers (bolt builder attaching Shape, cell builder attaching Glow, etc.) — those are 5f-5i.
- Do NOT implement trail update systems (ring buffer sampling, ribbon mesh generation, afterimage pool repositioning) — those are 5f-5i.
- Do NOT implement aura rendering systems (child entity spawning, aura animation) — those are 5g.
- Do NOT implement the system that updates RunTemperature on node transitions — that is 5j.
- Do NOT add RON rendering blocks to entity definitions — those are 5f-5i.
- Do NOT write particle-related code — that is rantzsoft_particles2d (5c).
- Do NOT write post-processing code — that is rantzsoft_postprocess (5d).
- Do NOT implement damage display or death effect rendering — those are 5h (cell visuals).
- Do NOT create a `rendering/` or `graphics/` domain — visuals/ is the name.
- Do NOT put these types in a rantzsoft crate — they are game-specific, they live in breaker-game.

## Dependencies

- **Requires**: nothing (types only, no crate dependencies beyond Bevy and serde)
- **Independent of**: 5c (rantzsoft_particles2d), 5d (rantzsoft_postprocess) — can run in parallel
- **Required by**: 5f (bolt visuals), 5g (breaker visuals), 5h (cell visuals), 5i (walls & background), 5j (dynamic visuals), 5k (bump VFX), 5l (combat VFX), 5m (highlights), 5n (HUD), 5o (chip cards)

## Verification

- All type enums (Hue, Shape, Aura, Trail, VisualModifier, ModifierKind) deserialize from RON correctly
- EntityVisualConfig deserializes with all fields populated and with optional fields omitted
- Hue round-trips through `From<Hue> for Color` and back for all named variants
- Shape `shape_type_index()` returns correct integer for each variant
- ModifierStack push/remove/query operations work correctly
- ModifierStack diminishing returns curve produces expected values
- RunTemperature clamps to 0.0-1.0 range
- TemperaturePalette lerp produces correct intermediate colors
- EntityGlowMaterial, AuraMaterial, TrailRibbonMaterial, GlitchMaterial, HolographicMaterial compile with correct ShaderType derives
- FadeOut and PunchScale systems work identically after migration (all existing tests pass)
- fx/ domain is fully eliminated — no references remain
- VisualsPlugin builds without error
- All WGSL shader files parse without syntax errors
- All existing tests across the workspace pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## NEEDS DETAIL — API Design

These questions must be answered before implementation:

- How do entity builders consume `EntityVisualConfig`? (pass whole struct, or individual Shape/Hue/Glow args?)
- How are Aura and Trail entities spawned — does the builder create them, or is there a system that watches for `Added<AuraConfig>` and spawns the visual entity?
- What `pub` API does `VisualsPlugin` expose to other domains?
- Should Shape/Hue/Aura/Trail be `#[derive(Component)]` directly, or are they fields inside a config struct that a system reads to create the actual material?
- How does the entity_glow shader receive its shape type — integer uniform on a per-entity material instance, or a shared material with per-instance data?
- How do modifier types compose at compile time? Can we use enums + trait bounds to prevent invalid modifier combinations?
- What's the FadeOut/PunchScale migration — do they become `Birthing`-style shared components, or stay as-is but move to visuals/?
- Does `VisualsPlugin` register in `Update` or `FixedUpdate`? (visual systems probably `Update`, but modifier computation might need `FixedUpdate`)

## Status
`[NEEDS DETAIL]` — API design questions above must be resolved
