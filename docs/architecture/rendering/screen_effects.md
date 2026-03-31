# Screen Effects

## Post-Processing Pipeline Order

```
1. Scene render (meshes, materials, particles)
2. Screen flash (additive HDR overlay — BEFORE bloom so it gets bloomed naturally)
3. Bloom pass (HDR → glow halos)
4. Tonemapping (HDR → LDR)
5. Screen distortion (radial warp — on LDR output)
6. Chromatic aberration (RGB offset — on LDR output)
7. Desaturation (color → monochrome — on LDR output)
8. Vignette (gradient from edges — on LDR output)
9. CRT/scanline (if enabled — last shader pass)
10. Screen shake (camera offset — not a shader, applied outside render graph)
```

## FullscreenMaterial Implementation (Bevy 0.18)

Each post-processing effect is a component on the camera entity implementing the `FullscreenMaterial` trait. `node_edges()` controls ordering. `ViewTarget::post_process_write()` ping-pongs textures for chaining.

```
Node2d execution order:
  MsaaWriteback → MainOpaquePass → MainTransparentPass
    → Bloom (requires Hdr component on camera)
    → PostProcessing
    → Tonemapping → Fxaa → Upscaling
```

**Effect placement:**

| Effect | Position | Reason |
|--------|----------|--------|
| Screen flash | Before Bloom via `FullscreenMaterial::node_edges()` | `node_edges()` returns `[StartMainPassPostProcessing, self_label, Node2d::Bloom]`. Flash gets bloomed naturally. No custom ViewNode needed. |
| Radial distortion | After Tonemapping | Geometric warp on LDR |
| Chromatic aberration | After Tonemapping | RGB shift on LDR |
| Desaturation | After Tonemapping | Color transform on LDR |
| Vignette | After Tonemapping | Edge darkening on LDR |
| CRT/scanlines | Last before EndMainPassPostProcessing | Final overlay |
| Screen shake | Camera Transform offset | Not a shader pass |

**Key details:**
- `FullscreenMaterial` requires `Component + Copy + ShaderType + Default`
- Disable by setting `intensity = 0.0` (removing the component does NOT remove the render node)
- Dynamic uniforms: mutate the component on the camera entity — `ExtractComponentPlugin` handles GPU upload
- WGSL: `@group(0) @binding(0)` screen texture, `@binding(1)` sampler, `@binding(2)` uniform struct
- `#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput`
- std140 alignment: single `f32` must pad to 16 bytes

## Screen Shake

Four tiers: Micro (1-2px, 1-2f), Small (3-5px, 3-4f), Medium (6-10px, 4-6f), Heavy (12-20px, 6-10f). Directional, exponential decay, stacking with cap. Configurable multiplier.

## Danger Vignette (game-specific, in run/ domain)

Continuous — driven by timer state and lives count. Red-orange gradient from edges. Pulses at danger-scaled rhythm. Never >50% opacity. The crate provides the vignette shader; the `run/` domain drives it via modifier messages.

## Screen Effect Stacking

| Effect | Stacking Behavior |
|--------|-------------------|
| ScreenShake | Multiple shakes combine (stacking with cap). Decays independently per source. |
| ScreenFlash | Latest flash overrides. Only one active at a time — a second flash replaces the first. |
| RadialDistortion | Additive — up to 16 simultaneous sources in the distortion buffer. Oldest replaced when full. |
| ChromaticAberration | Latest overrides. One active at a time. |
| Desaturation | Latest target factor wins. Smooth transition to new target. |
| VignettePulse | Additive — multiple pulses combine up to 50% max opacity. |
| SlowMotion | Latest factor wins. Ramp system smooths the transition. |

## Distortion Buffer

Fixed-size array of 16 distortion sources. Oldest replaced when full.

```wgsl
struct DistortionSettings {
    sources: array<DistortionSource, 16>,
    active_count: u32,
    _pad: vec3<f32>,
}

struct DistortionSource {
    origin: vec2<f32>,
    radius: f32,
    intensity: f32,
}
```

## Configuration (game-specific, in shared/)

`GraphicsDefaults` RON + `GraphicsConfig` resource via `rantzsoft_defaults`. Stores: CRT toggle + intensity, grid density, bloom settings, screen shake multiplier, chromatic aberration multiplier. Registered as a shared resource in `game.rs`.
