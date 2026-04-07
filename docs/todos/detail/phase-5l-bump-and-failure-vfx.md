# 5i: Bump Grade & Failure State VFX

## Summary

Wire gameplay events to visual feedback. Bump grades trigger breaker/bolt flash + particles + screen effects scaled by grade quality. Failure states (bolt lost, life lost, shield absorption, run won, run over, time expired) trigger slow-mo, desaturation, screen flash, and particle bursts. Remove all remaining floating text feedback — the game communicates through light, motion, and screen effects only (Pillar 4).

## Context

This phase connects gameplay events to the visual systems built in earlier phases. The screen effect systems (shake, slow-mo, vignette, desaturation, flash) were built in 5j. The particle crate (rantzsoft_particles2d) was built in 5c. The postprocess crate (rantzsoft_postprocess) was built in 5d. Entity visuals (bolt, breaker, cell, wall) were set up in 5f-5i.

Key architecture change from the LEGACY plan: there is NO recipe system. Each VFX response is a direct Rust function that sends the appropriate messages (particle emitter spawns, postprocess triggers, modifier changes). Where the old plan referenced `ExecuteRecipe` with named recipes like `perfect_bump_recipe` or `bolt_death_recipe`, the new plan uses direct function calls. For example, the bump VFX function for Perfect grade directly spawns a `ParticleEmitter` entity with a radial burst preset, sends `TriggerScreenShake { tier: Micro }`, and sends `TriggerScreenFlash` with gold color — no recipe indirection.

The bump grade feedback design comes from `docs/design/graphics/feedback-juice.md`. The failure state design comes from the same document. All feedback is visual-only per DR-4 and Pillar 4.

## What to Build

### 1. Bump Grade VFX Dispatch

On `BumpPerformed { grade, position, direction }` message, a system dispatches grade-appropriate VFX:

**Perfect bump:**
- Gold/white flash on breaker: `TriggerScreenFlash` with gold tint, low intensity, 2-frame duration
- Spark burst at impact point: spawn `ParticleEmitter` with `EmissionMode::Burst { count: 24 }`, `SpawnParams` with radial velocity, gold/white color, HDR brightness >1.5 for bloom, lifetime 0.15-0.3s
- Micro screen shake: `TriggerScreenShake { tier: Micro, direction: Some(impact_direction) }`
- Breaker glow intensification: `AddModifier` on breaker entity with `GlowIntensity(2.0)` for 0.15s duration
- Bloom spike at impact: brief HDR >2.0 from the particle burst handles this naturally

**Early bump:**
- Dim archetype-color flash on breaker: `TriggerScreenFlash` with breaker's archetype color, very low intensity
- Small spark burst: `ParticleEmitter` with `Burst { count: 8 }`, archetype color, dimmer HDR (~1.2), shorter lifetime
- No screen shake
- Subtle breaker glow: `AddModifier` with `GlowIntensity(1.3)` for 0.1s

**Late bump:**
- Minimal flash: `TriggerScreenFlash` with archetype color, barely visible intensity
- Minimal sparks: `ParticleEmitter` with `Burst { count: 3 }`, dim, very short lifetime
- No screen shake
- No modifier

**Whiff (no bump):**
- Nothing. Silence IS the feedback. No flash, no sparks, no shake, no modifier. The absence teaches the player what they are aiming for.

The VFX dispatch system lives in `breaker-game/src/breaker/systems/bump_vfx.rs`. It reads the breaker's `EntityVisualConfig` (attached in 5g) for archetype color.

### 2. Bolt Lost VFX

When `BoltLost` triggers (bolt passes below playfield, no shield):

