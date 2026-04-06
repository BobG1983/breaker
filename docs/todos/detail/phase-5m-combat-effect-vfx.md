# 5j: Combat Effect VFX

## Summary

Implement per-effect visual functions for all triggered combat effects. Each chip effect's `fire()` function gains a companion VFX function that spawns particles, triggers postprocess effects, and applies entity modifiers to create the visual spectacle for that effect. Every VFX is a direct Rust function — no recipe system, no RON-driven VFX sequences. Currently effects have gameplay logic but no visuals; this phase makes them visible.

## Context

The combat effect system (effect/ domain) already has working gameplay logic: shockwave damages cells in a radius, chain lightning jumps between targets, piercing beam passes through cells, etc. But these effects are invisible — the gameplay happens, the player just cannot see it. This phase adds the visual layer.

Key architecture change from the LEGACY plan: the old plan used RON visual recipes (`assets/recipes/shockwave_default.ron`) composed from crate-owned primitive steps (`ExpandingRing`, `SparkBurst`, `Beam`, etc.). The new architecture eliminates recipes entirely. Each effect's VFX is a Rust function that directly:
- Spawns `ParticleEmitter` entities with appropriate presets (from rantzsoft_particles2d)
- Sends postprocess trigger messages (`TriggerRadialDistortion`, `TriggerScreenShake`, etc.)
- Sends modifier messages (`AddModifier`, `SetModifier`) for entity visual changes
- Spawns transient visual entities (expanding ring meshes, beam quad meshes) with `EntityGlowMaterial`

The visual design for each effect comes from `docs/design/graphics/effects-particles.md`. Every effect follows the Geo + Shader layering principle: a clean geometric shape the player reads at 60fps (geo layer) enhanced with bloom, distortion, and noise (shader layer).

## What to Build

### 1. Shockwave VFX

Visual: expanding ring from impact point. Ring thickness represents damage falloff.

- **Geo layer**: Spawn a ring entity (annular quad mesh) with `EntityGlowMaterial`. The ring expands outward over ~0.3s via a per-frame system that scales the entity's `Transform`. Ring color matches the source effect's tint. Core brightness HDR >1.5.
- **Shader layer**: `TriggerRadialDistortion` at the ring's position — screen refracts at the ring edge. `TriggerScreenShake { tier: Small }`. Brief bloom from HDR ring entity.
- **Spark burst**: `ParticleEmitter` with `Burst { count: 16 }` at impact point, radial velocity, short lifetime.
- **Stacking**: Higher chip stacks = larger maximum radius on the ring entity, thicker ring width, brighter HDR, stronger distortion intensity parameter.
- **Cleanup**: Ring entity despawns after expansion completes (~0.3s). Distortion source auto-expires via lifecycle system (5j).

VFX function signature: `fn shockwave_vfx(commands: &mut Commands, position: Vec2, radius: f32, stacks: u32, ...)`

### 2. Chain Lightning VFX

Visual: branching jagged lines connecting source cell to target cells.

- **Geo layer**: For each arc in the chain, spawn a line entity (thin quad mesh oriented between start and end points) with `EntityGlowMaterial`. Lines are jagged — subdivide the line into 4-6 segments with random perpendicular offsets to create a zigzag pattern. Update vertex positions each frame for 1-2 frames of "crackling" (re-randomize offsets), then fade.
- **Shader layer**: Brightness fluctuates rapidly on each line segment (per-frame random intensity jitter). Brief bloom flash at each target cell: `AddModifier` with `GlowIntensity(3.0)` for 0.05s on each target cell entity.
- **Spark points**: `ParticleEmitter` with `Burst { count: 4 }` at each branch point (where one arc meets the next), electric blue/white color.
- **Duration**: Near-instant — arcs appear, flicker for 2-3 frames, then fade to zero alpha over 2 frames. Speed communicates power.
- **Stacking**: More stacks = more chain targets = more arcs visible simultaneously.
- **Runtime composition**: This is code-composed (not recipe-composable) because the number and endpoints of arcs are determined at runtime by the chain resolution logic.

