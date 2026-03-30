# 5m: Combat Effect VFX

**Goal**: Add visual representations for all triggered combat effects. Currently these effects have gameplay logic but no visuals.

## What to Build

Each effect follows the **Geo + Shader** layered design: a clean readable geometric layer plus a shader enhancement layer.

### 1. Shockwave VFX

- **Geo**: Expanding ring from impact point, ring thickness = damage falloff
- **Shader**: Radial screen distortion at ring edge (from 5d), bloom on ring
- **Duration**: ~0.3s expansion, then fades
- **Stacking**: Higher stacks = larger radius, thicker ring, brighter bloom, stronger distortion
- **Screen**: Small shake on fire (from 5k)

### 2. Chain Lightning VFX

- **Geo**: Branching jagged lines connecting source cell to targets
- **Shader**: Electric crackling along lines, brightness fluctuates, Spark particles at branch points, brief bloom flash at each target
- **Duration**: Near-instant, flickers 1-2 frames, then fades
- **Stacking**: More stacks = more targets = more branching lines
- **Completion**: Emits `ChainLightningVfxComplete` back to gameplay for sequencing

### 3. Piercing Beam VFX

- **Geo**: Straight line from bolt through pierced cells
- **Shader**: Bright core with bloom, narrowing toward end, brief distortion along beam path
- **Duration**: Appears on pierce, lingers ~0.1s as fading afterimage

### 4. Pulse VFX

- **Geo**: Expanding filled circle from breaker/bolt position
- **Shader**: Radial gradient bright center→soft edge, screen distortion at edge, cells flash inside
- **Duration**: ~0.2s (faster than shockwave)

### 5. Explode VFX

- **Geo**: Radial burst lines from destroyed cell center
- **Shader**: Central flash (HDR >2.0), rapid Spark particle spray, brief shake, distortion ring at edge
- **Duration**: ~0.15t — fastest effect, maximum immediacy

### 6. Gravity Well VFX

- **Geo**: Circle showing radius of influence, faint radial lines pulling inward
- **Shader**: Screen-space distortion lens — warps what's beneath toward center, intensifies toward center. No dark void — warping and refraction only.
- **Duration**: Persistent while active, slow rotation animation
- **Ambient**: Glow Mote particles drift toward center

### 7. Tether Beam VFX

- **Geo**: Visible line connecting tethered bolts, slight elasticity (stretches when far, slackens when close)
- **Shader**: Animated energy flowing along beam (brightness traveling end to end), color matches bolt halo
- **Duration**: Persistent while tether active
- **Break**: Flash + Spark particles when tether snaps

### 8. Attraction/Magnetism VFX

- **Geo**: Faint curved arcs (2-3 thin lines, <0.3 HDR) between bolt and target
- **Shader**: Lines flicker in/out with jitter. Brighten at close range. Bolt wake trail bends toward target.
- Intentionally minimal — ambient steering, not spectacle.

### 9. Ramping Damage VFX

- **Geo**: Faint Energy Ring orbiting bolt, spins faster with consecutive hits
- **Shader**: Bolt halo shifts warmer (base→amber→white-hot) as ramp counter climbs. At 6+ stacks: afterimage frames in wake.
- **Reset**: On whiff reset — heat drains ~0.3s, orbital ring shatters outward in dim Sparks. Punishing visual moment.

### 10. Random Effect (Flux) VFX

- **Geo**: Brief multi-colored Spark starburst from entity on selection
- **Shader**: Rapid prismatic flash (~0.1s, 3-4 spectral colors) then resolves to selected effect's visual
- Fast and subtle — modifies other effects, isn't a spectacle itself

### 11. Quick Stop VFX

- **Geo**: Brief compression (squash 2-3 frames) + small Spark spray forward from breaker leading edge
- **Shader**: Trail from dash abruptly terminates. At high stacks: micro-distortion ripple from breaker position.

### 12. Bump Force Boost VFX

- **Geo**: Concentrated impact flash at contact point, radius scaled by force multiplier
- At 2x+: compact radial ring from impact (~0.1s)
- At 3x+: HDR >2.0 bloom engulfs breaker front edge
- Distinct from Shockwave (compact, not expanding)

### 13. Time Penalty VFX

- **Geo**: Red-orange energy line from event source to timer position, fading ~0.2s
- **Shader**: Timer HUD glitches briefly (chromatic split, scan line distortion, 2-3 frames). Timer flashes danger-red ~0.3s. Single danger vignette pulse.

### Translation Layer

Gameplay effect messages → module-owned render messages:
- `EffectTriggered { kind: Shockwave, ... }` → `SpawnShockwaveVfx { position, radius, intensity }`
- Each VFX module (`rendering/vfx/shockwave/`, etc.) defines its own message type and systems
- Standard Bevy message systems (not observers) — run in parallel via scheduler
- `VfxKind` enum exists for RON data dispatch; a single dispatch system in effect/ translates enum → module message

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: distortion, bloom, flash), 5e (particles: Spark, Energy Ring, Glow Mote, Electric Arc, Trail), 5k (screen effects: shake, distortion triggers)
- **Enhanced by**: 5g (bolt visuals for halo tinting in ramping damage), 5h (breaker visuals for quick stop)

## What This Step Builds

- 7 combat effects with full Geo + Shader layers: Shockwave, Chain Lightning, Piercing Beam, Pulse, Explode, Gravity Well, Tether Beam
- 3 ambient/passive effect visuals: Attraction/Magnetism, Ramping Damage, Random Effect (Flux)
- 3 utility effect visuals: Quick Stop, Time Penalty, Bump Force Boost
- Translation layer: gameplay effect messages → module-owned render messages per VFX module
- VfxKind enum for RON data dispatch

## Verification

- Each combat effect has a visible Geo + Shader representation
- Shockwave distorts the screen
- Chain lightning emits VfxComplete back to gameplay
- Gravity well warps the scene persistently
- All effects use correct particle types
- All existing tests pass
