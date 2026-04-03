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

// Dissolve (for Disintegrate — threshold 0.0 = off, 1.0 = fully dissolved):
if material.dissolve_threshold > 0.0 {
    let noise = (simplex_noise_2d(uv * 8.0) + 1.0) / 2.0;
    if noise < material.dissolve_threshold { discard; }
    // Burning edge glow at dissolve boundary:
    let edge = 0.05;
    if noise < material.dissolve_threshold + edge {
        let t = (noise - material.dissolve_threshold) / edge;
        return vec4(2.0, 0.8, 0.2, 1.0) * (1.0 - t);  // HDR orange edge
    }
}

let intensity = core * material.core_brightness + halo;
return vec4(material.color.rgb * intensity, max(core, halo));
```

**Dissolve integration:** No separate `dissolve.wgsl` shader. The `dissolve_threshold` uniform on `EntityGlowMaterial` defaults to 0.0 (off — zero cost, the `if` is not entered). When the Disintegrate primitive fires, it animates `dissolve_threshold` from 0.0→1.0 over the specified duration. Same material, no swap. On `RecipeComplete`, the game despawns the entity.

SDF primitives: `sdCircle`, `sdBox`, `sdHexagon`, etc. from the [munrocket WGSL gist](https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4). Shape type selects which SDF to evaluate.

## Aura Shader (single `AuraMaterial`, variant-switched)

Single `aura.wgsl` shader with a `variant` uniform selecting the rendering algorithm. One `AuraMaterial` type, one `Material2dPlugin` registration. See [types.md](types.md) for the consolidated material design.

| Variant | Visual | Enum |
|---------|--------|------|
| 0 (ShieldShimmer) | Defensive energy field | `Aura::ShieldShimmer { ... }` |
| 1 (TimeDistortion) | Rippling time-echo afterimage | `Aura::TimeDistortion { ... }` |
| 2 (PrismaticSplit) | Rainbow edge refractions | `Aura::PrismaticSplit { ... }` |

## Trail Rendering (three distinct techniques, NOT a single material)

Unlike auras, trail variants use **different rendering techniques**:

| Trail Variant | Technique | Shader/Material |
|--------------|-----------|----------------|
| ShieldEnergy | Mesh ribbon (TriangleStrip), ring buffer of positions | `trail_ribbon.wgsl` + `TrailRibbonMaterial` (SrcAlpha + One additive) |
| Afterimage | Pre-spawned sprite entity pool, repositioned each frame | Standard `Sprite` with alpha fade, no custom shader |
| PrismaticSplit | 3 overlapping ShieldEnergy ribbons with RGB channel tint | Same `TrailRibbonMaterial` × 3 |

See `docs/architecture/rendering/rantzsoft_vfx.md` — Trail Rendering section for full details.

## Primitive Shaders

| Shader | Used By |
|--------|---------|
| `beam.wgsl` | Beam, AnchoredBeam |
| `ring.wgsl` | ExpandingRing, EnergyRing, ExpandingDisc |
| `glow_line.wgsl` | GlowLine (wall borders, shield barrier base) |
| `timer_wall.wgsl` | Timer wall HUD gauge glow |
| `grid.wgsl` | Background playfield grid |
| `shield.wgsl` | Shield barrier energy field |

### ring.wgsl — Ring and Disc

Shared shader for ExpandingRing, EnergyRing, and ExpandingDisc. SDF-based: `sdCircle(uv, radius)` with configurable rendering mode via a `filled` uniform toggle.

- **Ring mode** (`filled = 0`): core glow at distance == 0 (ring boundary), exponential falloff both inward and outward. `thickness` controls the band width.
- **Disc mode** (`filled = 1`): opaque center, glowing edge. The disc interior is the bright part (`step(0.0, -d)` for solid fill inside), with exponential halo glow falloff outside.

Radius animates from 0 to `max_radius` over lifetime. HDR color for bloom.

### beam.wgsl — Beam Primitive

SDF-on-quad approach. The beam is an `sdCapsule(uv, start, end, width)` — a line segment with rounded ends. Core brightness along the beam center, exponential glow falloff perpendicular.

**Uniforms:** `direction` (normalized Vec2), `range` (beam length), `width` (current width), `max_width` (initial width), `hdr`, `color`, `shrink_progress` (0.0–1.0), `afterimage_alpha` (0.0–1.0).

**Lifecycle:**
1. Beam spawns at `max_width`. `shrink_progress = 0.0`.
2. Over `shrink_duration` seconds: `shrink_progress` ramps 0.0→1.0. Current `width = max_width * (1.0 - shrink_progress)`. The beam narrows from full width to zero — it lingers visibly, not instant.
3. After shrink completes: if `afterimage_duration > 0`, the beam enters afterimage phase. `afterimage_alpha` fades 1.0→0.0 over `afterimage_duration`. The beam renders as a dim ghost at final width.
4. Despawn when afterimage fades to 0 (or immediately if no afterimage).

**Fragment shader:**
```wgsl
let d = sdCapsule(uv, vec2(0.0), direction * range, width);
let core = step(0.0, -d) * hdr;
let halo = exp(-max(d, 0.0) * 3.0) * hdr * 0.5;
let alpha = mix(1.0, afterimage_alpha, afterimage_phase);
return vec4(color.rgb * (core + halo) * alpha, max(core, halo) * alpha);
```

Used by: `Beam` PrimitiveStep (one-shot directional beam), `AnchoredBeam` (persistent entity-tracking beam — recomputes start/end each frame from Source/Target GlobalTransform).

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
| `entity_glow.wgsl` (dissolve mode) | Disintegrate primitive | No separate dissolve shader. `entity_glow.wgsl` has a `dissolve_threshold` uniform (default 0.0 = off). When >0, samples noise and discards fragments below threshold. Burning-edge glow at boundary. Animated 0→1 over duration by the Disintegrate primitive handler. No material swap needed — same `EntityGlowMaterial`, just animate one uniform. |

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
| `flash.wgsl` | ScreenFlash | Before Tonemapping (HDR) | `FullscreenMaterial` with `node_edges() = [StartMainPassPostProcessing, FlashLabel, Tonemapping]`. **Additive blend in shader** (not GPU blend state): `return vec4(scene_color.rgb + flash.rgb, scene_color.a)`. HDR pipeline selected automatically. Do NOT use `Node2d::Bloom` as anchor. |
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
- `sdf.wgsl` — 2D SDF primitives (based on [munrocket SDF gist](https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4) and [Inigo Quilez SDF reference](https://iquilezles.org/articles/distfunctions2d/))
- `voronoi.wgsl` — 2D Voronoi cell ID for shader-based fracture

For CPU-side Voronoi (pre-computed fracture shards), custom Rust implementation in the crate. No external crates for noise or Voronoi.

### SDF Function Sources

The `sdf.wgsl` file contains these functions:

| Function | Source | Used By |
|----------|--------|---------|
| `sdCircle(p, r)` | munrocket gist | Circle shape, ExpandingRing/Disc |
| `sdBox(p, b)` | munrocket gist | Rectangle shape |
| `sdRoundedBox(p, b, r)` | [Inigo Quilez](https://iquilezles.org/articles/distfunctions2d/) | RoundedRectangle shape. Corners rounded by `r` (vec4 for per-corner radii, or uniform float). |
| `sdHexagon(p, r)` | munrocket gist | Hexagon shape |
| `sdRegularPolygon(p, r, n)` | Inigo Quilez | Octagon (n=8) |
| `sdRhombus(p, b)` | Inigo Quilez | Diamond shape |
| `sdSegment(p, a, b)` | munrocket gist | GlowLine, Beam |
| `sdCapsule(p, a, b, r)` | munrocket gist | Beam with rounded ends |
| `sdPolygon(p, verts, n)` | Custom implementation | Shield, Angular, Crystalline, Custom shapes |

**`sdPolygon` implementation:** Iterates edges of a convex polygon, computing the minimum distance to any edge. For built-in shapes (Shield, Angular, Crystalline), vertex lists are `const` arrays hardcoded in WGSL within the `switch` cases — no uniform data needed. For `Custom(CustomShape)`, vertices are passed as a uniform `array<vec2<f32>, 16>` with a `vertex_count: u32` uniform. The loop iterates `vertex_count` edges (WGSL requires a compile-time loop bound, so the loop always runs 16 iterations with early-out via `if (i >= vertex_count) { break; }`).
