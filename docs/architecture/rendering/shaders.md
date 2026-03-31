# Concept Shaders

All shaders live in `rantzsoft_vfx`. Separate shaders per visual concept because the math is genuinely different. Within a concept, variants are parameter sets.

**Research**: See [research/glow-and-destruction-shaders.md](research/glow-and-destruction-shaders.md) for full Bevy 0.18.1 API details and shader techniques.

## Additive Blending (applies to all entity/primitive shaders)

Bevy 0.18 `AlphaMode2d` has NO `Add` variant. All additive shaders require `Material2d::specialize()` overriding `BlendState` with `BlendFactor::One` for dst_factor. Black (0,0,0) = no contribution, bright values = light-on-top. SDF glow naturally produces 0 far from the shape.

## Entity Shader

| Shader | What It Renders | Technique |
|--------|----------------|-----------|
| `entity_glow.wgsl` | Core + halo glow for any entity | **SDF-based**: compute distance from shape boundary, exponential falloff for halo. HDR values > 1.0 produce bloom. |

### entity_glow Algorithm

```wgsl
// Per-fragment:
let uv = in.uv * 2.0 - 1.0;
let d = sdf_shape(uv, shape_type);     // SDF distance from shape boundary
let core = step(0.0, -d);               // 1.0 inside shape, 0.0 outside
let halo = exp(-max(d, 0.0) * material.halo_falloff);  // exponential glow falloff

// Spike modulation (for piercing):
let angle = atan2(uv.y, uv.x);
let spike = max(0.0, sin(f32(material.spike_count) * angle));
let d_spiked = d - spike * 0.08;        // push glow outward at spikes

let intensity = core * material.core_brightness + halo;
return vec4(material.color.rgb * intensity, max(core, halo));
```

SDF primitives: `sdCircle`, `sdBox`, `sdHexagon`, etc. from the [munrocket WGSL gist](https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4). Shape type selects which SDF to evaluate.

## Aura Shader (single `AuraMaterial`, variant-switched)

Single `aura.wgsl` shader with a `variant` uniform selecting the rendering algorithm. One `AuraMaterial` type, one `Material2dPlugin` registration. See [types.md](types.md) for the consolidated material design.

| Variant | Visual | Enum |
|---------|--------|------|
| 0 (ShieldShimmer) | Defensive energy field | `Aura::ShieldShimmer { ... }` |
| 1 (TimeDistortion) | Rippling time-echo afterimage | `Aura::TimeDistortion { ... }` |
| 2 (PrismaticSplit) | Rainbow edge refractions | `Aura::PrismaticSplit { ... }` |

## Trail Shaders (one per `Trail` variant)

| Shader | Visual | Enum Variant |
|--------|--------|--------------|
| `trail_shield_energy.wgsl` | Solid protective wake | `Trail::ShieldEnergy { ... }` |
| `trail_afterimage.wgsl` | Fading position copies | `Trail::Afterimage { ... }` |
| `trail_prismatic_split.wgsl` | Spectral color separation | `Trail::PrismaticSplit { ... }` |

## Primitive Shaders

| Shader | Used By |
|--------|---------|
| `beam.wgsl` | Beam primitive |
| `ring.wgsl` | ExpandingRing, EnergyRing, ExpandingDisc |
| `glow_line.wgsl` | GlowLine (wall borders, shield barrier base) |
| `timer_wall.wgsl` | Timer wall HUD gauge glow |
| `grid.wgsl` | Background playfield grid |
| `shield.wgsl` | Shield barrier energy field |

### ring.wgsl — Ring and Disc

Shared shader for ExpandingRing, EnergyRing, and ExpandingDisc. SDF-based: `sdCircle(uv, radius)` with configurable rendering mode via a `filled` uniform toggle.

- **Ring mode** (`filled = 0`): core glow at distance == 0 (ring boundary), exponential falloff both inward and outward. `thickness` controls the band width.
- **Disc mode** (`filled = 1`): alpha = 1.0 inside the circle (inverted — transparent core for the center, opaque fill around it). Wait — per user: "invert the filled core, should be alpha in the middle." So: **opaque center, transparent/glowing edge**. The disc interior is the bright part, with glow falloff outside. `step(0.0, -d)` for solid fill inside + exponential halo outside.

