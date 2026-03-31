# Aura Rendering Research — Bevy 0.18.1

Verified against docs.rs/bevy/0.18.1, Bevy GitHub (v0.18.1 tag), and external shader references.
Date: 2026-03-30

---

## 0. Project Context

The `rantzsoft_vfx` crate (planned, not yet implemented) will own all aura rendering.
The `Aura` enum is defined in `docs/architecture/rendering/types.md`:

```rust
pub enum Aura {
    ShieldShimmer { pulse_speed: f32, intensity: f32, color: Hue, radius: f32 },
    TimeDistortion { ripple_frequency: f32, echo_count: u32, intensity: f32, color: Hue },
    PrismaticSplit { refraction_intensity: f32, spectral_spread: f32, color: Hue },
}
```

Each variant maps to a dedicated `.wgsl` shader file inside `rantzsoft_vfx`.

---

## 1. Bevy 0.18.1 Material2d API (VERIFIED)

### Module path
```
bevy::sprite_render::Material2d         // trait
bevy::sprite_render::Material2dPlugin   // plugin
bevy::sprite_render::MeshMaterial2d     // component
bevy::sprite_render::AlphaMode2d        // enum
bevy::prelude::Mesh2d                   // component
```

`Material2d` moved from `bevy::sprite` to `bevy::sprite_render` in Bevy 0.17 (not a 0.18 change).

### Full trait definition (verified docs.rs/bevy/0.18.1)
```rust
pub trait Material2d: Sized + AsBindGroup + Asset + Clone {
    fn vertex_shader() -> ShaderRef { ShaderRef::Default }
    fn fragment_shader() -> ShaderRef { ShaderRef::Default }
    fn depth_bias(&self) -> f32 { 0.0 }
    fn alpha_mode(&self) -> AlphaMode2d { AlphaMode2d::Opaque }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> { Ok(()) }
}
```

All methods have defaults. Only override what you need.

### AlphaMode2d variants (verified — NO Additive variant)
```rust
AlphaMode2d::Opaque        // alpha overridden to 1.0
AlphaMode2d::Mask(f32)     // binary threshold
AlphaMode2d::Blend         // standard alpha blending
```

2D does NOT have Additive as an AlphaMode2d variant (unlike 3D).
**To get additive blending, you must use `specialize`.**

### Additive blending via specialize (verified BlendFactor variants)
```rust
use bevy::render::render_resource::{
    BlendComponent, BlendFactor, BlendOperation, BlendState,
    ColorTargetState, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};

fn specialize(
    descriptor: &mut RenderPipelineDescriptor,
    _layout: &MeshVertexBufferLayoutRef,
    _key: Material2dKey<Self>,
) -> Result<(), SpecializedMeshPipelineError> {
    if let Some(fragment) = descriptor.fragment.as_mut() {
        if let Some(target) = fragment.targets.first_mut() {
            if let Some(target) = target.as_mut() {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                });
            }
        }
    }
    Ok(())
}
```

**BlendFactor::One confirmed to exist** (one of 17 variants). This is the correct pattern for additive glow.

### Minimum struct definition for an aura material
```rust
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AuraShieldShimmerMaterial {
    #[uniform(0)]
    pub params: ShieldShimmerParams,  // WGSL struct with pulse_speed, intensity, color, radius
}

impl Material2d for AuraShieldShimmerMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/aura_shield_shimmer.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend  // use specialize for true additive
    }
    fn specialize(...) -> Result<...> { /* additive blend above */ }
}
```

Plugin registration (in rantzsoft_vfx Plugin::build):
```rust
app.add_plugins(Material2dPlugin::<AuraShieldShimmerMaterial>::default());
app.add_plugins(Material2dPlugin::<AuraTimeDistortionMaterial>::default());
app.add_plugins(Material2dPlugin::<AuraPrismaticSplitMaterial>::default());
```

---

