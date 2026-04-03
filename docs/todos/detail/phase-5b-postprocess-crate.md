# 5b: rantzsoft_postprocess Crate

## Summary

Create a standalone, game-agnostic post-processing crate at `rantzsoft_postprocess/` that provides FullscreenMaterial infrastructure and 7 common shader effects (screen flash, radial distortion, chromatic aberration, vignette, desaturation, CRT overlay) plus bloom tuning helpers. The crate exposes trigger messages for each effect, a `PostProcessConfig` resource for per-effect enable/disable and intensity defaults, and a `RantzPostProcessPlugin` with `default()` and `headless()` constructors.

## Context

The original Phase 5 architecture placed all visual infrastructure in a monolithic `rantzsoft_vfx` crate. The revised architecture (see `docs/todos/detail/phase-5-rethink/architecture.md`) splits this into three independent pieces:

- **rantzsoft_particles2d** (5c) — CPU particle engine
- **rantzsoft_postprocess** (5d, this phase) — FullscreenMaterial post-processing
- **breaker-game visuals/ domain** (5e) — game-specific visual types, entity shaders, modifiers

This split exists because particles and post-processing have zero overlap in API surface, rendering approach, and consumers. A monolithic crate would force games that only need particles to pull in the full render graph machinery, and vice versa. Each crate is independently useful for any 2D Bevy game.

This crate does NOT depend on `rantzsoft_particles2d`, `rantzsoft_spatial2d`, `rantzsoft_physics2d`, or any other `rantzsoft_*` crate. It depends only on Bevy.

## What to Build

### 1. Crate Scaffold

New workspace member at `rantzsoft_postprocess/` following `rantzsoft_*` conventions:

- `Cargo.toml` with `name = "rantzsoft_postprocess"`, edition 2024, `publish = false`, workspace lints
- Bevy dependency: `bevy = { version = "0.18.1", default-features = false, features = ["2d"] }` — plus any additional render features required by `FullscreenMaterial` (likely `bevy_render`, `bevy_core_pipeline`)
- Add to root `Cargo.toml` `[workspace.members]`
- Add cargo aliases to `.cargo/config.toml`: `postprocesscheck`, `postprocessclippy`, `postprocesstest`
- Add to `all-dtest` and `all-dclippy` compound aliases
- Zero game vocabulary — no references to bolt, breaker, cell, node, bump, flux, or any game terminology

### 2. Plugin with `default()` and `headless()` Constructors

`RantzPostProcessPlugin` — the root Bevy plugin:

- `default()` — registers all shaders, materials, render graph nodes, systems, and messages. For use in the full game.
- `headless()` — registers messages and the `PostProcessConfig` resource but skips shader compilation, material registration, and render graph node insertion. For use in automated testing and the scenario runner where no GPU is available.

Both constructors accept a `PostProcessConfig` or use `PostProcessConfig::default()`.

### 3. PostProcessConfig Resource

`PostProcessConfig` resource defined in the crate, inserted and mutated by the game:

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `bloom_enabled` | `bool` | `true` | Master bloom toggle |
| `bloom_intensity` | `f32` | `0.3` | Camera `Bloom` intensity |
| `bloom_threshold` | `f32` | `1.0` | HDR threshold for bloom contribution |
| `bloom_radius` | `f32` | `0.5` | Bloom blur radius |
| `flash_enabled` | `bool` | `true` | Screen flash effect toggle |
| `distortion_enabled` | `bool` | `true` | Radial distortion toggle |
| `chromatic_enabled` | `bool` | `true` | Chromatic aberration toggle |
| `vignette_enabled` | `bool` | `true` | Vignette toggle |
| `desaturation_enabled` | `bool` | `true` | Desaturation toggle |
| `crt_enabled` | `bool` | `false` | CRT overlay toggle (off by default) |
| `crt_intensity` | `f32` | `0.5` | CRT scanline/curvature strength |
| `chromatic_multiplier` | `f32` | `1.0` | Global multiplier for chromatic aberration intensity |

The game reads this from a RON defaults file (via `rantzsoft_defaults`) and inserts it as a resource. The debug menu mutates it at runtime. Post-processing systems check the relevant `*_enabled` flag each frame and skip processing when disabled.

