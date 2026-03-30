# 5d: Post-Processing Pipeline

**Goal**: Build the post-processing render infrastructure that screen effects, combat VFX, and entity shaders will use. This is the rendering backbone.

## What to Build

### 1. Bloom Tuning System

Current: `Bloom::default()` on camera with no per-entity control.

Target:
- Configurable bloom settings (intensity, threshold, radius) as a rendering/ resource
- Per-entity bloom contribution via HDR emissive values on materials
- Debug menu controls for bloom intensity and radius
- Temperature-aware bloom tint (shifts cool→warm with run progression, connects to 5f)

### 2. Additive Blending Pipeline

Current: Default blending on all materials.

Target:
- Custom `Material2d` base that uses additive blending for light-on-dark elements
- All gameplay entities (bolt, breaker, cells, effects) use additive blending
- Overlapping glows naturally combine and brighten (style pillar: "Light Is the Material")

### 3. Screen Distortion Shader

A post-processing render pass that applies screen-space distortion effects:
- **Radial distortion**: For shockwaves, gravity wells, explosions (used in 5m)
- **Directional distortion**: For screen shake displacement (used in 5k)
- Driven by a distortion buffer/texture that rendering systems write to
- Multiple simultaneous distortion sources combine additively

### 4. Chromatic Aberration Shader

Post-processing pass for RGB offset effects:
- Triggered by events (big hits, evolution triggers) via render messages
- Configurable intensity and duration
- Debug menu toggle and intensity slider

### 5. Screen Flash System

Full-screen additive color overlay:
- Brief HDR spikes (1-3 frames) with configurable color and intensity
- Gold/white for skill, white for triumphs, red-orange for danger
- Intensity tiers matching screen shake tiers (micro/small/medium/heavy)

### 6. Desaturation Shader

Post-processing pass for reducing color saturation:
- Used by failure states (bolt lost, run over) — see 5l
- Configurable blend factor (0.0 = full color, 1.0 = monochrome)
- Can animate from normal to desaturated over time

### 7. CRT/Scanline Overlay

- Post-processing pass with scan line pattern, slight curvature
- **OFF by default** — configurable in debug menu and RenderingDefaults RON
- Intensity configurable via slider
- Default state and intensity stored in `RenderingDefaults` RON file
- When a settings menu is added later, it writes a user preferences file that overrides `RenderingConfig` after the loading pipeline

## What NOT to Do

- Do NOT implement screen shake logic (that's 5k — screen effects)
- Do NOT implement specific VFX triggers (that's 5m — combat effects)
- Do NOT implement entity-level materials (that's 5g-5j — entity visuals)
- Build the pipeline and prove it works with simple test triggers via debug menu

## Dependencies

- **Requires**: 5c (rendering/ domain exists)
- **Independent of**: 5e (particle system) — can be done in either order
- DR-7 resolved: CRT off by default, configurable

## What This Step Builds

- Configurable bloom system (per-entity HDR emissive, debug tuning)
- Additive blending Material2d base for all light-on-dark elements
- Screen distortion post-processing pass (radial + directional)
- Chromatic aberration post-processing pass
- Screen flash system (full-screen additive HDR overlay, color/intensity/duration)
- Desaturation post-processing pass (animated 0.0–1.0 saturation blend)
- CRT/scanline overlay pass (off by default, configurable)
- RenderingDefaults RON file + RenderingConfig resource

## Verification

- Debug menu can trigger each post-processing effect independently
- Bloom is configurable and per-entity emissive values control brightness
- Additive blending produces correct light-on-dark compositing
- Screen distortion visually warps the rendered scene
- All existing tests pass, game plays normally