## 2. Aura as Child Mesh Entity (VERIFIED)

### Spawning pattern
```rust
// In AttachVisuals handler, after spawning main entity mesh:
fn attach_aura(
    commands: &mut Commands,
    entity: Entity,
    aura: &Aura,
    meshes: &mut Assets<Mesh>,
    materials_shimmer: &mut Assets<AuraShieldShimmerMaterial>,
    // ... other material asset stores
) {
    let aura_entity = match aura {
        Aura::ShieldShimmer { radius, .. } => {
            // Circle mesh at ~1.3-1.5x entity radius, behind parent (negative z)
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(*radius * 1.4))),
                MeshMaterial2d(materials_shimmer.add(build_shimmer_material(aura))),
                Transform::from_xyz(0.0, 0.0, -0.5),  // behind parent visual
            )).id()
        },
        // ... other variants
    };
    // Attach as child of the gameplay entity
    commands.entity(entity).add_child(aura_entity);
}
```

### Child API (verified EntityCommands in Bevy 0.18.1)
```rust
// Option A: closure (multiple children)
commands.entity(parent).with_children(|spawner| {
    spawner.spawn((Mesh2d(...), MeshMaterial2d(...), Transform::from_xyz(0., 0., -0.5)));
});

// Option B: single child shorthand
commands.entity(parent).with_child((
    Mesh2d(meshes.add(Circle::new(radius))),
    MeshMaterial2d(materials.add(mat)),
    Transform::from_xyz(0., 0., -0.5),
));

// Option C: spawn then link
let child = commands.spawn((...)).id();
commands.entity(parent).add_child(child);
```

### Z-ordering for "behind parent"
In 2D, z-translation controls draw order. Use `Transform::from_xyz(0.0, 0.0, -0.5)` on the aura child so it renders behind the entity's mesh. The entity's mesh visual should be at z=0.0 or slightly positive.