### 4. FullscreenMaterial Helpers

Core infrastructure that all post-processing shaders build on:

- **ViewTarget ping-pong**: Each effect reads from the current source texture via `view_target.post_process_write()` and writes to the destination. This is the standard Bevy pattern for chaining multiple post-process passes without intermediate textures.
- **Render graph node ordering**: Each effect defines a concrete `RenderLabel` type (e.g., `FlashLabel`, `DistortionLabel`, `ChromaticLabel`). The `node_edges()` method on each `FullscreenMaterial` impl controls where in the render pipeline the effect runs relative to other effects and built-in passes.
- **Material registration**: Each `FullscreenMaterial` type is registered via `app.add_plugins(FullscreenMaterialPlugin::<T>::default())` in the plugin setup.
- **Intensity-based disable**: Effects are disabled by setting `intensity = 0.0` on their material uniform. Removing the component does NOT remove the render graph node (the node is added on first `Added<T>` and persists). The shader early-exits when intensity is zero.
- **std140 alignment**: Single `f32` uniforms must pad to 16 bytes (`_padding: Vec3`). All uniform structs must respect std140 layout rules.

### 5. Bloom Tuning Helpers

Bloom is Bevy's built-in `Bloom` component on the camera — this crate does not implement its own bloom. Instead, it provides:

- A system that syncs `PostProcessConfig` bloom fields (`bloom_intensity`, `bloom_threshold`, `bloom_radius`) to the camera's `Bloom` component each frame (only when values change)
- Helper methods on `PostProcessConfig` for common bloom presets (e.g., `subtle()`, `intense()`, `cinematic()`)
- Per-entity bloom contribution remains via HDR emissive values >1.0 on materials — this is automatic and not managed by the crate

### 6. Seven Post-Processing Shaders

See **Shader Details** section below for per-shader specifics.

Each shader is a `FullscreenMaterial` implementation with:
- A WGSL shader file in `rantzsoft_postprocess/assets/shaders/`
- A corresponding Rust uniform struct
- A concrete `RenderLabel` for render graph ordering
- A system that reads trigger messages and updates the material uniforms (intensity, timer, parameters)
- A decay/animation system that drives time-based parameters (e.g., flash fade-out, distortion decay)

### 7. Trigger Messages

Each effect has a Bevy 0.18 message type that gameplay systems send to trigger effects:

| Message | Fields | Behavior |
|---------|--------|----------|
| `TriggerScreenFlash` | `color: Color`, `intensity: f32`, `duration_secs: f32` | Additive flash overlay, decays to zero |
| `TriggerRadialDistortion` | `origin: Vec2`, `intensity: f32`, `radius: f32`, `duration_secs: f32` | Adds a distortion source to the buffer |
| `TriggerChromaticAberration` | `intensity: f32`, `duration_secs: f32` | RGB channel offset, decays to zero |
| `TriggerVignette` | `intensity: f32`, `radius: f32`, `duration_secs: f32` | Edge darkening, decays to zero |
| `TriggerDesaturation` | `intensity: f32`, `duration_secs: f32` | Per-pixel saturation reduction, decays to zero |
| `SetCrtEnabled` | `enabled: bool` | Toggles CRT overlay on/off (persistent, not time-decaying) |

CRT is persistent (on/off toggle), not triggered. All other effects are transient (fire and decay). Multiple concurrent triggers of the same effect type stack additively (e.g., two simultaneous flashes sum their intensities, clamped to a max).

### 8. Effect Decay Systems

Each transient effect has a decay system running in `Update`:

- Reads elapsed time since trigger
- Interpolates intensity from initial value toward 0.0 over the specified duration
- Uses smooth falloff (not linear — exponential decay or ease-out curve for natural feel)
- When intensity reaches a near-zero threshold (< 0.001), snaps to exactly 0.0 so the shader can early-exit

For radial distortion specifically, the system manages a fixed-size source array (16 sources). New triggers occupy the first available slot. Expired sources are zeroed out. If all 16 slots are full, the oldest source is evicted.

## FullscreenMaterial Spike Results

These findings were verified in an earlier spike and are carried forward as constraints for implementation:

| Finding | Status | Implication |
|---------|--------|-------------|
| Multiple `FullscreenMaterial` types on one camera | **Works** | Each effect is its own `FullscreenMaterial` impl — no need to merge into a single uber-shader |
| Custom blend state | **Blocked** | `FullscreenMaterial` hardcodes `blend: None`. Additive blending (e.g., screen flash) must be done in the fragment shader (`scene_color.rgb + flash.rgb`), NOT via GPU blend state |
| `Node2d::Bloom` in `node_edges()` | **Panics** with `features = ["2d"]` | Do NOT use `Node2d::Bloom` as a positioning anchor. Use `Node2d::Tonemapping` as the pre-tonemapping anchor instead |
| HDR texture format | **Automatic** | Pre-tonemapping effects automatically get `Rgba16Float` via `view_target.is_hdr()`. No manual format selection needed |
| Custom ViewNode fallback | **Not needed** | No planned effect requires a custom `ViewNode` implementation |
| Removing component vs setting intensity to 0 | **Component removal does NOT remove render node** | The render graph node is added on first `Added<T>` and persists for the camera's lifetime. Disable effects by setting intensity to 0.0, which lets the shader early-exit |

## Shader Details

### 1. Screen Flash (`flash.wgsl`)

- **Pipeline position**: Before Bloom (before Tonemapping). `node_edges() = [StartMainPassPostProcessing, FlashLabel, Tonemapping]`
- **Uniforms**: `flash_color: vec4<f32>`, `intensity: f32`, `_padding: vec3<f32>`
- **Fragment logic**: `scene_color.rgb + flash_color.rgb * intensity`. Additive blend in shader (NOT GPU blend state — see spike results). Early-exit when `intensity < 0.001`.
- **Trigger message**: `TriggerScreenFlash { color, intensity, duration_secs }`
- **Why before bloom**: Flash contributes to bloom naturally — a bright flash blooms outward, which feels physically correct. Placing it after bloom would produce a flat overlay.

### 2. Radial Distortion (`distortion.wgsl`)

- **Pipeline position**: After Tonemapping. `node_edges() = [Tonemapping, DistortionLabel, ...]`
- **Uniforms**: Fixed array of 16 distortion sources. Each source: `origin: vec2<f32>`, `intensity: f32`, `radius: f32`. Plus `source_count: u32` (number of active sources), `screen_size: vec2<f32>`.
- **Fragment logic**: For each active source, compute distance from fragment to source origin. Within radius, offset the UV sample position radially (push outward for shockwave, pull inward for gravity well — sign of intensity controls direction). Accumulate offsets from all sources, then sample scene texture at the distorted UV.
- **Trigger message**: `TriggerRadialDistortion { origin, intensity, radius, duration_secs }`
- **Buffer management**: 16 source slots. Positive intensity = push outward (shockwave). Negative intensity = pull inward (gravity well). Sources decay independently.

### 3. Chromatic Aberration (`chromatic_aberration.wgsl`)

- **Pipeline position**: After Tonemapping (after distortion). `node_edges() = [DistortionLabel, ChromaticLabel, ...]`
- **Uniforms**: `intensity: f32`, `_padding: vec3<f32>`
- **Fragment logic**: Sample R, G, B channels at slightly offset UVs. Offset magnitude = `intensity * distance_from_center`. R shifts outward, B shifts inward (classic chromatic fringing). Early-exit when `intensity < 0.001`.
- **Trigger message**: `TriggerChromaticAberration { intensity, duration_secs }`

### 4. Vignette (`vignette.wgsl`)

- **Pipeline position**: After Tonemapping (after chromatic aberration). `node_edges() = [ChromaticLabel, VignetteLabel, ...]`
- **Uniforms**: `intensity: f32`, `radius: f32`, `softness: f32`, `_padding: f32`
- **Fragment logic**: Compute distance from UV center. Darken pixels outside `radius` by multiplying RGB by a smooth falloff toward the edges. `intensity` scales the darkening amount. Early-exit when `intensity < 0.001`.
- **Trigger message**: `TriggerVignette { intensity, radius, duration_secs }`

### 5. Desaturation (`desaturation.wgsl`)