VFX function: `fn chain_lightning_vfx(commands: &mut Commands, arcs: &[(Vec2, Vec2)], ...)`

### 3. Piercing Beam VFX

Visual: straight line extending from bolt through pierced cells.

- **Geo layer**: Spawn a beam entity (elongated quad mesh along the pierce trajectory) with `EntityGlowMaterial`. Beam has a bright core (HDR >2.0) narrowing toward the end (width tapers via UV-based alpha in the shader, or via mesh vertices tapering).
- **Shader layer**: Brief distortion along beam path: `TriggerRadialDistortion` at beam midpoint with elongated shape. Bloom from HDR core.
- **Duration**: Appears on pierce, lingers for ~0.1s as a fading afterimage (alpha ramps from 1.0 to 0.0 over 0.1s).
- **Cleanup**: Beam entity despawns after fade completes.

VFX function: `fn piercing_beam_vfx(commands: &mut Commands, start: Vec2, end: Vec2, ...)`

### 4. Pulse VFX

Visual: expanding filled circle from source position.

- **Geo layer**: Spawn a disc entity (circle quad with `EntityGlowMaterial`, `Shape::Circle`) that expands from zero radius to pulse radius over ~0.2s. Radial gradient from bright center to soft edge (handled by entity_glow shader's halo computation — the disc IS the core, the halo IS the gradient).
- **Shader layer**: `TriggerRadialDistortion` at the disc edge. `TriggerScreenShake { tier: Small }`. Cells inside the pulse: `AddModifier` with `GlowIntensity(2.0)` for 0.05s on each affected cell (brief flash).
- **Duration**: Faster than shockwave — ~0.2s full expansion. Pulse is immediate, shockwave is dramatic.
- **Cleanup**: Disc entity despawns after expansion.

VFX function: `fn pulse_vfx(commands: &mut Commands, position: Vec2, radius: f32, affected_cells: &[Entity], ...)`

### 5. Explode VFX

Visual: radial burst from destroyed cell. Fastest effect.

- **Geo layer**: Multiple short beam entities (6-8, radial, ~0.15s lifetime) radiating from center with `EntityGlowMaterial`. Each beam is a thin elongated quad pointing outward.
- **Shader layer**: Central flash: `TriggerScreenFlash` with HDR >2.0, white, 2-frame duration. `TriggerScreenShake { tier: Small }`. Brief distortion ring: `TriggerRadialDistortion` at center.
- **Particle spray**: `ParticleEmitter` with `Burst { count: 32 }`, high radial velocity, short lifetime (0.1-0.15s). Dense spray.
- **Duration**: Fastest effect — flash, burst, fade in ~0.15s. Maximum immediacy.

VFX function: `fn explode_vfx(commands: &mut Commands, position: Vec2, ...)`

### 6. Gravity Well VFX

Visual: persistent distortion lens at well position.

- **Geo layer**: Circle entity showing the well's radius of influence with `EntityGlowMaterial`, very low alpha (outline only). Faint radial lines (4-6 thin beam entities) pulling inward, slowly rotating.
- **Shader layer**: Persistent `TriggerRadialDistortion` at well center — the area inside warps the background, bending light toward center. Distortion intensifies toward center (intensity parameter scaled by distance). Subtle animation: slow rotation of distortion pattern via per-frame angle update.
- **Glow motes**: `ParticleEmitter` with `EmissionMode::Continuous { rate: 3.0 }` spawning dim glow mote particles that drift inward toward center (gravity parameter on particles points toward well center). Ambient, long-lifetime particles.
- **Duration**: Persistent while the gravity well effect is active. VFX entities are spawned when the well activates and despawned when it deactivates.
- **Stacking**: Larger radius, stronger distortion, more glow motes.
- **No dark void**: The gravity well stays within the "light is the material" identity — warping and refraction, not absence of light.

VFX function: `fn gravity_well_vfx_spawn(commands: &mut Commands, position: Vec2, radius: f32, source_entity: Entity, ...) -> Entity` (returns the VFX entity group for later cleanup)
Cleanup function: `fn gravity_well_vfx_despawn(commands: &mut Commands, vfx_entity: Entity)`

### 7. Tether Beam VFX

Visual: energy beam connecting two tethered bolts.

- **Geo layer**: Line entity (quad mesh) connecting bolt A to bolt B positions. Line has slight elasticity visual — width modulates based on distance (thinner when stretched, thicker when close). Updated each frame to track bolt positions.
- **Shader layer**: Energy flow along beam — animated brightness traveling from one end to the other (UV offset scrolling along the beam axis, handled by a dedicated `tether_beam.wgsl` or by EntityGlowMaterial with rotation_angle repurposed as a scroll parameter). Color matches bolt halo color.
- **Snap effect**: When the tether breaks (constraint removed), the beam entity plays a brief bright flash (HDR spike) + `ParticleEmitter` burst at both endpoints, then despawns.
- **Duration**: Persistent while the tether is active.

VFX function: `fn tether_beam_vfx_spawn(commands: &mut Commands, bolt_a: Entity, bolt_b: Entity, color: Hue, ...) -> Entity`
Update system: tracks bolt positions each frame, updates beam mesh vertices.

### 8. Shield Effect VFX (Bolt-Loss Protection)

Visual: energy barrier along bottom playfield edge.

- **Geo layer**: Barrier entity rendered along the bottom wall with a dedicated shield barrier shader or `EntityGlowMaterial` with Shield shape. Solid line, brighter than normal wall glow. Honeycomb/hexagonal pattern (per DR-3: patterned white).
- **Shader layer**: Shimmer/ripple animation — UV-based sine wave distortion on the pattern, scrolling over time.
- **Charge consumption**: On each charge used, barrier flashes bright + crack particles spawn at absorption point (handled in 5k shield absorption VFX). Visual crack lines accumulate on the barrier entity (could be a uniform controlling crack density).
- **Last charge break**: Full shatter VFX (handled in 5k).
- **Duration**: Persistent while charges remain. Spawned when shield effect activates, despawned when charges hit zero.

VFX function: `fn shield_barrier_vfx_spawn(commands: &mut Commands, wall_positions: (Vec2, Vec2), charges: u32, ...) -> Entity`

### 9. Speed Boost VFX

Visual: modifies bolt trail and glow. No standalone geo.

- **Entity modifiers**: `AddModifier` on affected bolt with `TrailLength(2.0)` (double trail) + `GlowIntensity(1.5)` (brighter core). Duration matches effect duration.
- **Speed lines**: `ParticleEmitter` with `EmissionMode::Continuous { rate: 8.0 }` attached to bolt entity, emitting particles in the bolt's wake direction with short lifetime and elongated shape (trail-type particles). Creates motion blur / speed line effect.

VFX function: `fn speed_boost_vfx(commands: &mut Commands, bolt_entity: Entity, ...)`

### 10. Damage Boost VFX

Visual: modifies bolt core brightness and color. No standalone geo.

- **Entity modifiers**: `AddModifier` on bolt with `CoreBrightness(1.5)` + `ColorShift(Gold)` (shift toward hotter amber/white). Duration matches effect duration.
- **Impact enhancement**: When a damage-boosted bolt hits a cell, the cell's bump flash (from 5k) is intensified — the bump VFX function checks for active damage boost and scales particle count and brightness.

VFX function: `fn damage_boost_vfx(commands: &mut Commands, bolt_entity: Entity, ...)`

### 11. Attraction/Magnetism VFX

Visual: faint curved arcs between bolt and attraction target.

- **Geo layer**: 2-3 thin line entities (arc segments) between bolt and target, bending in pull direction. Very dim (HDR <0.3). Lines flicker — per-frame random alpha jitter between 0.0 and base alpha. Brighten at close range (alpha scales inversely with distance).
- **Bolt wake**: bolt's trail bends slightly toward target (modify trail entity positions if trail system supports it, or add subtle `SquashStretch` modifier oriented toward target).
- **Intentionally minimal**: Attraction is ambient steering, not spectacle. The arcs are glimpses of the force field, not a persistent display.
- **Duration**: Persistent while attraction is active. Updated each frame.

VFX function: `fn attraction_vfx_spawn(commands: &mut Commands, bolt: Entity, target: Entity, ...) -> Entity`

### 12. Ramping Damage (Amp) VFX

Visual: orbital ring + progressive color shift on bolt.

- **Geo layer**: Faint energy ring entity orbiting the bolt. Ring brightness and rotation speed increase with each consecutive hit (ramp counter). Ring entity tracks bolt position each frame.
- **Shader layer**: Bolt halo color shifts progressively warmer via `AddModifier` with `ColorShift`: base color at 0 stacks, amber at 3 stacks, white-hot at 6+ stacks. At high stacks (6+), `AddModifier` with `AfterimageTrail(true)` — afterimage frames linger in wake.
- **Whiff reset**: On whiff (ramp counter resets to 0), heat drains visibly over ~0.3s (color shift modifier removed with a slow fade, not instant). Orbital ring shatters outward: `ParticleEmitter` burst with dim sparks radiating from bolt position. The cooldown is a punishing visual moment.
- **Duration**: Persistent while amp effect is active. Stacking visual updates each time the ramp counter changes.

VFX function: `fn ramping_damage_vfx_update(commands: &mut Commands, bolt: Entity, ramp_count: u32, ...)`
Reset function: `fn ramping_damage_vfx_reset(commands: &mut Commands, bolt: Entity, ...)`

### 13. Random Effect (Flux) VFX

Visual: brief prismatic flash before the selected effect fires.

- **Geo layer**: Multi-colored spark starburst from entity: `ParticleEmitter` with `Burst { count: 12 }`, particles cycling through 3-4 spectral colors (red, green, blue, gold), radial velocity, very short lifetime (~0.1s).
- **Shader layer**: Rapid prismatic flash on the source entity: `AddModifier` with `ColorCycle { colors: [Red, Cyan, Gold, Magenta], speed: 30.0 }` for 0.1s — extremely fast cycle that resolves to the selected effect's color.
- **Then**: The selected random effect's VFX function fires normally. Flux is a prefix VFX, not a replacement.

VFX function: `fn flux_vfx(commands: &mut Commands, position: Vec2, source_entity: Entity, ...)`

### 14. Quick Stop VFX

Visual: brief compression + spark spray from breaker.

- **Geo layer**: Brief compression on breaker: `AddModifier` with `SquashStretch { x_scale: 1.3, y_scale: 0.7 }` for 2-3 frames, then snap back.
- **Spark spray**: `ParticleEmitter` with `Burst { count: 8 }` from breaker's leading edge, directional velocity forward (in the breaker's facing direction), short lifetime.
- **High stacks (3+)**: Micro-distortion ripple from breaker position: `TriggerRadialDistortion` with very small radius and low intensity, 1-frame duration. "Momentum converted to stillness."

