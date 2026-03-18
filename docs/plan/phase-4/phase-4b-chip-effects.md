# Phase 4b: Chip Effect System

**Goal**: Chips do things. ChipSelected message -> gameplay effect applied to bolt/breaker entities.

## What Exists

- `ChipDefinition` with name/kind/description
- `ChipRegistry` loaded from RON
- `ChipSelected` message sent by chip select screen
- 3 placeholder RON files (piercing.amp.ron, wide_breaker.augment.ron, surge.overclock.ron)

## What to Build

### Chip Effect Types

Extend `ChipDefinition` with effect data. Component markers with values:

```rust
// Amp effects — applied as components on the bolt entity
enum AmpEffect {
    Piercing(u32),      // pierce through N cells
    DamageBoost(f32),   // extra damage per hit
    SpeedBoost(f32),    // bolt speed increase
    Ricochet(u32),      // extra bounces off walls
    SizeBoost(f32),     // bolt radius increase
}

// Augment effects — applied as components on the breaker entity
enum AugmentEffect {
    WidthBoost(f32),    // breaker width increase
    SpeedBoost(f32),    // breaker movement speed increase
    BumpStrength(f32),  // bump multiplier increase
    DashDistance(f32),   // dash distance increase
}
```

### Effect Application System

- Listen for `ChipSelected` message
- Look up the chip in the registry
- Apply the effect as a component on the appropriate entity (bolt for Amps, breaker for Augments)
- Stacking: if the component already exists, increment its value (flat per-stack addition)

### Effect Consumption Systems

Modify existing systems to check for chip effect components:
- **Piercing**: `bolt_cell_collision` checks for `Piercing(n)` — if present, skip despawn for first N cells
- **Wide Breaker**: breaker spawn/update checks for `WidthBoost(f32)` — modifies breaker width
- Other effects: stub systems that read the component but may not have full gameplay impact yet

### Hot-Reload Propagation

- When a chip RON file changes: rebuild the chip registry
- If any active chips were modified: re-apply their effects to live entities
- Extend the `HotReloadPlugin` chain established in Phase 3c

### RON Format Update

```ron
// assets/amps/piercing.amp.ron
(
    name: "Piercing Shot",
    kind: Amp,
    description: "Bolt passes through the first cell it hits",
    rarity: Common,
    max_stacks: 3,
    effect: Piercing(1),
)
```

## Acceptance Criteria

1. Selecting "Piercing Shot" adds a `Piercing(1)` component to the bolt
2. Selecting it again increments to `Piercing(2)`
3. Bolt actually pierces through cells when `Piercing` component is present
4. Wide Breaker visually widens the breaker
5. Hot-reload: changing `Piercing(1)` to `Piercing(2)` in RON updates live entities
6. Chip definitions with rarity and max_stacks parse from RON
