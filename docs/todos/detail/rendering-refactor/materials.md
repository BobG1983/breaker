# Material Struct Layouts

Complete `AsBindGroup` struct definitions for all custom materials in `rantzsoft_vfx`. All fields use std140 alignment (16-byte aligned via `ShaderType`/`encase`).

## EntityGlowMaterial

The main entity rendering material. SDF-on-quad with shape selection, glow, dissolve, and modifier-driven uniforms.

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct EntityGlowMaterial {
    #[uniform(0)]
    pub uniforms: EntityGlowUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct EntityGlowUniforms {
    // ── Core visual ──
    pub color: Vec4,                // LinearRgba as vec4
    pub core_brightness: f32,       // from GlowParams, modified by CoreBrightness modifier
    pub halo_radius: f32,           // from GlowParams, modified by HaloRadius modifier
    pub halo_falloff: f32,          // from GlowParams
    pub bloom_intensity: f32,       // from GlowParams (informational — bloom is Bevy's post-process)

    // ── Shape ──
    pub half_extents: Vec2,         // derived from entity Scale2D aspect ratio
    pub shape_type: u32,            // Shape enum → integer (0=Rectangle, 1=RoundedRect, 2=Hexagon, etc.)
    pub corner_radius: f32,         // RoundedRectangle only, 0.0 for others

    // ── Modifiers ──
    pub spike_count: u32,           // SpikeCount modifier (0 = no spikes)
    pub dissolve_threshold: f32,    // Disintegrate animation (0.0 = off, 1.0 = fully dissolved)
    pub squash_x: f32,             // SquashStretch modifier x_scale (1.0 = no distortion)
    pub squash_y: f32,             // SquashStretch modifier y_scale (1.0 = no distortion)

    pub alpha_override: f32,        // AlphaOscillation modifier (1.0 = fully opaque)
    pub rotation_angle: f32,        // RotationSpeed modifier (accumulated radians)
    pub _padding: Vec2,             // std140 alignment to 16 bytes
}
```

### Shape Type Mapping

| Shape Variant | `shape_type` | Notes |
|---------------|-------------|-------|
| Rectangle | 0 | |
| RoundedRectangle | 1 | Uses `corner_radius` |
| Hexagon | 2 | |
| Octagon | 3 | |
| Circle | 4 | |
| Diamond | 5 | |
| Shield | 6 | Hardcoded WGSL verts |
| Angular | 7 | Hardcoded WGSL verts |
| Crystalline | 8 | Hardcoded WGSL verts |
| Custom | 9 | Uses `custom_vertices` uniform (see below) |

### Custom Shape Vertices

Custom shapes pass vertex data via a separate uniform buffer:

```rust
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct CustomShapeVertices {
    pub vertices: [Vec2; 16],       // max 16 vertices, padded
    pub vertex_count: u32,
    pub _padding: Vec3,             // std140 alignment
}
```

Bound at `@group(2)` only when `shape_type == 9`. For built-in shapes, the WGSL shader uses `const` vertex arrays in `switch` cases.

### WGSL Uniform Binding

```wgsl
struct EntityGlowUniforms {
    color: vec4<f32>,
    core_brightness: f32,
    halo_radius: f32,
    halo_falloff: f32,
    bloom_intensity: f32,
    half_extents: vec2<f32>,
    shape_type: u32,
    corner_radius: f32,
    spike_count: u32,
    dissolve_threshold: f32,
    squash_x: f32,
    squash_y: f32,
    alpha_override: f32,
    rotation_angle: f32,
    _padding: vec2<f32>,
}

@group(2) @binding(0) var<uniform> material: EntityGlowUniforms;
```

### Material2d Implementation

```rust
impl Material2d for EntityGlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/entity_glow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend  // transparency for halo falloff
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Additive blending: src.rgb * src.a + dst.rgb
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
}
```

---

## AuraMaterial

Single material type for all aura variants. Variant-switched in WGSL.

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct AuraMaterial {
    #[uniform(0)]
    pub uniforms: AuraUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct AuraUniforms {
    pub color: Vec4,                // LinearRgba
    pub variant: u32,               // 0=ShieldShimmer, 1=TimeDistortion, 2=PrismaticSplit
    pub intensity: f32,
    pub _pad0: Vec2,

    // ShieldShimmer params
    pub pulse_speed: f32,
    pub radius: f32,
    pub _pad1: Vec2,

    // TimeDistortion params
    pub ripple_frequency: f32,
    pub echo_count: u32,
    pub _pad2: Vec2,

    // PrismaticSplit params
    pub refraction_intensity: f32,
    pub spectral_spread: f32,
    pub _pad3: Vec2,
}
```

Union of all variant params — unused fields are ignored per-variant in the shader. Same `specialize()` additive blend as EntityGlowMaterial.

### WGSL Binding

```wgsl
@group(2) @binding(0) var<uniform> material: AuraUniforms;
```

---

## TrailRibbonMaterial

Used by ShieldEnergy and PrismaticSplit trail variants.

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct TrailRibbonMaterial {
    #[uniform(0)]
    pub uniforms: TrailRibbonUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TrailRibbonUniforms {
    pub color: Vec4,                // LinearRgba
    pub hdr_intensity: f32,
    pub alpha: f32,                 // global alpha multiplier (for fade-out)
    pub _padding: Vec2,
}
```

Same `specialize()` additive blend: `SrcAlpha + One`. Vertex alpha controls per-vertex fade along the ribbon (head=1.0, tail=0.0).

---

## ParticleMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ParticleMaterial {
    #[uniform(0)]
    pub uniforms: ParticleUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct ParticleUniforms {
    pub color: Vec4,                // LinearRgba (HDR values > 1.0 for bloom)
    pub alpha: f32,                 // lifetime-driven fade
    pub _padding: Vec3,
}
```

Same `specialize()` additive blend.

---

## GridMaterial

Background playfield grid. Standard `Material2d`, no additive blend.

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct GridMaterial {
    #[uniform(0)]
    pub uniforms: GridUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct GridUniforms {
    pub color: Vec4,                // grid line color (from RunTemperature palette)
    pub playfield_bounds: Vec4,     // min_x, min_y, max_x, max_y
    pub line_spacing: f32,
    pub line_thickness: f32,
    pub glow_intensity: f32,
    pub _padding: f32,
}
```

---

## ShieldMaterial

Shield barrier hexagonal energy field.

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ShieldMaterial {
    #[uniform(0)]
    pub uniforms: ShieldUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct ShieldUniforms {
    pub color: Vec4,                // pulsing white
    pub hex_scale: f32,
    pub pulse_speed: f32,
    pub intensity: f32,
    pub crack_count: u32,

    pub crack_seeds: [Vec4; 5],     // xy = world-space position, zw unused (std140 alignment)
    pub crack_radius: f32,
    pub _padding: Vec3,
}
```

Same `specialize()` additive blend.

---

## FullscreenMaterial Types

Each post-processing effect is a `Component` implementing `FullscreenMaterial`. These use `ShaderType + WriteInto` (not `AsBindGroup`).

### ScreenFlash

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct ScreenFlash {
    pub color: Vec4,                // flash color (HDR)
    pub intensity: f32,             // 0.0 = off, 1.0+ = active
    pub _padding: Vec3,            // std140 alignment
}
```

### RadialDistortion

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct RadialDistortion {
    pub sources: [DistortionSource; 16],
    pub active_count: u32,
    pub _padding: Vec3,
}

#[derive(Clone, Copy, Default, ShaderType, WriteInto)]
pub struct DistortionSource {
    pub origin: Vec2,               // screen-space UV origin
    pub radius: f32,
    pub intensity: f32,
}
```

### ChromaticAberration

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct ChromaticAberration {
    pub intensity: f32,             // 0.0 = off
    pub _padding: Vec3,
}
```

### Desaturation

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct Desaturation {
    pub factor: f32,                // 0.0 = full color, 1.0 = monochrome
    pub _padding: Vec3,
}
```

### Vignette

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct Vignette {
    pub color: Vec4,                // vignette tint color
    pub intensity: f32,             // 0.0 = off
    pub inner_radius: f32,          // where vignette starts (0.0-1.0 normalized)
    pub _padding: Vec2,
}
```

### CRT

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct CrtOverlay {
    pub scanline_intensity: f32,    // 0.0 = off
    pub scanline_frequency: f32,
    pub curvature: f32,             // barrel distortion amount
    pub _padding: f32,
}
```

### CollapseRebuild

```rust
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, WriteInto, Default)]
pub struct CollapseRebuild {
    pub progress: f32,              // 0.0 = start, 1.0 = complete
    pub direction: f32,             // 0.0 = collapse (out), 1.0 = rebuild (in)
    pub tile_count_x: f32,          // grid columns (as f32 for shader)
    pub tile_count_y: f32,          // grid rows
    pub tile_seed: f32,             // per-tile timing variation seed
    pub _padding: Vec3,
}
```