VFX function: `fn quick_stop_vfx(commands: &mut Commands, breaker_entity: Entity, breaker_position: Vec2, stacks: u32, ...)`

### 15. Bump Force Boost VFX

Visual: concentrated impact flash at contact point.

- **Geo layer**: Impact flash: `TriggerScreenFlash` at contact point, intensity scaled by force multiplier. At 2x+: compact radial ring (small expanding ring entity, much smaller and faster than Shockwave) from impact, ~0.1s duration.
- **Shader layer**: At 3x+: HDR >2.0 bloom engulfs breaker's front edge — `AddModifier` with `GlowIntensity(3.0)` on breaker for 0.1s.
- **Distinct from Shockwave**: Force ring is compact and immediate, not a large expanding ring. Different visual read.

VFX function: `fn bump_force_vfx(commands: &mut Commands, contact_point: Vec2, force_multiplier: f32, breaker_entity: Entity, ...)`

### 16. Time Penalty VFX

Visual: energy line from event source to timer + timer glitch.

- **Geo layer**: Red-orange energy line (beam entity) streaking from event source position to timer's screen position, fading over ~0.2s. Line uses `EntityGlowMaterial` with red-orange color, HDR >1.5.
- **Shader layer**: Timer HUD glitches briefly — if the timer wall gauge (5n) exists, apply `AddModifier` with `SquashStretch` jitter for 2-3 frames + brief chromatic aberration on the timer entity. Timer flashes danger-red: `AddModifier` with `ColorShift(OrangeRed)` for 0.3s.
- **Vignette**: Single brief danger vignette pulse at screen edges: `TriggerVignette` with red-orange, 0.15s duration. Connects cause to consequence visually.

