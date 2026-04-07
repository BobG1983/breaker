# 5p: Evolution VFX — AoE

## Summary

Implement bespoke VFX for three area-of-effect evolutions: Supernova, Gravity Well (evolution tier), and Dead Man's Hand. These evolutions all produce screen-filling area effects but with fundamentally different visual identities — Supernova is emergent chaos from cascade overlap, Gravity Well is persistent spatial warping, and Dead Man's Hand is a dramatic loss-triggered detonation. Dead Man's Hand has a pending mechanic rework (Phase 7), so its VFX is provisional.

## Context

Key architecture changes from the LEGACY plan:
- **No recipe system.** Each evolution VFX is a direct Rust function/system.
- **rantzsoft_particles2d** for particle effects (spark bursts, glow motes, energy rings).
- **rantzsoft_postprocess** for screen effects (radial distortion, screen flash, screen shake).
- **Visual types from visuals/ domain** (5e) — Hue, GlowParams, VisualModifier, ModifierStack.
- **DR-9 corrections**:
  - Supernova is NOT a single screen-filling blast. It is a chain cascade — the spectacle is emergent from many overlapping base shockwave/bolt-spawn effects, each with a subtle Supernova visual marker (brighter ring, extra spark density).
  - Dead Man's Hand mechanic needs a bigger payoff rethink. Current mechanic (shockwave + speed boost on bolt loss) is underwhelming for an evolution. Design deferred to Phase 7. VFX here is provisional.

## Evolutions in This Batch

### Supernova

**Mechanic summary**: On perfect bump + cell impact + cell destroyed, spawns 2 inheriting bolts + a shockwave. The cascade happens because spawned bolts inherit the evolution, so their cell kills also trigger spawns + shockwaves. Effect compounds exponentially until cells run out.

Current RON (`supernova.evolution.ron`):
- Trigger: PerfectBumped -> Impacted(Cell) -> CellDestroyed -> `Do(SpawnBolts(count: 2, inherit: true))` + `Do(Shockwave(base_range: 96.0, stacks: 1, speed: 400.0))`
- Ingredients: Piercing Shot x3 + Surge x1

**VFX design (from DR-9)**:
- NOT a single authored blast. The visual identity IS the cascade — many overlapping effects creating emergent density.
- Base shockwave and bolt-spawn VFX from 5l fire normally for each trigger.
- Supernova-triggered shockwaves get a subtle visual distinction: brighter ring (higher HDR), extra spark density (more particles per burst).
- Supernova-spawned bolts get a brief prismatic birth trail (fades after ~0.3s).
- Additive blending means overlapping rings and sparks naturally produce visual intensity in cascade zones.
- No bespoke "supernova explosion" animation — the effect composes from many instances of enhanced base effects.

**What to implement**:
1. Supernova shockwave variant: when a shockwave is triggered by Supernova attribution, use enhanced parameters — higher HDR ring (1.8 vs base 1.2), more spark particles (16 vs base 8), slightly different ring color (warmer white / faint gold tint).
2. Supernova bolt-spawn marker: newly spawned bolts from Supernova get a brief `VisualModifier` for a prismatic birth trail that auto-removes after ~0.3s.
3. Attribution detection: check chip attribution on shockwave/bolt-spawn effects to route to Supernova variants.

### Gravity Well (Evolution)

**Mechanic summary**: On cell destroyed, spawns a gravity well entity (strength 500, duration 5s, radius 160, max 4 active) that pulls bolts toward the destruction point.

Current RON (`gravity_well.evolution.ron`):
- Trigger: CellDestroyed -> `Do(GravityWell(strength: 500.0, duration: 5.0, radius: 160.0, max: 4))`
- Ingredients: Bolt Size x2 + Magnetism x2

**VFX design (from DR-9 and effects-particles.md)**:
- Larger and more intense version of the base gravity well VFX from 5l.
- Wider visible radius than base (160 vs base ~80).
- Stronger screen-space distortion lens — visible background warping that noticeably bends nearby entity visuals (gameplay positioning unchanged, purely visual warping).
- More intense rotation animation in the distortion pattern.
- Higher density of glow mote particles drifting inward toward the center.
- Energy ring pulse particle on activation (expanding ring that fades quickly).
- The evolution tier should make the gravity well feel like a significant spatial presence, not just a subtle tug.

**What to implement**:
1. Evolution gravity well VFX function: called when GravityWell fires with evolution attribution. Spawns enhanced persistent visuals:
   - Stronger `TriggerRadialDistortion` (intensity ~0.6, centered on well position, radius 160)
   - Inward-drifting glow mote particles (ContinuousEmitter, ~12 particles, slow inward drift)
   - Slow rotation animation on the distortion pattern
2. Activation burst: energy ring particle + brief screen flash on gravity well spawn.
3. Deactivation: fade out distortion and particles over ~0.3s when well expires.
4. Distinction from base: base gravity well (5l) uses smaller radius, weaker distortion, fewer motes. Evolution version uses all the same VFX types but at higher intensity.

### Dead Man's Hand

**Mechanic summary**: On bolt loss, fires a shockwave (range 128, speed 500) + speed boost (1.5x) on all remaining bolts. Mechanic is pending a full rethink in Phase 7 — current payoff is underwhelming for an evolution.

Current RON (`dead_mans_hand.evolution.ron`):
- Trigger: BoltLost -> `Do(Shockwave(base_range: 128.0, stacks: 1, speed: 500.0))` + `Do(SpeedBoost(multiplier: 1.5))`
- Ingredients: Damage Boost x3 + Last Stand x1

**VFX design (from DR-9 — provisional, pending mechanic rework)**:
- Dramatic shockwave centered on where the bolt was lost — bigger and more intense than any base shockwave.
- Visual speed-up effect on remaining bolts: trails flare longer, glow intensifies, conveying "fury upon loss."
- The emotion is rage/empowerment after a loss — the remaining bolts become visually supercharged.
- Screen shake (heavy tier) on trigger.
- Screen flash in OrangeRed.
- Brief slow-mo (~0.1s) at the moment of loss for dramatic beat.

**What to implement**:
1. Dead Man's Hand VFX function (provisional): fires when the shockwave/speed boost triggers from Dead Man's Hand attribution.
   - Enhanced shockwave: expanding ring at higher HDR (2.0), larger radius, OrangeRed color, with radial distortion.
   - Screen effects: heavy screen shake, OrangeRed screen flash, brief slow-mo trigger.
   - Speed-up visual on remaining bolts: `VisualModifier` for increased trail length and glow intensity.
2. Mark all Dead Man's Hand VFX code clearly as provisional — it will be revised when the mechanic is reworked in Phase 7.

## What to Build

### 1. Supernova Shockwave Variant

- In the shockwave VFX function (from 5l), add a branch for Supernova attribution:
  - Ring HDR: 1.8 (vs base 1.2)
  - Spark burst count: 16 particles (vs base 8)
  - Ring color: faint gold/warm white tint (vs base white)
- Supernova bolt-spawn prismatic trail: on bolt spawn from Supernova, apply a timed `VisualModifier` (prismatic color cycle, auto-remove after 0.3s).

### 2. Evolution Gravity Well VFX

- Gravity well VFX function with evolution-tier parameters:
  - Persistent radial distortion (intensity 0.6, radius 160) anchored to well entity position
  - Continuous inward glow mote emitter (ContinuousEmitter, ~12 motes, slow drift speed ~30 units/sec, inward toward center, HDR 0.5, color midnight blue tones)
  - Distortion rotation animation (slow rotation ~0.3 rad/s)
- Activation burst: RadialBurst particles + screen flash
- Deactivation fade: systems to lerp distortion intensity and particle emission to zero over ~0.3s before despawn
- Anchored entity tracking: VFX entities follow the gravity well entity's position each frame

### 3. Dead Man's Hand Provisional VFX

- Enhanced shockwave function: expanding ring (HDR 2.0, OrangeRed, radius 128, speed 500)
- Radial distortion at loss position (intensity 0.5, duration 0.4s)
- Screen effects: heavy shake, OrangeRed flash (intensity 1.5, ~4 frames)
- Slow-mo trigger (~0.1s at factor 0.3)
- Speed-up modifier on remaining bolts: `VisualModifier` for trail length increase and glow intensity boost

### 4. Evolution VFX Attribution Routing (Shared)

- Extend the evolution VFX routing mechanism from 5q (Nova Lance) to handle Supernova, Gravity Well, and Dead Man's Hand attributions.
- Each base effect (Shockwave, GravityWell, SpeedBoost) checks chip attribution and routes to evolution-tier VFX when appropriate.

### 5. Tests

- Supernova shockwaves use enhanced parameters (higher HDR, more sparks) vs base shockwaves
- Supernova bolt spawns receive a prismatic birth trail modifier
- Gravity Well evolution spawns persistent distortion + mote particles anchored to well entity
- Gravity Well VFX cleans up (fade + despawn) when the well entity expires
- Dead Man's Hand triggers enhanced shockwave + screen effects at bolt loss position
- Dead Man's Hand applies speed-up modifiers to remaining bolt entities
- Attribution routing correctly distinguishes base from evolution for all three evolutions

## What NOT to Do

- Do NOT create a single "supernova explosion" animation. The Supernova visual identity is the emergent cascade, not a bespoke blast.
- Do NOT implement the Dead Man's Hand mechanic rework. VFX is provisional — the mechanic changes in Phase 7.
- Do NOT modify base shockwave or gravity well VFX from 5l. Evolution variants are separate code paths triggered by attribution, not parameter overrides on the base.
- Do NOT create visual distortion that affects gameplay positioning. Gravity well distortion is purely visual — bolt physics are handled by the GravityWell mechanic.
- Do NOT create a recipe system. All VFX is direct Rust function calls.

## Dependencies

- **5l** (combat effect VFX): base Shockwave VFX, base Gravity Well VFX, and base SpeedBoost VFX must exist. Evolution variants build on these as enhanced versions.
- **5c** (rantzsoft_particles2d): spark bursts for shockwaves, glow mote emitters for gravity wells, energy ring particles.
- **5d** (rantzsoft_postprocess): radial distortion for gravity wells and Dead Man's Hand, screen flash, chromatic aberration.
- **5e** (visuals/ domain): VisualModifier for speed-up visuals on bolts, prismatic birth trail modifier.
- **5k** (bump/failure VFX): screen shake infrastructure, slow-mo trigger.
- **5q** (evo beams): evolution VFX routing mechanism (attribution detection, base-vs-evolution branching).

## Verification

- Supernova-triggered shockwaves are visibly brighter and spark-denser than base shockwaves
- Supernova cascade produces emergent visual density from overlapping enhanced effects
- Gravity Well evolution distortion is visibly more intense and wider than base
- Gravity Well glow motes drift inward persistently while well is active
- Gravity Well VFX fades and despawns cleanly when well expires
- Dead Man's Hand fires dramatic shockwave + screen effects at bolt loss position
- Dead Man's Hand visually speeds up remaining bolts (longer trails, brighter glow)
- Additive blending creates visible intensity buildup in overlap zones
- Attribution routing correctly identifies each evolution
- All existing tests pass

## Status: NEEDS DETAIL
