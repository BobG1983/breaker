# 5n: Visual Modifier System

**Goal**: Implement the system that modifies bolt and breaker appearance based on active chip effects, with diminishing visual returns on stacking.

## What to Build

### 1. Visual Modifier Application System

A rendering/ system that reads active chip effects on bolt/breaker and applies visual modifications:
- Reads `ActiveChipEffects` (or equivalent) from gameplay components
- Maps each active effect to its `VisualModifier` (defined in 5f)
- Applies the combined modifier to the entity's rendering

### 2. Diminishing Returns Stacking

Each modifier type stacks with diminishing visual returns:
- 1st stack: full multiplier (e.g., 1.5x trail length)
- 2nd stack: reduced multiplier (e.g., 1.35x)
- 3rd stack: further reduced (e.g., 1.2x)
- 4th+ stacks: minimal (e.g., 1.1x)

Exact curve tunable per modifier type.

**IMPORTANT**: Diminishing returns apply ONLY to visual modifiers, NOT to gameplay effects. A bolt with 5 speed stacks gets the full gameplay speed multiplier — it just doesn't render with a 7.5x trail length.

### 3. Bolt Visual Modifiers

| Effect | Visual Change |
|--------|--------------|
| Speed Boost | Trail length multiplier, glow intensity up, halo color warmer |
| Damage Boost | Core brightness up, color shift toward amber/white |
| Piercing | Angular glow, energy spikes (already in 5g as state, here as modifier strength) |
| Size Boost | Glow scales with size (already partially in 5g, here with proper scaling) |

Different modifier types stack independently — speed + damage = longer trail AND color shift.

### 4. Breaker Visual Modifiers

| Effect | Visual Change |
|--------|--------------|
| Speed Boost | Aura stretches in movement direction, trailing wisps, dash trail intensity (built in 5h, driven by modifier here) |
| Width Boost | Aura pulse on activation, stretch animation (built in 5h, driven by modifier here) |
| Bump Force | Front face glow, pulsing (built in 5h, driven by modifier here) |

### 5. Modifier-Driven Rendering

rendering/ systems that consume the computed modifier values:
- Bolt shader reads `ComputedBoltModifiers { trail_length_mult, glow_intensity_mult, color_shift, particle_emitter, ... }`
- Breaker shader reads `ComputedBreakerModifiers { ... }`
- These are computed components maintained by the modifier system, not raw chip data

### 6. Visual Modifier Debug

Debug menu:
- Display active modifiers and their stacked values
- Override modifier values for visual tuning
- Toggle diminishing returns on/off (for comparison)

## What NOT to Do

- Do NOT implement new visual effects — this step composes existing effects from 5g-5h using modifier data
- Do NOT change gameplay multipliers — visual-only concern

## Dependencies

- **Requires**: 5g (bolt visuals: trail, glow, state rendering), 5h (breaker visuals: aura, trail, state rendering), 5f (VisualModifier types)
- **Enhanced by**: 5m (combat effects may also have visual modifiers)

## Catalog Elements Addressed

From `catalog/effects.md` (Visual Modifiers):
- Speed Boost (bolt): NONE → modifier-driven
- Damage Boost (bolt): NONE → modifier-driven
- Piercing (bolt state): NONE → modifier-driven strength
- Size Boost (bolt): PARTIAL → complete
- Size Boost (breaker): PARTIAL → complete
- Visual Modifier System: NONE → implemented with diminishing returns

## Verification

- Bolt with 1 speed stack looks visibly different from bolt with 3 stacks
- Diminishing returns prevent extreme visual scaling
- Multiple modifier types combine correctly (speed + damage = trail + color shift)
- Gameplay multipliers are unaffected by visual diminishing returns
- Debug menu shows modifier values
- All existing tests pass
