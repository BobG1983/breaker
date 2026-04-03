# 5n: Visual Modifier System

**Goal**: Implement the `ModifierStack` component and modifier computation system in `rantzsoft_vfx` that combines Set/Add modifiers with diminishing returns and updates shader uniforms.

Architecture: `docs/architecture/rendering/modifiers.md`

## What to Build

### 1. ModifierStack Component

Crate-owned component on each entity registered via `AttachVisuals`. Stores:
- Set entries: `HashMap<&'static str, VisualModifier>` — latest value per source key
- Add entries: `Vec<(&'static str, VisualModifier)>` — stacked with DR

### 2. Modifier Message Handlers

Systems that process the three message types:
- `SetModifier { entity, modifier, source }` — overwrites by source key
- `AddModifier { entity, modifier, source }` — adds to stack
- `RemoveModifier { entity, source }` — removes by source key

### 3. Diminishing Returns Computation

Per-modifier-type curves from `ModifierConfig` resource (game configures at startup):
- Numeric modifiers (TrailLength, GlowIntensity, etc.): values multiply, then DR curve applied
- Non-numeric modifiers (ColorShift, ColorCycle, AlphaOscillation, SquashStretch): latest wins, no DR
- `AfterimageTrail(bool)`: any true = enabled
- DR applies to `AddModifier` stacks only, NOT to `SetModifier`

### 4. Shader Uniform Update

Each frame (Update schedule, not FixedUpdate — for smooth interpolation):
- Combine Set values with DR-scaled Add values per modifier kind
- Update entity's material uniforms from computed results
- SquashStretch modifies SDF UV coordinates (shader uniform, not Transform)

### 5. ModifierConfig Resource

```rust
ModifierConfig {
    curves: HashMap<ModifierKind, Vec<f32>>,  // per-type DR curves
    default_curve: Vec<f32>,                  // fallback: [1.0, 0.7, 0.4, 0.2]
}
```

Game inserts at startup. Values beyond vec length use the last entry.

### 6. Debug Menu

- Display active modifiers per entity (modifier kind, source, value, DR-scaled value)
- Override modifier values for visual tuning
- Toggle diminishing returns on/off

## What NOT to Do

- Do NOT change gameplay multipliers — visual-only concern
- Do NOT implement new visual effects — this step implements the modifier computation, not the effects that send modifiers (those are 5g-5m)

## Dependencies

- **Requires**: 5f (VisualModifier types), 5g/5h (entity visuals that receive modifiers)
- **Enhanced by**: 5m (combat effects that send modifiers)

## Verification

- Entity with 1 speed stack looks visibly different from 3 stacks
- Diminishing returns prevent extreme visual scaling (3 stacks < 3x visual change)
- Multiple modifier types combine correctly (speed + damage = trail + color shift)
- Gameplay multipliers are unaffected by visual DR
- SetModifier overwrites by source key correctly
- RemoveModifier cleans up correctly
- All existing tests pass
