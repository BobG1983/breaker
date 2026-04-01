# Effects

## Combat Effects (Triggered)

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Shockwave | NONE | Critical | High | High | COVERED | Entity with expansion logic, damage in radius. No visual ring, no distortion. |
| Chain Lightning | NONE | Critical | High | High | COVERED | Arc entities track targets + damage. No visual arcs rendered. |
| Piercing Beam | NONE | Critical | High | High | COVERED | Beam entities process damage. No visible beam line. |
| Pulse | NONE | Important | High | High | COVERED | Emitter + ring expansion. No visual filled circle or distortion. |
| Explode | NONE | Critical | High | High | COVERED | Instant area damage. No flash, no burst, no particles. |
| Gravity Well | NONE | Critical | High | High | COVERED | Pull force entities. No distortion lens, no radius indicator. |
| Tether Beam | NONE | Critical | High | High | COVERED | Beam damage between bolts. No visible beam line. |
| Attraction/Magnetism | NONE | Low | Low-Med | Medium | COVERED | Faint curved arcs (<0.3 HDR) between bolt and target. Wake trail bends toward target. Brighten at close range. Minimal by design. |
| Ramping Damage (Amp) | NONE | Low | Low-Med | Medium | COVERED | Halo shifts warmer with consecutive hits (base→amber→white-hot). Orbiting energy ring spins faster. On whiff reset: heat drains, ring shatters. |
| Random Effect (Flux) | NONE | Low | Medium | Medium | COVERED | Brief prismatic flash (~0.1s, 3-4 spectral colors) then resolves to selected effect's visual. Multi-colored spark starburst. Fast and subtle. |

### Implementation

**Shockwave:** Recipe `"shockwave"`: `ExpandingRing(speed: 400.0, max_radius: 32.0, thickness: 2.0, hdr: 1.2, color: White, lifetime: 0.3) + RadialDistortion(intensity: 0.3, duration: 0.3) + ScreenShake(tier: Small) + SparkBurst(count: 8, velocity: 200.0, hdr: 0.8, color: White, gravity: 50.0, lifetime: 0.2)`. Fired by `effect/` on Shockwave `fire()`. For stacked Shockwave: game scales `max_radius` in the direct primitive path instead of recipe.

**Chain Lightning:** No recipe — code-composed. For each arc in the chain, game sends `SpawnElectricArc { start: cell_a_pos, end: cell_b_pos, jitter: 3.0, flicker_rate: 8.0, hdr: 1.0, color: DodgerBlue, lifetime: 0.15 }`. Multiple arcs fire for multi-target chains.

**Piercing Beam:** Recipe `"piercing_beam"`: `Beam(direction: Forward, range: 200.0, width: 3.0, hdr: 1.2, color: White, shrink_duration: 0.1, afterimage_duration: 0.05)`. Fired at bolt position with `direction` from bolt velocity.

**Pulse:** Recipe `"pulse"`: `ExpandingDisc(speed: 500.0, max_radius: 40.0, hdr: 0.8, color: White, lifetime: 0.2) + RadialDistortion(intensity: 0.2, duration: 0.2) + ScreenShake(tier: Micro)`.

**Explode:** Recipe `"explode"`: `ExpandingRing(speed: 600.0, max_radius: 24.0, thickness: 3.0, hdr: 2.0, color: White, lifetime: 0.15) + SparkBurst(count: 16, velocity: 300.0, hdr: 1.5, color: White, gravity: 60.0, lifetime: 0.2) + ScreenShake(tier: Small) + ScreenFlash(color: White, intensity: 1.5, duration_frames: 2)`.

**Gravity Well:** No recipe for the persistent effect — code-composed with anchored primitives. On `fire()`: `ExecuteRecipe { recipe: "gravity_well_ambient", source: Some(well_entity) }`. Recipe: `AnchoredDistortion(entity: Source, radius: 80.0, intensity: 0.4, rotation_speed: 0.5) + AnchoredGlowMotes(entity: Source, count: 6, drift_speed: 20.0, radius: 60.0, hdr: 0.3, color: MidnightBlue, inward: true)`. On `reverse()`: well entity despawns → anchored primitives self-despawn.

**Tether Beam:** No recipe for the persistent beam — code-composed. On `fire()`: `ExecuteRecipe { recipe: "tether_beam", source: Some(bolt_a), target: Some(bolt_b) }`. Recipe: `AnchoredBeam(entity_a: Source, entity_b: Target, width: 1.5, hdr: 0.6, color: White, energy_flow_speed: 3.0, elasticity: 0.5)`. On break: `ExecuteRecipe { recipe: "tether_snap" }` at midpoint: `SparkBurst + ScreenFlash`.

**Attraction/Magnetism:** No recipe — code-composed with anchored primitive. On `fire()`: `ExecuteRecipe { recipe: "attraction_arc", source: Some(bolt), target: Some(target) }`. Recipe: `AnchoredArc(entity_a: Source, entity_b: Target, curvature: 0.3, hdr: 0.3, color: White, flicker_rate: 4.0, jitter: 1.0)`.

**Ramping Damage:** No recipe for the persistent ring — code-composed. On `fire()`: `ExecuteRecipe { recipe: "ramping_ring", source: Some(bolt) }`. Recipe: `AnchoredRing(entity: Source, radius: 16.0, thickness: 0.8, hdr: 0.4, color: White, rotation_speed: 1.0)`. Each hit: `SetModifier(RotationSpeed(hit_count * 0.5), source: "ramping")` + `SetModifier(ColorShift([lerp White→Gold→White based on count]), source: "ramping_color")`. On whiff reset: `RemoveModifier(source: "ramping")` + recipe `"ramping_reset"`: `SparkBurst(count: 4, ..., color: Gold)`.

**Random Effect (Flux):** Recipe `"flux_flash"`: `SparkBurst(count: 10, velocity: 150.0, hdr: 0.6, color: White, gravity: 0.0, lifetime: 0.1)`. Colors are multi-hued — each spark gets a random spectral color (implementation detail in the emitter). Fires before the selected effect's recipe.

---

## Defensive / Utility Effects

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Shield (bolt-loss) | NONE | Critical | High | High | COVERED | `ShieldActive` charges are logic-only. No bottom barrier. |
| Second Wind wall | NONE | Important | High | Medium | COVERED | Invisible wall, no materialization flash. |
| Quick Stop | NONE | Low | Low | Low | COVERED | Aura compresses forward (squash, 2-3 frames). Spark spray forward from leading edge. Trail abruptly terminates. Micro-distortion at high stacks. |
| Time Penalty | NONE | Low | Low-Med | Medium | COVERED | Timer red-orange flash + glitch (chromatic split, jitter, 2-3 frames). Red-orange energy line from source to timer. Single danger vignette pulse. |
| Life Lost | NONE | Critical | High | High | COVERED | Decrements lives silently. Style guide: slow-mo + vignette. |

### Implementation

**Shield:** Not a recipe — dedicated `shield.wgsl` entity. Spawned on `ShieldActive` added. See Entities catalog > Walls & Background > Shield barrier.

**Second Wind wall:** Recipe `"second_wind_save"`: `ExpandingRing(speed: 500.0, max_radius: 60.0, thickness: 3.0, hdr: 1.5, color: White, lifetime: 0.2) + ScreenFlash(color: White, intensity: 1.0, duration_frames: 3) + ScreenShake(tier: Small)`. Fired at the bottom wall position when the invisible wall saves a bolt. Plus `TriggerSlowMotion { factor: 0.5, duration: 0.15 }` (brief dramatic pause).

**Quick Stop:** Recipe `"quick_stop"`: `SparkBurst(count: 4, velocity: 100.0, hdr: 0.4, color: White, gravity: 10.0, lifetime: 0.1)`. Fired at breaker leading edge. Plus modifier: `SetModifier(SquashStretch { x_scale: 0.85, y_scale: 1.15 }, source: "quick_stop")` for 2-3 frames (system clears it after).

**Time Penalty:** Recipe `"time_penalty"`: `Beam(direction: N, range: 400.0, width: 1.0, hdr: 0.6, color: OrangeRed, shrink_duration: 0.2, afterimage_duration: 0.0) + VignettePulse(color: OrangeRed, intensity: 0.3, duration: 0.3)`. Fired at event source position toward timer. Plus `TriggerChromaticAberration { intensity: 0.3, duration: 0.15 }` sent directly.

**Life Lost:** No recipe. Direct primitive messages from `effect/`: `TriggerSlowMotion { factor: 0.2, duration: 0.5 }` + `TriggerVignettePulse { camera, color: OrangeRed, intensity: 0.4, duration: 0.5 }`. Life orb dissolve handled by HUD system (see UI catalog).

---

## Visual Modifiers (Active on Bolt/Breaker)

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Speed Boost (bolt) | NONE | Important | Medium | High | COVERED | Modifies speed. No trail extension, no glow change. |
| Damage Boost (bolt) | NONE | Important | Medium | High | COVERED | Modifies multiplier. No color shift, no brightness change. |
| Piercing (bolt state) | NONE | Important | Medium | High | COVERED | Bolt passes through. No angular glow, no energy spikes. |
| Size Boost (bolt) | PARTIAL | Low | Low | Low | COVERED | Scale changes. No proportional glow scaling. |
| Size Boost (breaker) | PARTIAL | Low | Low | Low | COVERED | Width scales. No aura stretch. |
| Bump Force Boost | NONE | Low | Low | Medium | COVERED | On bump: concentrated flash scaled by multiplier. At 2x+: compact radial ring from impact. At 3x+: HDR >2.0 bloom at impact. Distinct from Shockwave (compact, not expanding). |
| Visual Modifier System | NONE | Important | High | High | COVERED | Diminishing-returns stacking system. Not implemented. |

### Implementation

All visual modifiers use `AddModifier`/`RemoveModifier` messages — no recipes. Each effect's `fire()` sends the modifier, `reverse()` removes it.

| Effect | Modifier Message | Source Key |
|--------|-----------------|------------|
| Speed Boost (bolt) | `AddModifier(TrailLength(1.5))` + `AddModifier(GlowIntensity(1.2))` | `"speed_boost"` |
| Damage Boost (bolt) | `AddModifier(ColorShift(Gold))` + `AddModifier(CoreBrightness(1.3))` | `"damage_boost"` |
| Piercing (bolt) | `AddModifier(SpikeCount(4))` | `"piercing"` |
| Size Boost (bolt) | `AddModifier(ShapeScale(size_mult))` + `AddModifier(GlowIntensity(size_mult * 0.8))` | `"size_boost"` |
| Size Boost (breaker) | `AddModifier(ShapeScale(width_mult))` | `"width_boost"` |
| Bump Force Boost | `AddModifier(CoreBrightness(1.0 + force_mult * 0.5))` | `"bump_force"` |

**Bump Force impact VFX** (fires on bump, not persistent): Recipe `"bump_force_impact"`: `ExpandingRing(speed: 200.0, max_radius: 12.0, thickness: 2.0, hdr: 1.5, color: White, lifetime: 0.1) + ScreenFlash(color: White, intensity: 0.4, duration_frames: 1)`. Recipe scales with force multiplier — at 3x+, use a brighter variant `"bump_force_impact_heavy"`.

**Visual Modifier System:** Implemented in step 5n. `ModifierStack` component + DR computation. See `docs/architecture/rendering/modifiers.md`.