Radius animates from 0 to `max_radius` over lifetime. HDR color for bloom.

### glow_line.wgsl — Glowing Line Segment

**SDF-on-quad** approach, same as entity_glow. Oversized quad with `sdSegment(uv, start, end)` — the line SDF. Core brightness along the line center, exponential glow falloff perpendicular to the line.

Uniforms: `start_offset`, `end_offset` (line endpoints in local space), `width`, `hdr`, `color`, `shimmer_speed`. Shimmer via time-based sine modulation of intensity: `intensity *= 1.0 + sin(globals.time * shimmer_speed) * 0.2`.

### timer_wall.wgsl — Gauge Glow

Generic "gauge glow bar" primitive — game-agnostic, usable for any horizontal fill-level indicator.

Uniforms: `fill_level` (0.0–1.0), `temperature` (color shift), `pulse_speed`, `intensity`. Fragment: full glow left of `fill_level * uv.x`, fading glow right of it. Color lerps between two endpoint colors based on `temperature`. Pulse via `sin(globals.time * pulse_speed)` modulating intensity.

### grid.wgsl — Playfield Grid

Single quad covering the playfield area. Fragment shader computes grid lines from UV coordinates.

Uniforms: `line_spacing`, `line_thickness`, `color` (from RunTemperature), `glow_intensity`. Grid lines are thin bright lines with subtle glow falloff. RadialDistortion effects warp the grid naturally (they warp UVs in the distortion post-process, which occurs after the grid renders).

Configurable density via `VfxConfig` (passed as uniform). See [temperature.md](temperature.md) for how color is driven.

### shield.wgsl — Shield Barrier Energy Field

Custom `Material2d` shader for the shield barrier entity. Semi-transparent energy field with animated hexagonal pattern and procedural crack damage visualization.

Uniforms: `color` (pulsing white per DR-3), `hex_scale` (honeycomb cell size), `pulse_speed`, `intensity`, `crack_seeds: array<vec2<f32>, 5>` (world-space positions of damage cracks), `crack_count: u32` (0–5), `crack_radius: f32` (how far cracks have spread).

Algorithm:
1. Compute hexagonal tiling from UV coordinates (pointy-top hexagons)
2. Each hex cell has animated shimmer (phase offset from cell position)
3. For each crack seed: sample noise around the seed position, darken hex cells within `crack_radius`. More cracks = more dark regions.
4. On final charge (`crack_count == max`): dark regions cover 80%+, then the game fires `TriggerFracture` on the entity.

Additive blending via `specialize()`. Placed behind the breaker on a lower z-layer.

## Destruction Shaders

| Shader | Used By | Technique |
|--------|---------|-----------|
| `dissolve.wgsl` | Disintegrate primitive | Noise threshold ramp: sample custom simplex noise (`noise.wgsl`), `discard` fragments below threshold. Optional burning-edge glow at boundary. Animate threshold 0→1 over duration. No external deps. |

### Dissolve Algorithm

```wgsl
// Custom simplex noise in rantzsoft_vfx/assets/shaders/noise.wgsl (no external dep)
#import rantzsoft_vfx::noise::simplex_noise_2d

let noise = (simplex_noise_2d(in.uv * 8.0) + 1.0) / 2.0;
if noise < threshold { discard; }
// Optional: burning edge glow at boundary
let edge = 0.05;
if noise < threshold + edge {
    let t = (noise - threshold) / edge;
    return vec4(2.0, 0.8, 0.2, 1.0) * (1.0 - t);  // HDR orange edge
}
```

### Split Implementation

**Shader clip-plane approach** (two overlapping meshes, each discarding one side):
```wgsl
if in.world_position.y < material.split_y { discard; }
```

For physically separating halves: CPU mesh slicing via Sutherland-Hodgman polygon clipping (~40-60 lines of Rust). Spawn two entities with independent velocities.

### Fracture Implementation