- **Pipeline position**: After Tonemapping (after vignette). `node_edges() = [VignetteLabel, DesaturationLabel, ...]`
- **Uniforms**: `intensity: f32`, `_padding: vec3<f32>`
- **Fragment logic**: `luminance = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722))`. Lerp between original color and luminance gray by `intensity`. At intensity 1.0, fully grayscale. Early-exit when `intensity < 0.001`.
- **Trigger message**: `TriggerDesaturation { intensity, duration_secs }`

### 6. CRT Overlay (`crt.wgsl`)

- **Pipeline position**: Last in the chain. `node_edges() = [DesaturationLabel, CrtLabel]`
- **Uniforms**: `enabled: u32` (0 or 1), `intensity: f32`, `time: f32`, `curvature: f32`, `scanline_density: f32`, `noise_amount: f32`
- **Fragment logic**: Apply barrel distortion to UVs (CRT curvature). Multiply by scanline pattern (horizontal dark bars). Add film grain noise scaled by `noise_amount`. Darken corners (CRT natural vignette). Skip all processing when `enabled == 0`.
- **Trigger message**: `SetCrtEnabled { enabled }` (persistent toggle, not time-decaying)
- **Default state**: Off. Player opts in via settings.

### 7. Bloom Tuning (no custom shader)

- **Pipeline position**: N/A — uses Bevy's built-in `Bloom` component
- **No WGSL shader** — this is a sync system, not a custom render pass
- **System**: Reads `PostProcessConfig` fields (`bloom_intensity`, `bloom_threshold`, `bloom_radius`), writes to the camera's `Bloom` component when values change
- **Per-entity bloom**: Entities with HDR emissive material values >1.0 bloom naturally through Bevy's built-in system. This crate does not manage per-entity bloom — it manages the camera-level bloom settings.

### Full Pipeline Order

```
Scene Render
  |
  v
StartMainPassPostProcessing
  |
  v
Screen Flash (FlashLabel)          <-- additive flash, before bloom so it blooms
  |
  v
Tonemapping (Node2d::Tonemapping)  <-- Bevy built-in, includes Bloom pass
  |
  v
Radial Distortion (DistortionLabel)
  |
  v
Chromatic Aberration (ChromaticLabel)
  |
  v
Vignette (VignetteLabel)
  |
  v
Desaturation (DesaturationLabel)
  |
  v
CRT Overlay (CrtLabel)             <-- last, applies to everything
```

## What NOT to Do

- Do NOT implement screen shake — that is a camera `Transform` offset driven by gameplay systems in `breaker-game`, not a post-processing shader
- Do NOT implement game-specific visual types (Shape, Hue, Aura, Trail, GlowParams, VisualModifier) — those belong in the `visuals/` game domain (5e)
- Do NOT implement entity-level materials (entity_glow SDF, additive Material2d, glitch_text, holographic) — those are game-side shaders in `visuals/` (5e)
- Do NOT implement particle systems — that is `rantzsoft_particles2d` (5c)
- Do NOT implement temperature-aware bloom tinting — temperature is a game concept; this crate provides the bloom tuning knobs, the game applies temperature-driven values
- Do NOT implement transition systems (flash transition, glitch transition, etc.) — transitions are game-side systems that send trigger messages to this crate
- Do NOT implement specific VFX triggers or effect-to-postprocess wiring — that is Phase 5j/5k/5l game-side work
- Do NOT use game vocabulary anywhere in the crate (bolt, breaker, cell, node, bump, flux, etc.)
- Do NOT use `Node2d::Bloom` in `node_edges()` — it panics with 2d features (see spike results)
- Do NOT rely on GPU blend state for additive effects — `FullscreenMaterial` hardcodes `blend: None`
- Do NOT remove `FullscreenMaterial` components to disable effects — set intensity to 0.0 instead

## Crate Structure

