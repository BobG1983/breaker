# 5h: Screen Effects, Visual Modifiers & Temperature Palette

## Summary

Wire screen-level effects (shake, slow-mo, vignette, desaturation) to gameplay via postprocess trigger messages. Build the modifier computation system that reads `ModifierStack` components and applies them to `EntityGlowMaterial` uniforms each frame. Build the temperature palette system that reads `RunTemperature` and applies palette shifts to grid, bloom, and wall colors on node transitions. This is the "dynamic visuals" phase — runtime visual changes to already-rendered entities.

## Context

This phase combines three concerns that were split across different old phases:

| Old phase | Concern | Where it lands now |
|-----------|---------|-------------------|
| 5k (screen effects) | Screen shake, slow-mo, vignette, desaturation trigger/lifecycle | Here — trigger/lifecycle systems in breaker-game that send postprocess messages |
| 5n (visual modifiers) | ModifierStack computation, diminishing returns application | Here — modifier computation system in visuals/ domain |
| 5f (temperature) | RunTemperature update, palette application | Here — temperature systems in visuals/ domain |

The types for all three (ModifierStack, VisualModifier, RunTemperature, TemperaturePalette) were defined in 5e (visuals domain). The postprocess shaders and trigger messages were built in 5d (rantzsoft_postprocess). This phase connects them: gameplay events trigger screen effects via postprocess messages, modifier stacks drive entity material uniforms, and temperature drives ambient palette shifts.

Key architecture change from the LEGACY plan: there is NO recipe system. Screen effects are triggered by sending `TriggerScreenFlash`, `TriggerRadialDistortion`, `TriggerChromaticAberration`, `TriggerDesaturation`, `TriggerVignette` messages directly from game systems. There is no `ExecuteRecipe` indirection. Modifiers are applied by sending `SetModifier` / `AddModifier` / `RemoveModifier` messages to the visuals/ domain.

## What to Build

### 1. Screen Shake System

Four-tier directional shake via camera `Transform` offset (not a shader effect):

| Tier | Displacement | Duration |
|------|-------------|----------|
| Micro | 1-2px | 1-2 frames |
| Small | 3-5px | 3-4 frames |
| Medium | 6-10px | 4-6 frames |
| Heavy | 12-20px | 6-10 frames |

Implementation details:
- `ScreenShakeTier` enum with `Micro`, `Small`, `Medium`, `Heavy` variants, each with displacement range and duration
- `TriggerScreenShake { tier: ScreenShakeTier, direction: Option<Vec2> }` message — if direction is `None`, use random direction
- `ActiveShake` component on the camera entity tracking current displacement, remaining duration, decay rate
- Exponential decay: displacement halves each frame after initial impulse
- Multiple shakes stack additively up to a maximum cap (e.g., 30px total displacement)
- Configurable multiplier via a `ScreenShakeConfig` resource (0.0 = disabled, 1.0 = default, 2.0 = maximum) — scales both displacement and duration
- System runs in `Update`, reads `Time<Real>` (not Virtual — shake should not slow down during slow-mo)
- System lives in `breaker-game/src/visuals/systems/screen_shake.rs`

### 2. Slow-Motion / Time Dilation

Smooth ramp into and out of slow-motion using `Time<Virtual>::set_relative_speed()`:

- `TriggerSlowMotion { factor: f32, duration: f32, ramp_in: f32, ramp_out: f32 }` message — `factor` is the target speed (0.3 = 30% speed), `duration` is time at target speed, `ramp_in`/`ramp_out` are transition durations
- `DilationState` resource tracking current phase (Idle, RampingIn, Holding, RampingOut), elapsed time within phase, and target factor
- Ramp timer uses `Time<Real>` — avoids recursive slowdown (ramp would slow itself if it used Virtual time)
- Affects `FixedUpdate` step rate (fewer steps per second, not slower steps)
- Multiple slow-mo triggers: latest trigger wins if its factor is lower (slower); higher-speed triggers are ignored while a slower one is active
- System lives in `breaker-game/src/visuals/systems/time_dilation.rs`

### 3. Danger Vignette System

Continuous vignette driven by timer state and lives count:

- Not a punctual burst — this is an ambient danger indicator that intensifies as the player's situation worsens
- Timer <25% remaining: vignette at 10-40% opacity, pulsing with increasing frequency as time drops
- Last life remaining: persistent 15% vignette opacity (additive with timer vignette)
- Combined vignette never exceeds 50% opacity
- System reads `NodeTimer` and `Lives` resources, sends `TriggerVignette` messages to rantzsoft_postprocess
- System lives in `breaker-game/src/run/systems/danger_vignette.rs` (run domain owns the gameplay state that drives it)

### 4. Screen Effect Trigger Lifecycle

The postprocess crate (5d) provides the shaders and materials. This phase adds the trigger/lifecycle management that decays effects over time:

- `TriggerScreenFlash` — sets flash intensity + color, auto-decays to zero over `duration_frames`. System tracks active flash and decrements each frame.
- `TriggerRadialDistortion` — adds a distortion source to the shader's fixed-size source array (16 max). Each source has position, radius, intensity, and remaining duration. System removes expired sources each frame.
- `TriggerChromaticAberration` — sets intensity, auto-decays over duration. One active instance at a time (latest wins if stronger).
- `TriggerDesaturation` — smoothly transitions to target saturation factor over duration. System interpolates current value toward target each frame.
- All messages are crate-owned (rantzsoft_postprocess). The lifecycle systems in breaker-game consume the trigger messages, manage the decay state, and write updated uniform values to the postprocess materials.
- Lifecycle systems live in `breaker-game/src/visuals/systems/postprocess_lifecycle.rs` (or split by effect if the file gets large)

