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

**FullscreenMaterial spike — COMPLETE.** Verified in `.claude/specs/fullscreen-material-spike.md`. Key findings:
- Multiple FullscreenMaterial types on one camera: **works**
- Custom blend state: **blocked** (hardcoded `blend: None`) — additive blend must be done in the fragment shader
- `Node2d::Bloom` in node_edges: **panics** with `features = ["2d"]` — use `Node2d::Tonemapping` as the pre-tonemapping anchor
- HDR texture format: **automatic** — pre-tonemapping effects get `Rgba16Float` via `view_target.is_hdr()`
- No custom ViewNode fallback needed for any planned effect

Key details:
- Screen flash: additive blend in shader (`scene_color.rgb + flash.rgb`), NOT GPU blend state
- Flash placement: `node_edges() = [StartMainPassPostProcessing, FlashLabel, Tonemapping]`
- Each effect needs a concrete `RenderLabel` type for cross-effect `node_edges()` chaining
- Distortion buffer: 16-source fixed array uniform (see `docs/architecture/rendering/screen_effects.md`)
- Effects disabled by setting `intensity = 0.0` (removing component does NOT remove render node; node added on first `Added<T>`)
- std140 alignment: single `f32` must pad to 16 bytes (`_padding: Vec3`)
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
