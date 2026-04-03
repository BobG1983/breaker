# 5l: HUD & Gameplay UI

## Summary

Implement a diegetic HUD per DR-1 — all gameplay information integrated into the game world, no overlaid panels or dashboards. The node timer is a glowing gauge along the top wall. Lives are energy orbs near the breaker. Node progress is tick marks on a side wall. All numeric readouts use monospace Data typography. Remove any existing side panels or overlay HUD elements.

## Context

DR-1 resolved the HUD style as Diegetic/Integrated: all information lives in the game world. The timer is not a UI bar in a corner — it is a visible glow on the top wall that dims as time runs out. Life orbs are not icons in a panel — they are small glowing entities near the breaker that dissolve when a life is lost. Node progress ticks are not a text readout — they are marks on the wall that light up as nodes are cleared.

This approach means HUD elements are rendered with the same visual system as gameplay entities (EntityGlowMaterial, particles, modifiers). They participate in the visual identity: they glow, they respond to temperature palette shifts, and they can receive VFX treatments (timer glitch on time penalty, orb dissolve on life lost, etc.).

Key architecture change from the LEGACY plan: the old plan referenced `AttachVisuals` messages and recipes for HUD elements (e.g., life orb dissolve recipe). The new plan uses direct entity spawning with EntityGlowMaterial and direct VFX function calls. The old plan placed systems in `screen/playing/hud/`. The new plan places them in the appropriate state domain — likely `state/run/node/hud/` or wherever the active-gameplay HUD ends up after the state boundary refactor. The exact module path may shift, but the systems conceptually belong to the active node gameplay state.

## What to Build

### 1. Timer Wall Gauge

A visual gauge entity overlaid on the top wall that represents time remaining:

- **Entity**: Separate overlay entity positioned on top of the top wall. Uses a dedicated `timer_wall.wgsl` shader (or EntityGlowMaterial with custom uniform mapping) that renders a horizontal gauge.
- **Fill level**: `fill_level` uniform (0.0-1.0) drives the gauge. At 1.0: full bright glow across the entire top wall. At 0.0: completely dark. The gauge drains from right to left (or from edges inward — tune during implementation).
- **Color shift**: Gauge color shifts based on remaining time:
  - >50% remaining: cool color (from temperature palette)
  - 25-50%: transitioning toward amber
  - <25%: red-orange, pulsing with increasing frequency (urgency)
  - The `pulse_speed` parameter increases as time drops, creating an accelerating heartbeat feel
- **Numeric readout**: Small monospace `Text2d` child entity near the gauge center showing remaining seconds (Data typography — clean, stable, no jitter). This is the only text element; it supplements the gauge for precise readability.
- **Spawn/despawn**: System spawns the gauge on `OnEnter(PlayingState::Active)` (or equivalent active-node state). Despawns on exit.
- **Update**: Each `FixedUpdate`, system reads the node timer resource and updates `fill_level`, color, and `pulse_speed` uniforms on the gauge material.
- **Temperature integration**: Gauge base color responds to `RunTemperature` — cool runs have blue-tinted gauge, hot runs have amber-tinted gauge (before danger thresholds override).

### 2. Timer Wall Gauge Shader (timer_wall.wgsl)

A Material2d shader for the gauge overlay:

- Renders a horizontal bar with soft edges
- `fill_level` uniform controls how much of the bar glows (left-to-right fill)
- `color` uniform for base color
- `pulse_speed` uniform: sinusoidal brightness modulation at the specified frequency
- `danger_color` uniform: blended in as fill drops below danger threshold
- Transition edge: the boundary between filled and unfilled has a soft glow/feathered falloff, not a hard cutoff
- HDR values >1.0 on the filled portion trigger bloom

### 3. Life Orbs

N small energy orb entities representing remaining lives:

- **Entity**: Each orb is a small entity with `EntityGlowMaterial`, `Shape::Circle`, using the breaker's archetype color tint. Core brightness HDR >1.2 for subtle bloom.
- **Layout**: Orbs are evenly spaced horizontally at a fixed y-offset below the breaker's spawn/home position (bottom of playfield). Spacing calculated from orb count so they stay centered.
- **Tracking**: A system each `FixedUpdate` updates orbs' x-positions to follow the breaker's current x-position. Orbs drift with the breaker with slight smoothing (lerp toward target x each frame, not instant snap).
- **Life loss visual**: When a life is consumed, the rightmost remaining orb plays a dissolve effect:
  - Ramp `dissolve_threshold` on its EntityGlowMaterial from 0.0 to 1.0 over ~0.3s
  - Spawn `ParticleEmitter` with `Burst { count: 12 }` at the orb's position — sparks matching archetype color, radiating outward
  - Despawn the orb entity after dissolve completes
- **Life gain visual** (Second Wind evolution): A new orb materializes at the rightmost position:
  - Spawn the orb entity with `dissolve_threshold` at 1.0, then ramp down to 0.0 over ~0.3s (reverse dissolve — orb fades in)
  - Glow mote particles drift inward toward the orb during materialization: `ParticleEmitter` with `Burst { count: 8 }`, particles with gravity toward orb center
  - Brief glow pulse: `AddModifier` with `GlowIntensity(2.0)` for 0.15s on the new orb