### 5. Modifier Computation System

Reads `ModifierStack` components and applies computed values to `EntityGlowMaterial` uniforms:

- Runs each frame in `Update` on all entities with `(ModifierStack, Handle<EntityGlowMaterial>)`
- For each `ModifierKind` present in the stack, computes the effective value using diminishing returns curve (defined in 5e on ModifierStack)
- Maps computed values to EntityGlowMaterial uniform fields:
  - `GlowIntensity` -> `uniforms.core_brightness` multiplier
  - `CoreBrightness` -> `uniforms.core_brightness` additive
  - `HaloRadius` -> `uniforms.halo_radius` multiplier
  - `ShapeScale` -> `uniforms.half_extents` multiplier
  - `SpikeCount` -> `uniforms.spike_count`
  - `ColorShift` -> `uniforms.color` override
  - `ColorCycle` -> `uniforms.color` cycling through 4 colors at speed
  - `AlphaOscillation` -> `uniforms.alpha_override` oscillating between min/max at frequency
  - `SquashStretch` -> `uniforms.squash_x` / `uniforms.squash_y`
  - `RotationSpeed` -> `uniforms.rotation_angle` incrementing each frame
  - `TrailLength` -> updates trail component params (not a material uniform)
  - `AfterimageTrail` -> spawns/despawns afterimage trail entities
- `SetModifier` / `AddModifier` / `RemoveModifier` message handlers (visuals-owned messages defined in 5e) that modify the entity's `ModifierStack`
- Modifier duration tracking: system decrements duration timers each frame and removes expired entries
- System lives in `breaker-game/src/visuals/systems/modifier_computation.rs`

### 6. Temperature Palette System

Reads `RunTemperature` and applies palette shifts to ambient visual elements:

- `RunTemperature` resource (defined in 5e) tracks 0.0 (cool) to 1.0 (hot), updated on node transitions
- `TemperaturePalette` (defined in 5e) provides cool/hot endpoints for grid, bloom, and wall colors
- Temperature update system: on node clear, increments `RunTemperature` by `1.0 / total_nodes`. On run start, resets to 0.0.
- Palette application system: each frame, reads current temperature, lerps between cool and hot endpoints, updates:
  - Background grid material color uniform
  - Bevy `Bloom` settings (tint shifts from cool bloom color to hot bloom color)
  - Wall `EntityGlowMaterial` color uniform
- Temperature changes are smooth (lerp over ~0.5s after node transition, not instant snap)
- Systems live in `breaker-game/src/visuals/systems/temperature_palette.rs`

### 7. Debug Menu Integration

Add debug controls for all dynamic visual systems:

- Screen shake: intensity multiplier slider, test buttons for each shake tier
- Slow-mo: test button with configurable factor/duration
- Vignette: test toggle, opacity override slider
- Per-postprocess-effect: enable/disable toggles, intensity override sliders
- Temperature: manual slider to preview palette at any temperature value
- Modifier: test buttons to add/remove modifiers on the current bolt/breaker

## What NOT to Do

- Do NOT wire specific gameplay events to effects yet — bump grades, bolt lost, life lost, etc. are wired in 5k (bump/failure VFX). This phase builds the trigger/lifecycle systems and proves they work via messages and debug menu.
- Do NOT implement combat effect VFX (shockwave ring, chain lightning, etc.) — that is 5l.
- Do NOT implement particle effects — particle spawning for specific events comes in 5k and 5l. This phase only handles screen-level postprocess effects and entity material modifiers.
- Do NOT create a recipe system or `ExecuteRecipe` message — each VFX is triggered by direct function calls or specific typed messages.
- Do NOT modify entity builders (bolt, breaker, cell, wall) — those were set up in 5f-5i.

## Dependencies

- **Requires**: 5d (rantzsoft_postprocess — trigger messages and shader materials), 5e (visuals domain — ModifierStack, VisualModifier, RunTemperature, TemperaturePalette, EntityGlowMaterial types), 5f-5i (entities to modify — bolt, breaker, cell, wall builders must have attached EntityGlowMaterial handles)
- **Independent of**: 5c (rantzsoft_particles2d — no particle effects in this phase)
- **Required by**: 5k (bump/failure VFX — uses screen shake, slow-mo, desaturation, flash), 5l (combat VFX — uses distortion, modifiers), 5m (highlights — uses screen flash)

## Verification

- Screen shake displaces camera correctly per tier, with exponential decay
- Screen shake respects the multiplier config (0.0 disables, 2.0 doubles)
- Multiple shakes stack to the cap, do not exceed it
- Slow-mo ramps smoothly in and out (no instant snap)
- Slow-mo ramp uses `Time<Real>` — verify by triggering slow-mo and confirming the ramp itself does not slow down
- Slow-mo affects `FixedUpdate` rate (fewer physics steps during slow-mo)
- Danger vignette activates at timer <25% and last-life thresholds
- Combined vignette never exceeds 50% opacity
- Screen flash triggers and auto-decays over specified duration
- Radial distortion sources add and auto-remove on expiry
- Chromatic aberration triggers and auto-decays
- Desaturation smoothly transitions to target factor
- ModifierStack computation produces correct material uniform values
- Diminishing returns curve matches expected values (unit test the math)
- SetModifier/AddModifier/RemoveModifier messages correctly modify stacks
- Modifier duration expiry removes entries automatically
- ColorCycle cycles through 4 colors at correct speed
- AlphaOscillation oscillates between min/max at correct frequency
- Temperature updates on node clear (increments by correct amount)
- Temperature resets to 0.0 on run start
- Palette lerp produces correct intermediate colors at any temperature value
- Palette changes are smooth (not instant) after node transition
- Debug menu controls work for all systems
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL
