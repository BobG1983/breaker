# 5k: Screen Effects & Feedback

**Goal**: Implement screen-level effect trigger systems (shake, slow-mo, vignette) that combat VFX and feedback steps will use. Post-processing shaders were built in 5d — this step adds the trigger/lifecycle logic.

Architecture: `docs/architecture/rendering/screen_effects.md`, `docs/architecture/rendering/slow_motion.md`

## What to Build

### 1. Screen Shake System

Four-tier directional shake via camera `Transform` offset (not a shader):

| Tier | Displacement | Duration |
|------|-------------|----------|
| Micro | 1-2px | 1-2 frames |
| Small | 3-5px | 3-4 frames |
| Medium | 6-10px | 4-6 frames |
| Heavy | 12-20px | 6-10 frames |

- Direction roughly matches impact direction
- Exponential decay
- Multiple shakes stack with a cap
- Configurable multiplier via `VfxConfig.shake_multiplier`
- Triggered by `TriggerScreenShake { camera, tier, direction }` message

### 2. Slow-Motion / Time Dilation

`Time<Virtual>::set_relative_speed()` with smooth ramp:
- `TriggerSlowMotion { factor, duration, ramp_in, ramp_out }` message
- `DilationRamp` resource using `Time<Real>` for the ramp timer (avoids recursive slowdown)
- Affects FixedUpdate step rate (fewer steps, not slower steps)
- See `docs/architecture/rendering/slow_motion.md` for gotchas

### 3. Danger Vignette System

Continuous vignette driven by timer state and lives count (in `run/` domain):
- Timer <25%: 10-40% opacity, pulsing at danger-scaled rhythm
- Last life: persistent 15% opacity
- Never >50% combined opacity
- `run/danger_vignette.rs` sends `TriggerVignettePulse` messages

### 4. Screen Effect Trigger Lifecycle

The FullscreenMaterial effects from 5d need trigger/lifecycle management:
- `TriggerScreenFlash` → sets flash intensity + color, auto-decays over duration_frames
- `TriggerRadialDistortion` → adds source to distortion buffer (16-source array), auto-removes on duration expiry
- `TriggerChromaticAberration` → sets intensity, auto-decays over duration
- `TriggerDesaturation` → smooth transition to target factor over duration
- All triggered by typed per-primitive messages (crate-owned)

### 5. Debug Menu Integration

- Screen shake intensity multiplier slider
- Test buttons for each shake tier
- Slow-mo test button
- Vignette test toggle
- Per-effect override sliders

## What NOT to Do

- Do NOT wire specific gameplay events to effects yet (that's 5l, 5m)
- Build the systems and prove they work via messages and debug triggers

## Dependencies

- **Requires**: 5c (crate), 5d (post-processing shaders)
- **Independent of**: 5e-5j

## Verification

- Each screen effect triggers correctly via debug menu
- Screen shake has correct per-tier displacement and decay
- Slow-mo affects game clock (FixedUpdate rate changes)
- DilationRamp uses Time<Real> (verified by slowing down — ramp doesn't slow recursively)
- Danger vignette pulses at correct thresholds
- Multiple effects can be active simultaneously
- All existing tests pass