VFX function: `fn time_penalty_vfx(commands: &mut Commands, source_position: Vec2, timer_position: Vec2, ...)`

### 17. Effect VFX Dispatch Integration

Each chip effect's `fire()` function calls the appropriate VFX function after performing its gameplay logic. Each effect's `reverse()` or `expire()` function calls any cleanup (despawn persistent VFX entities, remove modifiers).

This is the integration point: effect gameplay code in `effect/` calls VFX functions that live in the appropriate domain (bolt/, breaker/, or a new `effect/vfx/` module). The VFX functions are pure visual — they do not affect gameplay state.

Pattern:
```
effect/effects/shockwave/fire.rs:
  fn fire(...) {
      // ... gameplay logic (damage cells, etc.) ...
      shockwave_vfx(commands, position, radius, stacks, ...);
  }
```

## What NOT to Do

- Do NOT create a recipe system, recipe store, or RON recipe files — all VFX are direct Rust function calls
- Do NOT create per-effect rendering modules in a VFX crate — VFX functions live game-side
- Do NOT create a `VfxKind` dispatch enum — each effect calls its own VFX function directly
- Do NOT implement evolution VFX — those are 5q-5t (they are bespoke, higher-tier visual experiences)
- Do NOT implement highlight moment VFX — that is 5m
- Do NOT modify effect gameplay logic (damage, targeting, timing) — only add visual calls alongside existing logic
- Do NOT create new postprocess shaders — use the existing ones from rantzsoft_postprocess (5d)

