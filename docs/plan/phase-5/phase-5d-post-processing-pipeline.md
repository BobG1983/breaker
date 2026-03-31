# 5d: Post-Processing Pipeline

**Goal**: Build the post-processing render infrastructure in `rantzsoft_vfx` that screen effects, combat VFX, and entity shaders will use.

Architecture: `docs/architecture/rendering/screen_effects.md`, `docs/architecture/rendering/shaders.md`

## What to Build

### 1. Bloom Tuning

Current: `Bloom::default()` on camera with no tuning.

Target:
- Configurable bloom settings (intensity, threshold, radius) via `VfxConfig`
- Per-entity bloom contribution via HDR emissive values on materials (values >1.0 bloom naturally)
- Temperature-aware bloom tint (shifts cool→warm with run progression, connects to 5f)
- Debug menu controls for bloom intensity and radius

### 2. Additive Blending Base Material

Bevy 0.18 `AlphaMode2d` has no `Add` variant. Build the additive blend pattern:
- `Material2d::specialize()` overriding `BlendState` with `BlendFactor::One` for dst_factor
- This pattern is used by entity_glow, particle, aura, and primitive materials
- See `docs/architecture/rendering/shaders.md` — Additive Blending section

### 3. FullscreenMaterial Post-Processing Effects

Each effect is a `FullscreenMaterial` component on the camera entity. `node_edges()` controls render graph ordering. `ViewTarget::post_process_write()` ping-pongs textures for chaining.

Build these FullscreenMaterial implementations:

| Effect | Shader | Pipeline Position |
|--------|--------|-------------------|
| Screen flash | `flash.wgsl` | Before Bloom (via `node_edges()`) |
| Radial distortion | `distortion.wgsl` | After Tonemapping |
| Chromatic aberration | `chromatic_aberration.wgsl` | After Tonemapping |
| Desaturation | `desaturation.wgsl` | After Tonemapping |
| Vignette | `vignette.wgsl` | After Tonemapping |
| CRT overlay | `crt.wgsl` | Last |

Key details:
- Screen flash must use `ViewTarget::TEXTURE_FORMAT_HDR` for the pipeline color target
- Distortion buffer: 16-source fixed array uniform (see `docs/architecture/rendering/screen_effects.md`)
- Effects disabled by setting `intensity = 0.0` (removing component does NOT remove render node)
- std140 alignment: single `f32` must pad to 16 bytes
- CRT off by default, configurable via `VfxConfig`

### 4. VfxConfig Integration

`VfxConfig` resource (defined in crate, inserted by game from `GraphicsDefaults` RON):
- `shake_multiplier`, `bloom_intensity`, `crt_enabled`, `crt_intensity`, `chromatic_multiplier`
- Post-processing systems read VfxConfig each frame
- Debug menu mutates VfxConfig at runtime

## What NOT to Do

- Do NOT implement screen shake logic (that's 5k — camera Transform offset, not a shader)
- Do NOT implement specific VFX triggers (that's 5m)
- Do NOT implement entity-level materials (that's 5g-5j)
- Build the pipeline and prove it works with simple test triggers via debug menu

## Dependencies

- **Requires**: 5c (rantzsoft_vfx crate exists)
- **Independent of**: 5e (particle system) — can be done in either order

## Verification

- Debug menu can trigger each post-processing effect independently
- Bloom is configurable and per-entity emissive values control brightness
- Additive blending produces correct light-on-dark compositing
- Screen flash renders before bloom (gets bloomed naturally)
- Distortion visually warps the rendered scene
- Effects chain correctly (flash → bloom → tonemap → distortion → chromatic → desaturation → vignette → CRT)
- All existing tests pass
