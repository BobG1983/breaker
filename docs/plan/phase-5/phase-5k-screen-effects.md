# 5k: Screen Effects & Feedback

**Goal**: Implement the screen-level effect systems (shake, flash, desaturation, slow-mo, vignette) that combat VFX and feedback steps will trigger.

## What to Build

### 1. Screen Shake System

Four-tier directional screen shake:

| Tier | Displacement | Duration | Events (wired in later steps) |
|------|-------------|----------|-------------------------------|
| Micro | 1-2px | 1-2 frames | Perfect bump, single cell break |
| Small | 3-5qx | 3-4 frames | Cell chain 3+, shockwave, fast wall impact |
| Medium | 6-10px | 4-6 frames | Evolution trigger, large chain 5+, explosion |
| Heavy | 12-20px | 6-10 frames | Supernova, screen-filling events |

Characteristics:
- Direction roughly matches impact direction (bolt→wall = horizontal, explosion = radial)
- Amplitude decays exponentially — sharp onset, quick falloff
- Multiple shakes stack with a cap to prevent nausea
- Moves the camera/viewport, not individual elements (preserves relative positions)
- Configurable intensity multiplier in debug menu (0.0-2.0)

Driven by module-owned message: `TriggerScreenShake { tier, direction }`.

### 2. Screen Flash System

Full-screen additive HDR overlay (from 5d infrastructure, now with trigger logic):
- Brief spikes (1-3 frames)
- Color/intensity per event type:
  - Gold/white: skill moments (perfect bump)
  - White: triumphs (run won)
  - Red-orange: danger (bolt lost, life lost)
  - Temperature-tinted: escalation events

Driven by module-owned message: `TriggerScreenFlash { color, intensity, duration_frames }`.

### 3. Slow-Motion / Time Dilation

Game clock slowdown for failure states and dramatic moments:
- Configurable slowdown factor (e.g., 30% speed for 0.3s)
- Affects game clock INCLUDING node timer (the pause is real, not cosmetic)
- Smooth ramp in/out (not instant snap to slow speed)
- Can be triggered with duration and factor

Driven by module-owned message: `TriggerSlowMotion { factor, duration }`.

### 4. Danger Vignette

Red-orange gradient inward from screen edges:
- Timer <25%: 10-40% opacity, pulsing at danger-scaled rhythm
- Last life: persistent 15% opacity
- Never >50% combined opacity
- Pulses accelerate (slow → accelerating → heartbeat-synced)

Driven continuously by timer state and lives count.

### 5. Screen Distortion Triggers

The distortion shader from 5d is ready — this step adds the trigger interface:
- Module-owned message: `TriggerRadialDistortion { origin, radius, intensity, duration }` — for shockwave, explosion
- Distortion sources managed by rendering/ with automatic cleanup on duration expiry

### 6. Chromatic Aberration Triggers

The chromatic aberration shader from 5d — this step adds triggers:
- Module-owned message: `TriggerChromaticAberration { intensity, duration }` — for big hits, evolution triggers
- Configurable in debug menu

### 7. Debug Menu Integration

Screen effects debug controls:
- Screen shake intensity multiplier
- Chromatic aberration intensity slider
- Bloom intensity override
- CRT toggle (off by default)
- Test buttons for each effect tier

## What NOT to Do

- Do NOT wire specific gameplay events to screen effects yet (that's 5l, 5m)
- Build the systems and prove they work via render messages and debug triggers

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing shaders for distortion/flash/chromatic/desaturation)
- **Independent of**: 5e-5j (particles and entity visuals)

## What This Step Builds

- Screen shake system (4 tiers: Micro/Small/Medium/Heavy, directional, exponential decay, stacking with cap)
- Screen flash trigger system (per-event color/intensity, gold/white/red-orange)
- Slow-motion / time dilation system (configurable factor + duration, affects game clock)
- Danger vignette (red-orange gradient, pulsing at danger-scaled rhythm, threshold-based)
- Radial distortion trigger interface (for shockwave/explosion, managed lifecycle)
- Chromatic aberration trigger interface (for big hits/evolution triggers)
- Debug menu: intensity sliders, test buttons for each tier, CRT toggle

## Verification

- Each screen effect triggers correctly via debug menu
- Screen shake has correct per-tier displacement and decay
- Slow-mo actually affects game clock
- Danger vignette pulses at correct thresholds
- Multiple effects can be active simultaneously without conflict
- All existing tests pass