**Known issue**: When a Mesh2d entity is a child of an entity with a Sprite component, Transform may not propagate correctly in Bevy 0.17.2 (GitHub issue #21764). Since our entities use Mesh2d directly (no Sprite), this should not be a problem.

### Mesh shape recommendation
- Circle: `meshes.add(Circle::new(aura_radius))` — simplest, works for all entity shapes
- Annulus/Ring: Not a built-in primitive in Bevy 0.18 mesh2d. Use a circle mesh large enough to contain the aura.
- The SDF approach (see section 4) handles "ring-shaped" glow mathematically inside the shader, so the mesh can be a simple circle or quad.

---

## 3. Time/Globals in 2D WGSL Shaders (VERIFIED)

**Critical: 2D shaders must use the 2D imports, NOT the PBR/3D imports.**

### Correct import for Material2d shaders
```wgsl
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}
```

NOT `bevy_pbr::mesh_view_bindings` (3D only — will cause validation errors).

### Globals struct (verified from bevy_render/src/globals.wgsl, v0.18.1)
```wgsl
struct Globals {
    // Time since startup in seconds. Wraps after 1 hour to avoid f32 precision loss.
    time: f32,
    // Delta time in seconds since the previous frame.
    delta_time: f32,
    // Frame count since app startup. Wraps at u32::MAX.
    frame_count: u32,
    // WebGL2 alignment padding (conditional, present when SIXTEEN_BYTE_ALIGNMENT)
    _webgl2_padding: f32,
}
```

Usage in fragment shader:
```wgsl
let t = globals.time;
let dt = globals.delta_time;
```

### Material uniform binding group
For Material2d, material uniforms use `@group(2)`:
```wgsl
@group(2) @binding(0) var<uniform> material: MyMaterialParams;
// textures start at binding 1
@group(2) @binding(1) var some_texture: texture_2d<f32>;
@group(2) @binding(2) var some_sampler: sampler;
```

---

## 4. SDF-Based Aura Shader Technique (VERIFIED)

### Core SDF glow formula
For a circle of radius `r` centered at UV origin (0.5, 0.5):
```wgsl
let uv = in.uv - vec2(0.5, 0.5);      // center at origin
let d = length(uv) - r;                // signed distance to circle edge
                                        // d < 0 = inside, d > 0 = outside

// Ring glow (glow on both sides of the edge)
let ring_width = 0.05;
let edge_dist = abs(d);
let glow = smoothstep(ring_width, 0.0, edge_dist);

// Outer halo falloff (glow only outside)
let halo_strength = 0.02 / max(d, 0.001);  // 1/x falloff
let halo = clamp(halo_strength, 0.0, 1.0);
```

This is feasible for arbitrary SDF shapes — the parent entity's Shape enum maps to an SDF function:
- `Shape::Circle` → `length(uv) - r`
- `Shape::Rectangle` → `max(abs(uv.x) - w, abs(uv.y) - h)` (box SDF)
- `Shape::Hexagon` → hexagon SDF (dot product with 3 axes)
- `Shape::Angular` → custom SDF

The aura shader receives a `shape_type` uniform and branches on it, or separate shader variants per shape. Given the project already has separate shaders per Aura variant, the simpler approach is circle-mesh aura with a circle SDF (sufficient since the aura glows around the entity regardless of shape).

### SDF technique verdict
Feasible in WGSL for 2D. The mesh is a large-enough circle, the fragment shader computes the distance from the fragment to the "effective edge" of the shape (using the entity's bounding radius), and produces a soft falloff. Full SDF matching of arbitrary shapes requires passing shape parameters as uniforms — doable but adds complexity. For most aura effects, approximating with a circular SDF is visually sufficient.

---

## 5. Shield Shimmer Shader Technique

### What creates "shimmer"
Three techniques combined (from energy shield shader art research):

**A. Noise-driven pulsation**
Scroll a 2D noise texture or procedural simplex noise over time to create "breathing" variations in brightness:
```wgsl
// Scroll UV over time, sample noise, remap to glow strength range
let scrolled_uv = uv + vec2(0.0, globals.time * scroll_speed);
let noise_val = simplexNoise2(scrolled_uv * noise_scale);
let pulsed_alpha = mix(glow_min, glow_max, noise_val * 0.5 + 0.5);
```

**B. Sine wave edge ripple**
```wgsl
// Ripple along the edge of the shield
let angle = atan2(uv.y, uv.x);
let ripple = sin(angle * ripple_freq + globals.time * ripple_speed) * ripple_amp;
let effective_radius = base_radius + ripple;
let d = length(uv) - effective_radius;
```

**C. Scanline sweep**
A bright scanline sweeps vertically over the shield:
```wgsl
let scan_pos = fract(globals.time * scan_speed);
let scan_dist = abs(uv_normalized.y - scan_pos);
let scan_line = smoothstep(scan_thickness, 0.0, scan_dist) * scan_strength;
```

### Recommended shader uniform struct for ShieldShimmer
```wgsl
struct ShieldShimmerParams {
    color: vec4<f32>,
    pulse_speed: f32,
    intensity: f32,
    radius: f32,
    ripple_freq: f32,
}
```

### WGSL simplex noise 2D (ready to use in Bevy)
```wgsl
fn mod289(x: vec2f) -> vec2f { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn mod289_3(x: vec3f) -> vec3f { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn permute3(x: vec3f) -> vec3f { return mod289_3(((x * 34.0) + 1.0) * x); }

fn simplex_noise_2d(v: vec2f) -> f32 {
    let C = vec4(0.211324865405187, 0.366025403784439,
                 -0.577350269189626, 0.024390243902439);
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);
    var i1 = select(vec2(0.0, 1.0), vec2(1.0, 0.0), x0.x > x0.y);
    var x12 = x0.xyxy + C.xxzz;
    x12.x -= i1.x;
    x12.y -= i1.y;
    i = mod289(i);
    var p = permute3(permute3(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.0));
    m *= m; m *= m;
    let x = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x) - 0.5;
    let a0 = x - floor(x + 0.5);
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130.0 * dot(m, g);
}
```

Source: MIT license, Stefan Gustavson (WGSL port by munrocket).

---

## 6. Time Distortion Ripple Shader Technique

### Core ripple technique (radial wave from center)
```wgsl
let uv = in.uv - vec2(0.5, 0.5);
let dist = length(uv);
let t = globals.time;

// Expanding radial waves (multiple rings)
let wave1 = sin(dist * ripple_frequency - t * 4.0) * 0.5 + 0.5;
let wave2 = sin(dist * ripple_frequency * 1.5 - t * 3.0 + 1.0) * 0.5 + 0.5;

// Combine and apply SDF falloff
let d = dist - entity_radius;
let edge_falloff = smoothstep(aura_outer_radius, entity_radius, dist);
let wave_combined = (wave1 * 0.6 + wave2 * 0.4) * edge_falloff;
```

### Echo/afterimage effect (UV displacement)
The TimeDistortion aura gets its ripple look from displacing UVs based on a wave:
```wgsl
// Displacement distortion on the entity body
let uv_dir = normalize(uv);
let displacement = uv_dir * cos(dist * ripple_frequency - t * wave_speed) * intensity;
// Sample entity color with displaced UVs (for overlay on entity mesh)
```

For the **aura child mesh** specifically (not overlaid on entity), the effect is:
- Rings of color that pulse outward from the entity center
- Fading alpha at the outer edge
- Multiple phase-shifted rings for `echo_count` parameter

```wgsl
fn time_distortion_alpha(uv: vec2f, dist: f32, time: f32,
                          frequency: f32, echo_count: u32, intensity: f32) -> f32 {
    var alpha = 0.0;
    for (var i = 0u; i < echo_count; i++) {
        let phase_offset = f32(i) * (6.283 / f32(echo_count)); // spread echoes evenly
        let ring = sin(dist * frequency - time * 3.0 + phase_offset);
        let ring_alpha = max(0.0, ring) * (1.0 - f32(i) / f32(echo_count)); // fade by echo index
        alpha = max(alpha, ring_alpha);
    }
    return alpha * intensity;
}
```

---

## 7. Prismatic Split Shader Technique

### Spectral color separation (verified technique)
The prismatic effect separates color channels by offsetting their UVs along the edge gradient direction, simulating wavelength-dependent refraction (chromatic dispersion):

```wgsl
// Different offsets per channel simulate spectral dispersion
let edge_dir = normalize(uv);  // direction outward from center
let r_offset = edge_dir * refraction_intensity * 1.0;  // red bends least
let g_offset = edge_dir * refraction_intensity * 0.5;  // green intermediate
let b_offset = edge_dir * refraction_intensity * 0.0;  // blue doesn't shift (reference)

// Apply to an energy color texture or procedural gradient:
// Each channel samples from a slightly different position on the aura gradient
let r_dist = length(uv + r_offset) - entity_radius;
let g_dist = length(uv + g_offset) - entity_radius;
let b_dist = length(uv + b_offset) - entity_radius;

// Compute glow for each channel independently
let r_glow = compute_edge_glow(r_dist);
let g_glow = compute_edge_glow(g_dist);
let b_glow = compute_edge_glow(b_dist);

// Combine into rainbow fringe
let final_color = vec4(r_glow, g_glow, b_glow, max(max(r_glow, g_glow), b_glow));
```

### Spectral gradient (rainbow mapping)
The `spectral_spread` parameter controls how wide the rainbow fringe is. A spectral hue rotation based on distance from the edge creates the full rainbow:
```wgsl
fn hue_to_rgb(h: f32) -> vec3f {
    let r = abs(h * 6.0 - 3.0) - 1.0;
    let g = 2.0 - abs(h * 6.0 - 2.0);
    let b = 2.0 - abs(h * 6.0 - 4.0);
    return clamp(vec3(r, g, b), vec3(0.0), vec3(1.0));
}

// Map edge distance to hue position
let hue = fract((d / spectral_spread) + globals.time * 0.1); // slow rotation
let spectral_color = hue_to_rgb(hue);
```

### Multi-sample smoothing for natural result
Iterate multiple samples along the dispersion direction for a smoother rainbow without harsh bands:
```wgsl
const LOOP: i32 = 4;
var color = vec3(0.0);
for (var i = 0; i < LOOP; i++) {
    let slide = f32(i) / f32(LOOP) * spectral_spread;
    color.r += edge_glow_at_offset(slide * 0.0);   // red at zero offset
    color.g += edge_glow_at_offset(slide * 0.5);   // green shifts
    color.b += edge_glow_at_offset(slide * 1.0);   // blue shifts most
}
color /= f32(LOOP);
```

---

## 8. Performance: How Many Aura Entities Simultaneously

### What happens per aura entity
Each aura entity is a Mesh2d with a MeshMaterial2d. The render cost is:
- **Draw call**: 1 per unique material instance (unless Bevy batches identical materials)
- **Shader work**: Fragment shader per pixel covered by the mesh

### Batching behavior in Bevy 0.18
Bevy batches/instances Mesh2d entities automatically when they share the same material asset handle AND the same mesh. Aura entities for different gameplay entities will have **different material instances** (different params), so they will NOT batch — each is a separate draw call.

### Practical entity counts for this game
- Bolts: 1-5 active bolts × 1 aura = 1-5 draw calls
- Breaker: 1 × 1 aura = 1 draw call
- Cells: cells do not have auras (per design docs)
- Total: 2-6 aura draw calls maximum at any moment

**This is trivially cheap.** Even 100 custom shader draw calls per frame on modern hardware is negligible. The fragment shader complexity matters more than draw call count at this scale.

### Fragment shader cost
The aura shaders use:
- 1 noise sample (simplex): ~20-30 GPU instructions
- SDF distance calculation: ~5 GPU instructions
- A few sin/cos calls: ~2-5 instructions each

For a 200×200 pixel aura area = 40,000 fragments × ~50 instructions = 2M instructions. At 10 TFLOPS (mid-range GPU), this is ~0.0002ms per aura. Negligible.

**Verdict: No GPU performance concerns for this game's scale.**

---

## 9. Complete Example: Minimal Aura Material in Bevy 0.18.1

### Rust (in rantzsoft_vfx)
```rust
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
        RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};

/// Uniform struct for the shield shimmer aura shader.
/// Must match the WGSL struct layout exactly.
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
#[repr(C)]
pub struct ShieldShimmerParams {
    pub color: Vec4,        // RGBA
    pub pulse_speed: f32,
    pub intensity: f32,
    pub radius: f32,
    pub ripple_freq: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct AuraShieldShimmerMaterial {
    #[uniform(0)]
    pub params: ShieldShimmerParams,
}

impl Material2d for AuraShieldShimmerMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/aura_shield_shimmer.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            if let Some(Some(target)) = fragment.targets.first_mut() {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                });
            }
        }
        Ok(())
    }
}
```

### WGSL (assets/shaders/aura_shield_shimmer.wgsl)
```wgsl
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct ShieldShimmerParams {
    color: vec4<f32>,
    pulse_speed: f32,
    intensity: f32,
    radius: f32,
    ripple_freq: f32,
}

@group(2) @binding(0) var<uniform> params: ShieldShimmerParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv - vec2(0.5, 0.5);
    let dist = length(uv);
    let t = globals.time;

    // SDF glow around the entity edge
    let d = dist - params.radius;
    let edge_glow = exp(-abs(d) * 12.0) * params.intensity;

    // Sine ripple on the edge
    let angle = atan2(uv.y, uv.x);
    let ripple = sin(angle * params.ripple_freq + t * params.pulse_speed) * 0.02;
    let d_rippled = dist - (params.radius + ripple);
    let ripple_glow = exp(-abs(d_rippled) * 10.0) * params.intensity * 0.5;

    // Noise-driven shimmer (simplified — use simplex_noise_2d for production)
    let shimmer = sin(uv.x * 20.0 + t * params.pulse_speed) *
                  sin(uv.y * 20.0 + t * params.pulse_speed * 0.7) * 0.3 + 0.7;

    let alpha = clamp((edge_glow + ripple_glow) * shimmer, 0.0, 1.0);
    return vec4(params.color.rgb * params.color.a, alpha) * alpha;  // premultiplied
}
```

---

## 10. Key Gotchas

1. **2D vs 3D imports**: Use `bevy_sprite::mesh2d_view_bindings::globals` NOT `bevy_pbr::mesh_view_bindings::globals`. Using the PBR import in a Material2d shader causes a GPU validation error about missing pipeline binding.

2. **AlphaMode2d has no Additive**: Must use `specialize()` to configure additive blend state. The `fragment.targets.first_mut()` access pattern is the correct path through RenderPipelineDescriptor → FragmentState → ColorTargetState.

3. **Material2d module path**: `bevy::sprite_render` (not `bevy::sprite`) as of Bevy 0.17+.

4. **Z-ordering for aura behind entity**: Use `Transform::from_xyz(0., 0., -0.5)` on the aura child entity. Both parent and child must have Transform components.

5. **No batching for unique material params**: Each aura entity is a separate draw call. Fine at game scale (max ~6 auras).

6. **Mesh shape for aura**: Use `Circle::new(aura_radius)` where `aura_radius` = entity bounding radius × 1.3-1.5. The shader handles the soft edge mathematically.

7. **@group(2) for material uniforms**: Confirmed for Material2d. Group 0 = view bindings (globals, camera), Group 1 = mesh bindings (transform), Group 2 = material uniforms.

8. **Missing Bevy WGSL noise built-ins**: WGSL/Bevy has no built-in noise functions. Use the MIT-licensed simplex noise implementation from munrocket's gist (see section 5).

9. **Mesh2d child of Sprite bug**: Known issue in 0.17.2 (#21764) where Transform doesn't propagate from Sprite parent to Mesh2d child. Not applicable here since entity visuals are Mesh2d-based (no Sprite components).

10. **Plugin registration**: Each distinct Material2d type needs its own `Material2dPlugin::<T>::default()` call in the rantzsoft_vfx Plugin::build.

---

## Sources Consulted

- docs.rs/bevy/0.18.1/bevy/sprite_render/trait.Material2d.html
- docs.rs/bevy/0.18.1/bevy/sprite_render/enum.AlphaMode2d.html
- docs.rs/bevy/0.18.1/bevy/render/render_resource/struct.BlendState.html
- docs.rs/bevy/0.18.1/bevy/render/render_resource/struct.BlendComponent.html
- docs.rs/bevy/0.18.1/bevy/render/render_resource/enum.BlendFactor.html
- docs.rs/bevy/0.18.1/bevy/ecs/system/struct.EntityCommands.html
- github.com/bevyengine/bevy/blob/v0.18.1/examples/shader/shader_material_2d.rs
- github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_render/src/globals.wgsl
- github.com/bevyengine/bevy/discussions/7143 (globals.time in Material2d)
- danielilett.com/2023-02-09-tut6-3-energy-shield/ (shield shader techniques)
- blog.maximeheckel.com (prismatic dispersion technique)
- inspirnathan.com/posts/65-glow-shader-in-shadertoy/ (SDF glow formula)
- geeks3d.com/20110316/ (ripple shader technique)
- gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39 (WGSL noise)
- bevy.org/learn/migration-guides/0-17-to-0-18/