- Slow-mo: `TriggerSlowMotion { factor: 0.3, duration: 0.3, ramp_in: 0.05, ramp_out: 0.1 }` — brief dramatic pause
- Exit streak: spawn `ParticleEmitter` with `EmissionMode::Burst { count: 16 }`, directional velocity downward (following bolt's exit trajectory), bright color matching bolt's hue, elongated lifetime for streak effect
- Brief desaturation: `TriggerDesaturation { target_factor: 0.7, duration: 0.3 }` — world briefly loses color
- Bolt entity plays dissolve (set `dissolve_threshold` on EntityGlowMaterial, ramping from 0.0 to 1.0 over ~0.2s via a tween system or manual per-frame update)

System lives in `breaker-game/src/bolt/systems/bolt_lost_vfx.rs`.

### 3. Shield Absorption VFX

When shield absorbs a bolt loss (bolt hits shield barrier instead of falling out):

- Shield barrier flash: brief HDR >2.0 flash on the shield wall entity's `EntityGlowMaterial`
- Crack particles: spawn `ParticleEmitter` at absorption point with angular fragments (shard-like, radial burst upward), white color matching shield pattern (DR-3: patterned white)
- If last charge consumed: barrier shatter effect — multiple `ParticleEmitter` bursts along the barrier length, `TriggerScreenShake { tier: Micro }`, `TriggerScreenFlash` at bottom edge
- No slow-mo — the save is instant and satisfying. The speed contrast with bolt-lost slow-mo makes the shield feel powerful.

System lives in `breaker-game/src/bolt/systems/shield_absorption_vfx.rs` or alongside the shield gameplay system.

### 4. Life Lost VFX

When a life is consumed (bolt lost with remaining lives):

- Longer slow-mo: `TriggerSlowMotion { factor: 0.2, duration: 0.5, ramp_in: 0.05, ramp_out: 0.15 }` — more dramatic than bolt lost
- Danger vignette pulse: `TriggerVignette` with red-orange tint, brief high-intensity pulse
- Life orb dissolve: the HUD life orb (built in 5n) plays dissolve — set `dissolve_threshold` ramping to 1.0, plus spark burst from the orb's position
- Bolt lost VFX also fires (life lost implies bolt lost)

System lives alongside bolt lost VFX or in `breaker-game/src/run/systems/life_lost_vfx.rs`.

### 5. Run Won VFX

On final node cleared (run victory):

- Freeze-frame: `TriggerSlowMotion { factor: 0.0, duration: 0.5, ramp_in: 0.02, ramp_out: 0.3 }` — game stops at the moment of victory, then slowly resumes for transition
- Screen flash: `TriggerScreenFlash` with white, high intensity, 4-frame duration
- Final cell destruction particle burst is enhanced (larger count, brighter) since the last cell triggers the win
- Transition to run-end screen begins after slow-mo ramp-out

System lives in `breaker-game/src/run/systems/run_won_vfx.rs`.

### 6. Run Over VFX (Defeat)

On run end (final life lost or timer expired):

- Extended slow-mo: `TriggerSlowMotion { factor: 0.15, duration: 1.0, ramp_in: 0.05, ramp_out: 0.5 }` — long, contemplative deceleration
- Full desaturation: `TriggerDesaturation { target_factor: 1.0, duration: 0.8 }` — world fades to near-monochrome
- All active particle emitters: reduce emission rate to zero, let existing particles fade naturally (the world is winding down)
- Bolt trail fades: reduce bolt's trail opacity to zero over the slow-mo duration
- Tone is contemplative, not punishing (Pillar 8: failure must feel fair, fast, and forward-looking)

System lives in `breaker-game/src/run/systems/run_over_vfx.rs`.

### 7. Time Expired VFX

When the node timer reaches zero:

- Timer wall effect: the timer wall gauge (built in 5n) shatters — `ParticleEmitter` bursts along the top wall, shard particles scattering downward
- Red-orange vignette: `TriggerVignette` with red-orange, medium intensity, 0.5s duration
- Desaturation from edges: `TriggerDesaturation` ramping to 0.8 over 0.5s
- This precedes run-over VFX if the timer expiry ends the run

System lives in `breaker-game/src/run/systems/time_expired_vfx.rs`.

### 8. Remove Floating Text Feedback

Remove all existing floating text feedback systems:
- "PERFECT", "EARLY", "LATE", "WHIFF" text popups on bump
- "BOLT LOST" text
- Any other gameplay-event floating text

These are fully replaced by the visual-only systems above. Search the codebase for `Text2d` spawns related to gameplay feedback and remove them. The highlight moment text labels ("SAVE.", "OBLITERATE.", etc.) are different — those are built in 5m with GlitchText treatment and are not removed here.

### 9. Cell Destruction Context Detection

Cell destruction visuals scale with context (from `docs/design/graphics/effects-particles.md`):

| Context | Visual |
|---------|--------|
| Single cell break | Clean dissolve + brief spark burst (6-8 sparks) |
| Combo chain (2-4 cells rapid succession) | Shatter — cells fracture into shard particles, scatter outward |
| Chain reaction / shockwave kill (5+ cells) | Energy release — detonation rings, screen pulse, high particle density |
| Evolution-triggered mass destruction | Evolution-specific VFX (5q-5t) on top of the above |

A context detection system tracks recent cell destructions within a short time window (~0.3s) and escalates the death effect tier. The cell death VFX function reads the current context tier and dispatches the appropriate particle configuration.

System lives in `breaker-game/src/cells/systems/cell_death_vfx.rs`.

## What NOT to Do

- Do NOT implement combat effect VFX (shockwave ring, chain lightning arcs, etc.) — that is 5l
- Do NOT implement highlight moment VFX — that is 5m
- Do NOT implement HUD elements (life orbs, timer wall) — that is 5n. Reference their existence for dissolve/shatter effects but do not build them.
- Do NOT create a recipe system — all VFX dispatch is direct function calls
- Do NOT modify the screen shake, slow-mo, or postprocess lifecycle systems built in 5j — only send messages to them
- Do NOT add new shader effects — use the existing EntityGlowMaterial dissolve and postprocess triggers

## Dependencies

- **Requires**: 5c (rantzsoft_particles2d — particle emitters for bursts), 5d (rantzsoft_postprocess — screen flash, desaturation, vignette, chromatic aberration triggers), 5e (visuals domain — EntityGlowMaterial for dissolve, modifier messages), 5f (bolt visuals — bolt trail, bolt hue for death streak color), 5g (breaker visuals — archetype color for bump flash), 5h (cell visuals — cell death visual baseline), 5i (wall visuals — shield barrier entities), 5j (screen effects — shake, slow-mo, vignette, desaturation lifecycle systems)
- **Independent of**: 5l (combat VFX), 5m (highlights), 5n (HUD), 5o (chip cards), 5p (screens)
- **Required by**: 5m (highlights — uses screen flash established here)

## Verification

- No floating text remains in the game for bump grades, bolt lost, or any gameplay event
- Perfect bump produces gold flash + 24-spark burst + micro-shake + breaker glow intensification
- Early bump produces dim archetype flash + 8-spark burst, no shake
- Late bump produces minimal flash + 3 sparks, no shake
- Whiff produces zero visual feedback
- The gradient from Perfect to Whiff is dramatically visible during gameplay
- Bolt lost triggers slow-mo (0.3s at 30%) + exit streak + desaturation
- Shield absorption triggers barrier flash + crack particles, no slow-mo
- Shield last-charge break triggers barrier shatter + micro-shake + flash
- Life lost triggers longer slow-mo (0.5s at 20%) + vignette pulse + bolt-lost VFX
- Run won triggers freeze-frame + white flash + enhanced final cell burst
- Run over triggers extended slow-mo + full desaturation + fade-out of particles and trails
- Time expired triggers timer wall shatter + red-orange vignette + desaturation
- Cell death context detection correctly escalates: single < combo < chain < evolution
- Single cell death: dissolve + spark burst
- Combo chain: shard scatter particles
- Chain reaction: detonation rings + screen pulse + dense particles
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL
