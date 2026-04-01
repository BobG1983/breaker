# Glow and Destruction Shaders — Bevy 0.18.1 Research

> **Raw research.** Architecture decisions that differ from these findings are in `docs/architecture/rendering/`.
> Specifically: this doc recommends `noisy_bevy` for dissolve noise and `voronator` for fracture Voronoi.
> **Architecture decision**: custom `noise.wgsl` and `voronoi.wgsl` with no external deps (see `shaders.md`).
> Also: this doc recommends pre-computed CPU Voronoi for fracture.
> **Architecture decision**: shader-side Voronoi (sufficient for cell destruction VFX, no physical shard separation needed).

Researched: 2026-03-30
Version confirmed: Bevy 0.18.1 (breaker-game/Cargo.toml)

---

## 1. Core + Halo Glow for 2D Shapes (SDF-based)

**Technique: SDF distance → inverse-distance or exponential falloff**

The standard approach for 2D shape glow:
1. Compute the SDF distance `d` from the current fragment to the shape boundary.
   - Negative `d` = inside shape (core)
   - Positive `d` = outside shape (halo region)
2. Apply a falloff function to `d` to produce glow intensity.

**Two falloff options:**

Option A — Inverse distance (sharp near-edge, gentle falloff):
```wgsl
let glow_intensity = clamp(0.01 / abs(d), 0.0, 1.0);
```
Caveat: `1/x` produces artifacts when `d ≈ 0` — must clamp. Numerator controls glow radius.

Option B — Exponential falloff (softer, more physically plausible):
```wgsl
let glow_intensity = exp(-d * halo_falloff);
```
Where `halo_falloff` is the falloff rate (higher = tighter glow). This is the formula used in Shadertoy glow examples and the Bevy WGSL SDF tutorial.

**Full fragment shader pattern:**
```wgsl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;  // center UV at (0,0)

    // SDF for the shape (example: circle)
    let d = sdCircle(uv, 0.4);

    // Core: step function — inside shape = 1.0, outside = 0.0
    let core = step(0.0, -d);

    // Halo: exponential falloff from boundary outward
    let halo = exp(-max(d, 0.0) * material.halo_falloff) * material.halo_radius;

    // Combine: core at full brightness, halo as additive glow
    let intensity = core * material.core_brightness + halo;

    return vec4<f32>(material.color.rgb * intensity, max(core, halo));
}
```

**WGSL SDF primitives (exact, from munrocket WGSL gist):**

```wgsl
// Circle
fn sdCircle(p: vec2f, r: f32) -> f32 {
    return length(p) - r;
}

// Rectangle (b = half-extents)
fn sdBox(p: vec2f, b: vec2f) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2f(0.0))) + min(max(d.x, d.y), 0.0);
}

// Hexagon (flat-top, r = inradius)
fn sdHexagon(p: vec2f, r: f32) -> f32 {
    let k = vec3f(-0.866025404, 0.5, 0.577350269);
    var q: vec2f = abs(p);
    q = q - 2.0 * min(dot(k.xy, q), 0.0) * k.xy;
    q = q - vec2f(clamp(q.x, -k.z * r, k.z * r), r);
    return length(q) * sign(q.y);
}
```

**Sources:**
- [WGSL 2D SDF Primitives gist (munrocket)](https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4)
- [Glow Shader in Shadertoy — inspirnathan](https://inspirnathan.com/posts/65-glow-shader-in-shadertoy/)

---

## 2. HDR Emission for Bloom — Bevy 0.18.1

**How bloom is triggered:** Any HDR color value with any channel > 1.0 will be picked up by Bevy's bloom post-process pass. The fragment shader simply outputs values > 1.0.

```wgsl
// This fragment output will trigger bloom:
return vec4<f32>(3.5, 1.2, 0.8, 1.0);  // red channel at 3.5 — very bright
```

**Camera setup for bloom in Bevy 0.18.1:**

The current API (confirmed from bloom_3d.rs example in v0.18.1) uses `Camera { hdr: true }` plus the `Bloom` component. The `Bloom` component has `#[require(Hdr)]` in the struct definition, meaning adding `Bloom` alone may auto-insert `Hdr`. However, workshop examples from Bevy Rustweek 2025 show explicitly setting `hdr: true`:

```rust
commands.spawn((
    Camera2d,
    Camera { hdr: true, ..default() },
    Bloom::NATURAL,  // or Bloom::default()
));
```

**Bloom struct fields (Bevy 0.18.1):**
```rust
pub struct Bloom {
    pub intensity: f32,                    // default: 0.15
    pub low_frequency_boost: f32,
    pub low_frequency_boost_curvature: f32,
    pub high_pass_frequency: f32,          // default: 1.0
    pub prefilter: BloomPrefilter,
    pub composite_mode: BloomCompositeMode,
    pub max_mip_dimension: u32,
    pub scale: Vec2,
}
```

**Presets:**
- `Bloom::NATURAL` — energy-conserving, subtle (recommended)
- `Bloom::OLD_SCHOOL` — 2000s-era glow aesthetic
- `Bloom::SCREEN_BLUR` — intense full-screen blur

**Module path:** `bevy::post_process::bloom::Bloom`

**Note:** PR #18873 "Split Camera.hdr out into a new component" was targeted at the 0.17 milestone but there is contradictory information about exact merge. The workshop code for Bevy 0.18 uses `Camera { hdr: true, ..default() }`. Use this pattern until confirmed otherwise.

**Emissive color example (to trigger bright bloom):**
```rust
Color::srgb(5.0, 1.0, 1.0)  // red channel at 5x — creates strong red bloom
```

**Sources:**
- [Bloom struct — docs.rs 0.18.1](https://docs.rs/bevy/0.18.1/bevy/post_process/bloom/struct.Bloom.html)
- [bloom_3d.rs example (v0.18.1)](https://github.com/bevyengine/bevy/blob/v0.18.1/examples/3d/bloom_3d.rs)
- [Bevy Workshop Bloom setup](https://vleue.github.io/bevy_workshop-rustweek-2025/6-visuals/bloom.html)

---

## 3. Spike/Angular Modifications for Glow Halo

**Technique: Polar coordinates + sine wave modulation**

To add `spike_count` radial energy spikes to the halo, modulate the effective SDF distance using polar angle:

```wgsl
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;

    // Base SDF distance
    let base_d = sdCircle(uv, 0.4);

    // Polar coordinates
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);

    // Angular modulation — spike_count full sine cycles around 360 degrees
    let spike_amount = 0.1;  // how far spikes extend
    let angular_mod = spike_amount * max(0.0, sin(f32(spike_count) * angle));

    // Modulate the SDF: spikes = regions where halo extends further
    // Subtract angular_mod from d so glow extends outward at spike peaks
    let d = base_d - angular_mod;

    let core = step(0.0, -d);
    let halo = exp(-max(d, 0.0) * material.halo_falloff);
    let intensity = core * material.core_brightness + halo;

    return vec4<f32>(material.color.rgb * intensity, max(core, halo));
}
```

**Key insight:** `sin(spike_count * angle)` creates `spike_count` lobes (full cycles) around the circle. Using `max(0, sin(...))` keeps only the positive lobes (outward spikes). Subtracting from `d` makes glow extend further in spike directions.

**For sharper spikes:** Use `pow(max(0, sin(N * angle)), k)` where higher `k` = sharper spike tips.

**Sources:**
- [Shadertoy glow with polar spikes (inspirnathan)](https://inspirnathan.com/posts/51-shadertoy-tutorial-part-5/)
- [GM Shaders SDF Tricks](https://mini.gmshaders.com/p/gm-shaders-mini-sdf-tricks)

---

## 4. Additive Blending in Material2d — Bevy 0.18.1

**CRITICAL FINDING: `AlphaMode2d` has NO Add variant.**

`AlphaMode2d` in Bevy 0.18.1 has exactly three variants:
```rust
pub enum AlphaMode2d {
    Opaque,
    Mask(f32),
    Blend,
}
```

There is no `AlphaMode2d::Add` or `AlphaMode2d::Additive`. To achieve additive blending, you MUST override `specialize()` in your `Material2d` implementation.

**`Material2d::specialize()` signature:**
```rust
fn specialize(
    descriptor: &mut RenderPipelineDescriptor,
    layout: &MeshVertexBufferLayoutRef,
    key: Material2dKey<Self>,
) -> Result<(), SpecializedMeshPipelineError>
```

**How to implement additive blending:**

```rust
use bevy::render::render_resource::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites,
};

impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glow.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Additive blend: src_color * 1 + dst_color * 1
        let additive_blend = BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        };

        if let Some(fragment) = &mut descriptor.fragment {
            for target in &mut fragment.targets {
                if let Some(target) = target {
                    target.blend = Some(additive_blend);
                    target.write_mask = ColorWrites::ALL;
                }
            }
        }
        Ok(())
    }
}
```

**What additive blending means visually:** The glow material adds its RGB values on top of whatever is behind it. Black (0,0,0) in the shader = transparent/no contribution. Bright values = light-on-top. Perfect for glows, halos, particle effects.

**WGSL implication:** With additive blending, the alpha channel in the fragment output doesn't control transparency in the usual sense. The color channels ARE the contribution. Output alpha 0.0 still contributes color additively. Design the shader to output 0.0 in regions that should be invisible (the SDF-based glow naturally approaches 0 far from the shape).

**wgpu types needed:**
- `BlendState` — `wgpu::BlendState` (re-exported via bevy render)
- `BlendComponent` — `wgpu::BlendComponent`
- `BlendFactor` — `wgpu::BlendFactor::One`, `BlendFactor::Zero`, etc.
- `BlendOperation` — `wgpu::BlendOperation::Add`

Import path in Bevy: `bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState}`

**Sources:**
- [AlphaMode2d — docs.rs 0.18.1](https://docs.rs/bevy/0.18.1/bevy/sprite_render/enum.AlphaMode2d.html)
- [Material2d trait — docs.rs 0.18.1](https://docs.rs/bevy/0.18.1/bevy/sprite_render/trait.Material2d.html)
- [mesh2d_manual.rs example](https://github.com/bevyengine/bevy/blob/main/examples/2d/mesh2d_manual.rs)
- [BlendComponent — docs.rs wgpu](https://docs.rs/wgpu/latest/wgpu/struct.BlendComponent.html)

---

## 5. Dissolve/Disintegrate Shader

**Standard technique:** noise threshold with `discard`.

```wgsl
#import noisy_bevy::simplex_noise_2d

struct DissolveMaterial {
    threshold: f32,  // 0.0 = fully visible, 1.0 = fully dissolved
    color: vec4<f32>,
}

@group(2) @binding(0)
var<uniform> material: DissolveMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample noise at UV coordinates
    let noise_val = simplex_noise_2d(in.uv * 8.0);  // scale controls grain size
    let normalized = (noise_val + 1.0) / 2.0;       // map [-1,1] → [0,1]

    // Discard fragments below threshold
    if normalized < material.threshold {
        discard;
    }

    // Optional: add an edge glow at the dissolve boundary
    let edge_width = 0.05;
    if normalized < material.threshold + edge_width {
        let edge_t = (normalized - material.threshold) / edge_width;
        // Bright edge glow (HDR values trigger bloom)
        return vec4<f32>(material.color.rgb * 3.0, 1.0) * (1.0 - edge_t);
    }

    return material.color;
}
```

**noisy_bevy integration (Bevy 0.18.1 compatible — version 0.13.0):**

```rust
// Cargo.toml:
// noisy_bevy = "0.13"

use noisy_bevy::NoisyShaderPlugin;

app.add_plugins(NoisyShaderPlugin);
```

```wgsl
#import noisy_bevy::simplex_noise_2d
// Available: simplex_noise_2d(p: vec2<f32>) -> f32  — returns [-1, 1]
// Available: fbm_simplex_2d(p: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32
```

**Animating dissolve:** Add a `threshold` field to the material, update it each frame via system. Animate 0.0 → 1.0 over the desired duration.

```rust
// System to animate dissolve:
fn animate_dissolve(
    time: Res<Time>,
    mut materials: ResMut<Assets<DissolveMaterial>>,
    query: Query<(&Handle<DissolveMaterial>, &DissolveTimer)>,
) {
    for (handle, timer) in &query {
        if let Some(mat) = materials.get_mut(handle) {
            mat.threshold = timer.elapsed / timer.duration;
        }
    }
}
```

**Material swapping vs. modifying:** You do NOT need to swap materials. You add the dissolve material at spawn time and animate the `threshold` uniform. When `threshold >= 1.0`, despawn the entity. Alternatively, spawn with the original material and swap to the dissolve material when destruction begins.

**`discard` in WGSL:** Fully supported. Per the WebGPU spec, `discard` terminates the fragment invocation and discards the fragment output. All GPU implementations support it.

**Sources:**
- [noisy_bevy crate](https://github.com/johanhelsing/noisy_bevy)
- [noisy_bevy docs.rs](https://docs.rs/noisy_bevy/latest/noisy_bevy/)
- [PlayCanvas dissolve tutorial (technique reference)](https://developer.playcanvas.com/tutorials/custom-shaders/)
- [WebGPU discard semantics](https://github.com/gpuweb/gpuweb/issues/361)

---

## 6. Mesh Splitting (Split Primitive)

**Two approaches:**

### Option A: Shader-based clip planes (simpler, no separate meshes)

Use two material instances on two overlapping meshes, each discarding one side:

```wgsl
// Half A — keep fragments above the split axis
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // split_axis: 0 = horizontal, 1 = vertical
    // split_offset: world-space position of the split
    if in.world_position.y < material.split_y {
        discard;
    }
    return material.color;
}

// Half B — keep fragments below the split axis
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.world_position.y > material.split_y {
        discard;
    }
    return material.color;
}
```

**Limitation:** The two halves are still the same mesh — they cannot physically separate or move independently. Good for a visual split flash effect, not for physically separating shards.

### Option B: CPU mesh slicing (independent shard entities)

For halves that physically separate:

1. At split time, compute the clip plane in mesh-local space.
2. Iterate the mesh's vertex/index buffer.
3. For each triangle: classify vertices as above/below plane. For triangles that straddle the plane, compute intersection points and create new vertices.
4. Build two new `Mesh` assets from the resulting vertex sets.
5. Spawn two new entities, each with one half-mesh, apply diverging velocities.

**Bevy mesh mutation API:**
```rust
// Read existing mesh vertices:
let positions: &[[f32; 3]] = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    .unwrap()
    .as_float3()
    .unwrap();

// Create new mesh from scratch:
let mut half_a = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
half_a.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_vertices);
half_a.insert_indices(Indices::U32(new_indices));
```

**Practical verdict for this project:**
- Shader clip approach: trivial to implement, no independent motion — suitable for a "flash split" visual
- CPU mesh slicing: complex (30-100 lines of geometry math), enables independent shard physics — suitable for the Split destruction primitive if halves need to fly apart

For a brickbreaker context where Split cells have two halves that scatter, CPU mesh slicing is the right approach. The geometry math is well-understood (Sutherland-Hodgman clipping for convex polygons) but must be written from scratch or found in a crate.

**Sources:**
- [Real-time fracturing — diva-portal (2D Voronoi paper)](https://www.diva-portal.org/smash/get/diva2:1452512/FULLTEXT02)
- Bevy mesh API: `Mesh::attribute()`, `Mesh::insert_attribute()`

---

## 7. Mesh Fracture (Fracture Primitive)

**Three options compared:**

### Option A: Pre-computed Voronoi at spawn time (RECOMMENDED)

At the time the cell entity is spawned with its visual:
1. Pick N random Voronoi seed points within the cell's bounding shape.
2. Compute the 2D Voronoi diagram (Fortune's algorithm, O(n log n)).
3. For each Voronoi cell, create a `Mesh` asset (the polygon clipped to the cell boundary).
4. Store the N mesh handles as a component on the entity.

At fracture time:
1. Despawn the main cell entity.
2. Spawn N shard entities, each with one pre-computed mesh handle.
3. Apply diverging velocities (outward from fracture center + random angular velocity).

**Advantages:** Zero computation at fracture time. All the expensive geometry work happens at cell spawn (when a small hitch is acceptable). Deterministic given the same seed.

**Implementation complexity:** Medium. Need a Fortune's algorithm implementation or use a crate. The `voronator` crate (Rust) implements Fortune's algorithm for 2D Voronoi.

```toml
voronator = "0.2"  # check latest version
```

### Option B: Shader Voronoi (pure shader, no CPU geometry)

Assign each fragment a "shard ID" by running a Voronoi noise function in the shader:

```wgsl
fn voronoi_cell_id(p: vec2f, num_cells: f32) -> f32 {
    // Returns a cell ID in [0, 1] based on position
    let scaled_p = p * num_cells;
    let i = floor(scaled_p);
    let f = fract(scaled_p);
    var min_dist = 1e10;
    var cell_id = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let neighbor = vec2f(f32(x), f32(y));
            let point = hash2(i + neighbor);  // random point in cell
            let dist = length(f - point - neighbor);
            if dist < min_dist {
                min_dist = dist;
                cell_id = hash1(i + neighbor);
            }
        }
    }
    return cell_id;
}
```

Each shard gets a uniform velocity offset. Use `cell_id` to look up velocity from a uniform array. Fragments of the same cell translate together.

**Limitation:** The shards cannot physically separate into distinct entities — they remain one mesh with fragments moving by different UV offsets. Works visually for a "disintegrating in place" effect but not for shards that ricochet off walls.

**When to use:** For a purely visual "shattering glow" effect where shards fly apart and then disappear (fade out), without needing physics.

### Option C: Runtime Voronoi compute shader

Compute the Voronoi tessellation at fracture time using a GPU compute shader. Overkill for 2D brickbreaker — Fortune's algorithm on CPU for 8-16 shards takes < 1ms and is far simpler than a compute shader pipeline.

**Practical recommendation for this project:**

- **For "Fracture" destruction primitive:** Use Option A (pre-computed Voronoi at spawn). Compute at `AttachVisuals` time. 8-16 shards per cell. Store as `Vec<Handle<Mesh>>` component. At fracture, spawn shards with diverging velocities.
- **For a purely visual disintegration that doesn't need independent physics:** Option B (shader Voronoi) is much simpler and may look just as good.

**voronator crate for CPU Voronoi:**
```toml
voronator = "0.2"
```
```rust
use voronator::delaunator::Point;
use voronator::VoronoiDiagram;

let seeds: Vec<Point> = (0..num_shards)
    .map(|_| Point { x: rng.random(), y: rng.random() })
    .collect();
let diagram = VoronoiDiagram::from_tuple(&(-1.0, -1.0), &(1.0, 1.0), &seeds)
    .unwrap();
```

**Sources:**
- [Real-time fracturing paper (Voronoi 2D)](https://www.diva-portal.org/smash/get/diva2:1452512/FULLTEXT02)
- [Alan Zucconi — To Voronoi and Beyond](https://www.alanzucconi.com/2015/02/24/to-voronoi-and-beyond/)
- [Voronoi fracture demo (nikhilnxvverma1)](https://nikhilnxvverma1.github.io/voronoi-fracture/)
- [Inigo Quilez — distance functions](https://iquilezles.org/articles/distfunctions/)

---

## Summary Table

| Topic | Technique | Complexity | Bevy 0.18.1 API |
|-------|-----------|------------|-----------------|
| Core+halo glow | SDF + exp falloff | Low | Custom `Material2d` + WGSL |
| HDR bloom trigger | Output channels > 1.0 | Trivial | `Camera { hdr: true }` + `Bloom::NATURAL` |
| Angular spikes | Polar coords + sin modulation | Low | Modify SDF in shader |
| Additive blending | `specialize()` override with custom `BlendState` | Medium | No `AlphaMode2d::Add` — must use `specialize()` |
| Dissolve effect | Noise + discard + animated threshold | Low | `noisy_bevy` 0.13 (Bevy 0.18 compat) |
| Mesh split | CPU polygon clipping or shader clip-plane | Medium/Low | `Mesh::insert_attribute()` for CPU; `discard` for shader |
| Mesh fracture | Pre-computed Voronoi at spawn + shard entities | Medium | `voronator` crate for Voronoi; Bevy mesh API |

---

## Key Gotchas

1. **`AlphaMode2d::Add` does not exist** in Bevy 0.18.1. Do not use it. Use `specialize()` with a custom `BlendState`.

2. **Bloom requires `Camera { hdr: true }`** on the camera entity (Bevy 0.18.1). The `Bloom` component has `#[require(Hdr)]` in its definition but the workshop examples show explicitly setting `hdr: true` on the Camera. Add both to be safe.

3. **`Bloom` struct is in `bevy::post_process::bloom`** — not `bevy::core_pipeline::bloom` as in older versions. The struct is named `Bloom`, not `BloomSettings`.

4. **`noisy_bevy` version 0.13.0 targets Bevy 0.18** specifically. Use this for noise in dissolve shaders. The `bevy_shader_utils` crate's version support is less clear.

5. **SDF functions are not built into Bevy's WGSL**. You must include them in your shader file or as a separate `.wgsl` asset that you `#import`.

6. **`discard` in WGSL** works exactly like GLSL `discard` — fully supported in WebGPU.

7. **For additive glow with bloom**: output HDR color values (channels > 1.0) in the fragment shader AND use additive `BlendState` in `specialize()`. The bloom pass will pick up the HDR values from the framebuffer regardless of blend mode.