```
rantzsoft_postprocess/
  Cargo.toml
  src/
    lib.rs                          // pub mod declarations, crate-level docs
    plugin.rs                       // RantzPostProcessPlugin (default + headless)
    config.rs                       // PostProcessConfig resource
    labels.rs                       // RenderLabel types: FlashLabel, DistortionLabel, etc.
    bloom.rs                        // Bloom sync system (PostProcessConfig -> Bloom component)
    messages.rs                     // TriggerScreenFlash, TriggerRadialDistortion, etc.
    decay.rs                        // Effect decay systems (intensity -> 0.0 over duration)
    effects/
      mod.rs                        // pub mod declarations
      flash.rs                      // FlashMaterial, uniform struct, FullscreenMaterial impl
      distortion.rs                 // DistortionMaterial, source buffer, FullscreenMaterial impl
      chromatic_aberration.rs       // ChromaticAberrationMaterial, FullscreenMaterial impl
      vignette.rs                   // VignetteMaterial, FullscreenMaterial impl
      desaturation.rs               // DesaturationMaterial, FullscreenMaterial impl
      crt.rs                        // CrtMaterial, FullscreenMaterial impl
  assets/
    shaders/
      flash.wgsl
      distortion.wgsl
      chromatic_aberration.wgsl
      vignette.wgsl
      desaturation.wgsl
      crt.wgsl
```

## Dependencies

### Crate Dependencies

```toml
[dependencies]
bevy = { version = "0.18.1", default-features = false, features = ["2d"] }
```

Additional Bevy features may be needed depending on what `FullscreenMaterial` requires at compile time (e.g., render pipeline features). Determine the minimal feature set during implementation.

### What This Crate Does NOT Depend On

- `rantzsoft_particles2d` — independent crate, zero overlap
- `rantzsoft_spatial2d` — post-processing operates in screen space, not world space
- `rantzsoft_physics2d` — no physics concepts in post-processing
- `rantzsoft_defaults` — the crate defines `PostProcessConfig` as a plain Rust struct; the game wires it to RON via `rantzsoft_defaults` on its own
- `breaker-game` or any `breaker-*` crate — `rantzsoft_*` crates never depend on game crates

### What Depends on This Crate

- `breaker-game` — adds `rantzsoft_postprocess` as a dependency, registers `RantzPostProcessPlugin` in `game.rs`, sends trigger messages from gameplay systems
- `breaker-scenario-runner` — uses `RantzPostProcessPlugin::headless()` for scenario runs without GPU

## Verification

- Crate compiles as a workspace member with `cargo postprocesscheck`
- All clippy lints pass with `cargo postprocessclippy`
- `RantzPostProcessPlugin::headless()` works without GPU (messages register, config resource exists, no render graph nodes)
- `RantzPostProcessPlugin::default()` registers all 6 FullscreenMaterial types and their render graph nodes
- Bloom sync system correctly updates camera `Bloom` from `PostProcessConfig`
- Each shader compiles (WGSL validation)
- Screen flash renders before bloom and gets bloomed naturally
- Radial distortion visually warps the rendered scene (16 concurrent sources)
- Chromatic aberration produces visible RGB fringing at edges
- Vignette darkens screen edges
- Desaturation correctly lerps to grayscale
- CRT overlay applies scanlines, curvature, and noise when enabled
- Effects chain correctly in the documented pipeline order
- Multiple simultaneous effects of different types compose without artifacts
- Setting intensity to 0.0 effectively disables each effect (shader early-exits)
- Trigger messages fire effects that decay over the specified duration
- PostProcessConfig toggles correctly enable/disable individual effects
- All existing workspace tests pass after adding the crate
- Zero game vocabulary in any source file

## NEEDS DETAIL — API Design

These questions must be answered before implementation:

- What does triggering a screen flash look like from a gameplay system? (exact API — Bevy message, resource mutation, or direct component insertion?)
- Should trigger messages be Bevy messages (`MessageWriter<TriggerScreenFlash>`) or resource-based (`commands.insert_resource(ScreenFlash { ... })`)?
- What types need to be `pub` vs `pub(crate)`?
- What goes in `app.add_plugins(RantzPostProcessPlugin)` — does it auto-register all messages/systems, or does the consumer pick which effects to enable?
- Can effects be composed at compile time? (e.g., a type-safe builder for distortion sources rather than a fixed array)
- What's the headless story — does `headless()` register message types but skip shader/material creation? Do trigger messages silently no-op?
- How does the config resource interact with per-trigger overrides? (e.g., config says CRT is off, but a trigger says CRT on — who wins?)
- Pipeline ordering — is this hardcoded in the crate or configurable by the consumer?
- Trait bounds on the plugin — does it need a generic for anything?

## Status
`[NEEDS DETAIL]` — API design questions above must be resolved
