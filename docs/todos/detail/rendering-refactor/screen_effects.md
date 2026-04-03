# Screen Effects

## Post-Processing Pipeline Order

```
1. Scene render (meshes, materials, particles)
2. Screen flash (additive HDR overlay — BEFORE tonemapping, operates on HDR data)
3. Tonemapping (HDR → LDR)
4. Screen distortion (radial warp — on LDR output)
5. Chromatic aberration (RGB offset — on LDR output)
6. Desaturation (color → monochrome — on LDR output)
7. Vignette (gradient from edges — on LDR output)
8. CRT/scanline (if enabled — last shader pass)
9. Screen shake (camera offset — not a shader, applied outside render graph)
```

Note: Bloom is handled by Bevy's built-in `Bloom` component on the camera. HDR values >1.0 from scene rendering and from the screen flash shader bloom naturally. Flash is placed before tonemapping so it operates on HDR data.

## FullscreenMaterial Implementation (Bevy 0.18.1)

Each post-processing effect is a `Component` on the camera entity implementing the `FullscreenMaterial` trait. `node_edges()` controls render graph ordering. `ViewTarget::post_process_write()` ping-pongs textures for chaining between effects.

**Verified by spike** (see `.claude/specs/fullscreen-material-spike.md`): Multiple FullscreenMaterial types chain correctly on one camera. No custom ViewNode fallback needed.

### Core2d Render Graph (with `features = ["2d"]`)

```
StartMainPass
  → MainOpaquePass
  → MainTransparentPass
  → EndMainPass
  → StartMainPassPostProcessing
  → [ScreenFlash — placed here via node_edges]
  → Tonemapping
  → [Distortion → ChromaticAberration → Desaturation → Vignette → CRT — chained here]
  → EndMainPassPostProcessing
  → Upscaling
```

### Effect Placement

| Effect | Position | `node_edges()` anchor | Notes |
|--------|----------|----------------------|-------|
| Screen flash | Pre-tonemapping (HDR) | `[StartMainPassPostProcessing, FlashLabel, Tonemapping]` | Additive blend in shader (see below) |
| Radial distortion | Post-tonemapping (LDR) | `[Tonemapping, DistortionLabel, ChromaticLabel]` | Geometric warp |
| Chromatic aberration | Post-tonemapping | `[DistortionLabel, ChromaticLabel, DesaturationLabel]` | RGB shift |
| Desaturation | Post-tonemapping | `[ChromaticLabel, DesaturationLabel, VignetteLabel]` | Color → monochrome |
| Vignette | Post-tonemapping | `[DesaturationLabel, VignetteLabel, CrtLabel]` | Edge darkening |
| CRT/scanlines | Last | `[VignetteLabel, CrtLabel, EndMainPassPostProcessing]` | Final overlay |
| Collapse/rebuild | During transitions only | `[Tonemapping, CollapseLabel, EndMainPassPostProcessing]` | Replaces normal post chain during transitions |
| Screen shake | Camera Transform offset | N/A | Not a shader pass |

Each effect defines a concrete `RenderLabel` type (e.g., `struct FlashLabel;`) so other effects can reference it in their `node_edges()`. Overlapping edge declarations are safe (`EdgeAlreadyExists` is silently ignored).

### Critical: No `Node2d::Bloom` in node_edges

**Do NOT use `Node2d::Bloom` as an anchor.** With `features = ["2d"]` only, the Bloom node is not registered in the Core2d graph. Using it panics at startup. Use `Node2d::Tonemapping` as the downstream anchor for pre-tonemapping effects.

### Critical: No Custom Blend State

`FullscreenMaterial` hardcodes `blend: None` in its pipeline. There is no `specialize()` override. **Additive blending must be done in the fragment shader**, not via GPU blend state:

```wgsl
// screen_flash.wgsl — shader-side additive blend
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(screen_texture, texture_sampler, in.uv);
    let flash = material.color * material.intensity;
    return vec4(scene_color.rgb + flash.rgb, scene_color.a);
}
```

This applies to screen flash only. All other effects (distortion, chromatic, desaturation, vignette, CRT) are replace-style operations that work with `blend: None`.

### Key Details

- **Trait bounds:** `Component + ExtractComponent + Clone + Copy + ShaderType + WriteInto + Default`
- **Import path:** `bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin}`
- **HDR support:** Plugin auto-creates both LDR and HDR pipelines, selects at runtime via `view_target.is_hdr()`. Pre-tonemapping effects get `Rgba16Float` automatically.
- **Disable:** Set `intensity = 0.0` (removing the component does NOT remove the render node; the node's `Added<T>` trigger fires only once)
- **Dynamic uniforms:** Mutate the component on the camera entity — `ExtractComponentPlugin` handles GPU upload
- **WGSL bindings:** `@group(0) @binding(0)` screen texture, `@binding(1)` sampler, `@binding(2)` uniform struct
- **WGSL import:** `#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput`
- **std140 alignment:** Single `f32` must pad to 16 bytes (add `_padding: Vec3`)
- **One-frame delay:** Node is added on `Added<T>` in `ExtractSchedule`, so the effect renders starting the frame after the component is inserted. If this causes a visible pop, consider inserting the component at startup with `intensity = 0.0`.

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