- **Spawn/despawn**: System spawns orbs on `OnEnter` active node state, reading initial lives count. Despawns all on exit.

### 4. Node Progress Ticks

Tick marks on one side wall showing progress within the current run section:

- **Entity**: N small rectangular entities (`EntityGlowMaterial`, `Shape::Rectangle`) positioned at evenly spaced points along one side wall (e.g., right wall). N = nodes remaining in the current section/act (resets each boss clear or section transition for infinite run scaling).
- **Visual states**:
  - Completed node: bright glow, archetype color, HDR >1.2
  - Current node: brightest glow, pulsing slowly, HDR >1.5
  - Upcoming node: very dim outline, barely visible (HDR ~0.3)
- **Update**: When a node is cleared, the corresponding tick transitions from "current" to "completed" (brief flash + settle to bright), and the next tick transitions from "upcoming" to "current" (pulse begins).
- **Section transition**: When the player enters a new section (e.g., after a boss), all ticks despawn and new ticks spawn for the new section's node count. Brief `ParticleEmitter` burst at each tick position during the section transition for visual feedback.
- **Infinite run scaling**: The tick system always shows progress within the current section, not total run progress. This keeps the tick count finite and readable regardless of how long the run goes.
- **Spawn/despawn**: System spawns ticks on `OnEnter` active node state, reading run state for section info. Despawns on exit.

### 5. Side Panel Removal

Remove any existing overlay HUD elements:
- Remove "AUGMENTS" left panel (if it exists)
- Remove status right panel (if it exists)
- Remove any Bevy UI Node-based HUD overlays for gameplay information
- The playfield takes the full viewport width with thin wall borders (from 5i wall visuals)

Search for existing HUD-related systems, components, and UI node spawning code in the playing state modules.

### 6. Monospace Font Asset

Add a monospace font asset for:
- Timer numeric readout (Data typography — clean, stable, instantly readable)
- Run seed display (on run-end screen, 5p)
- Any other numeric data that must be instantly readable

The font should be a clean, geometric monospace face. Load it in the asset loading pipeline and make it available as a shared resource or via the font asset system.

### 7. HUD Interaction with Other Systems

The HUD elements are game-world entities, so they naturally interact with:
- **Temperature palette** (5j): orb and tick colors shift with `RunTemperature`
- **Danger vignette** (5j): vignette darkens screen edges, which affects perceived brightness of wall-integrated elements — gauge and ticks should have minimum brightness to remain readable through vignette
- **Time penalty VFX** (5l): time penalty makes the timer gauge glitch briefly — the time_penalty_vfx function sends `AddModifier` to the gauge entity
- **Highlight moments** (5m): Nail Biter highlight pulses the timer gauge entity
- **Slow-mo** (5j): during slow-mo, the timer gauge update rate slows with `Time<Virtual>` — the fill level visually pauses during the dramatic moment

## What NOT to Do

- Do NOT implement the HUD as Bevy UI nodes (Node, Button, etc.) — all HUD elements are world-space entities using EntityGlowMaterial
- Do NOT implement chip build display or active-chips indicator — that complexity is deferred. The chip card system (5o) and screens (5p) handle chip-related UI.
- Do NOT implement the run-end screen stats display — that is 5p (screens)
- Do NOT implement chip select timer pressure visualization — that is 5o (chip cards)
- Do NOT create a HUD overlay panel or dashboard — per DR-1, all information is diegetic

## Dependencies

- **Requires**: 5e (visuals domain — EntityGlowMaterial, Shape, Hue, GlowParams, modifier messages, PunchScale for orb materialization), 5i (wall visuals — wall entity positions for gauge and tick placement), 5j (modifier computation — for dynamic HUD element effects like glow pulses; temperature palette — for color shifts)
- **Enhanced by**: 5c (rantzsoft_particles2d — particle effects for orb dissolve/materialize, section transition bursts), 5k (bump/failure VFX — life lost triggers orb dissolve), 5l (combat VFX — time penalty triggers gauge glitch), 5m (highlights — Nail Biter pulses gauge)
- **Required by**: 5p (screens — screens reference HUD patterns for consistency)

## Verification

- Timer wall gauge visible along top wall, fills correctly from 1.0 to 0.0 as time passes
- Timer gauge color shifts: cool at >50%, amber transition at 25-50%, red-orange pulsing at <25%
- Timer pulse frequency increases as time drops (visible urgency escalation)
- Timer numeric readout visible in monospace, instantly readable, stable (no jitter)
- Life orbs visible below breaker, correct count matches remaining lives
- Life orbs track breaker x-position smoothly (not instant snap)
- Life orb dissolves on life loss: dissolve effect + spark burst + despawn
- Life orb materializes on life gain: reverse dissolve + inward particles + glow pulse
- Node progress ticks visible on side wall, correct count for current section
- Current node tick pulses, completed ticks are bright, upcoming ticks are dim
- Tick transitions are visually clear (flash on clear, pulse begins on next)
- Section transition respawns ticks with particle bursts
- No overlay panels or dashboards remain
- HUD elements respond to temperature palette shifts
- HUD remains readable through danger vignette (minimum brightness maintained)
- Monospace font loads correctly and renders Data typography
- Timer gauge is readable in peripheral vision during gameplay
- Life orbs are glanceable without looking away from the bolt
- All existing tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status: NEEDS DETAIL