**Shader Voronoi**: Assign each fragment a Voronoi cell ID in the shader. Each "shard" translates by a different velocity derived from the cell ID. Visually convincing — shards separate and scatter purely in the shader, no CPU mesh work needed. Sufficient for our needs (cell destruction VFX doesn't need independent shard physics).

```wgsl
let cell = voronoi_cell_id(in.uv, f32(material.shard_count));
let vel = hash_to_vec2(cell) * material.explosion_speed;
let displaced_uv = in.uv + vel * material.time;
// If displaced_uv is outside the original shape SDF: discard (shard has "left")
```

## Post-Processing Shaders

| Shader | Used By | Position in Pipeline | Implementation |
|--------|---------|---------------------|----------------|
| `flash.wgsl` | ScreenFlash | Before Bloom | `FullscreenMaterial` with `node_edges() = [StartMainPassPostProcessing, self, Node2d::Bloom]`. Fragment: `return scene_color + flash_color * flash_alpha`. **Must use `ViewTarget::TEXTURE_FORMAT_HDR`** for the pipeline color target (not bevy_default). |
| `distortion.wgsl` | RadialDistortion | After Tonemapping | FullscreenMaterial, 16-source fixed array uniform |
| `chromatic_aberration.wgsl` | ChromaticAberration | After Tonemapping | FullscreenMaterial |
| `desaturation.wgsl` | Desaturation | After Tonemapping | FullscreenMaterial |
| `vignette.wgsl` | Vignette | After Tonemapping | FullscreenMaterial |
| `collapse_rebuild.wgsl` | Collapse/Rebuild transition | Transition timing | FullscreenMaterial, tile-based radial converge/diverge |
| `crt.wgsl` | CRT overlay (off by default) | Last | FullscreenMaterial |

## Special Shaders

| Shader | Used By | Technique |
|--------|---------|-----------|
| `glitch_text.wgsl` | Highlight labels, titles, evolution names | Child overlay on Text2d entity |
| `holographic.wgsl` | Evolution rarity chip cards | Material2d on card background entity |

### glitch_text.wgsl — Glitch Text Overlay

**Approach:** Bevy `Text2d` entity renders the text with a monospace font (standard Bevy text layout). A **child overlay entity** (quad mesh with `GlitchMaterial`) sits on top, covering the text bounds. The overlay applies all glitch effects as a fragment shader — the Text2d itself is unmodified.

**GlitchMaterial uniforms:** `time`, `scanline_density`, `scanline_speed`, `chromatic_offset` (RGB channel UV shift), `jitter_intensity` (random block displacement), `punch_scale` (driven by Transform animation on parent, not shader).

**Fragment shader algorithm:**
1. Scanlines: `sin(uv.y * scanline_density + time * scanline_speed)` modulates alpha in horizontal bands
2. Chromatic split: sample background at `uv`, `uv + vec2(chromatic_offset, 0.0)`, `uv - vec2(chromatic_offset, 0.0)` for R, G, B channels
3. Jitter: hash-based block displacement — divide UV into blocks, offset some blocks horizontally based on `hash(block_id + floor(time * jitter_rate))`
4. Composite with additive blend

**Punch scale** is NOT a shader effect — it's a `Transform` scale animation on the parent entity (scale up to ~1.3x then back to 1.0 over ~0.15s). Managed by the recipe system or a simple tween.

**Lifetime:** GlitchText entities auto-despawn after `duration` seconds. The text + overlay are children of a parent entity that despawns.

### holographic.wgsl — Holographic Card Effect

Applied to Evolution-rarity chip card backgrounds. Simulates holographic foil shimmer — rainbow color shifting based on view angle (approximated from UV + time).

**Uniforms:** `base_color`, `shimmer_speed`, `spectral_intensity`, `scan_line_frequency`.

Fragment: base color + prismatic overlay that shifts hue based on `uv.x + uv.y + time * shimmer_speed`. Additive blend for the shimmer layer.

## Shared Shader Utilities (in rantzsoft_vfx)

The crate includes custom WGSL utility files (no external dependencies):

- `noise.wgsl` — simplex noise 2D/3D (based on the [munrocket MIT-licensed WGSL gist](https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39))
- `sdf.wgsl` — 2D SDF primitives: circle, box, hexagon, rounded box, polygon (based on [munrocket SDF gist](https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4))
- `voronoi.wgsl` — 2D Voronoi cell ID for shader-based fracture

For CPU-side Voronoi (pre-computed fracture shards), custom Rust implementation in the crate. No external crates for noise or Voronoi.