## Dependencies

- **Requires**: 5c (rantzsoft_particles2d — particle emitters for all burst/spray/mote effects), 5d (rantzsoft_postprocess — distortion, screen flash, chromatic aberration), 5e (visuals domain — EntityGlowMaterial, modifier messages, Hue for colors), 5f (bolt visuals — bolt trail entities for speed boost/afterimage), 5g (breaker visuals — breaker entity for quick stop/force boost), 5h (cell visuals — cell entities for flash-on-hit), 5i (wall visuals — wall entities for shield barrier), 5j (screen effects — shake, slow-mo, vignette lifecycle systems, modifier computation)
- **Independent of**: 5m (highlights), 5n (HUD), 5o (chip cards), 5p (screens)
- **Required by**: 5q-5t (evolution VFX — builds on combat VFX base patterns)

## Verification

- Every combat chip effect has visible VFX when it fires
- Shockwave: expanding ring + screen distortion at ring edge + spark burst + small shake
- Chain lightning: jagged arcs between correct targets + crackling flicker + spark at branch points
- Piercing beam: bright tapering beam along trajectory + brief distortion + 0.1s fade
- Pulse: expanding filled circle + edge distortion + cell flash inside radius
- Explode: radial burst lines + central HDR flash + dense particle spray + small shake, all in ~0.15s
- Gravity well: persistent distortion lens + inward glow motes + slow rotation, no dark void
- Tether beam: energy beam tracking two bolt positions + snap flash on break
- Shield barrier: visible hexagonal pattern + shimmer + crack accumulation per charge consumed
- Speed boost: longer trail + brighter glow + speed line particles in wake
- Damage boost: hotter color shift + brighter core
- Attraction: faint flickering arcs between bolt and target, minimal
- Ramping damage: orbital ring scaling with ramp count + color shift + afterimage at 6+ + punishing reset visual
- Flux: prismatic flash starburst + rapid color cycle, then selected effect fires
- Quick stop: squash-stretch compression + forward spark spray, distortion at 3+ stacks
- Bump force: concentrated impact flash, compact ring at 2x+, HDR bloom at 3x+
- Time penalty: red-orange beam to timer + timer glitch + danger vignette
- Stacking correctly scales visual intensity for all stackable effects
- Persistent effects (gravity well, tether, shield, attraction, ramping) spawn and despawn correctly with the effect lifecycle
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL
