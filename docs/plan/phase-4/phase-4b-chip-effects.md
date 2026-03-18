# Phase 4b: Chip Effect System

**Goal**: Chips do things. ChipSelected message -> gameplay effect applied to bolt/breaker entities.

**Wave**: 1 (foundation, no dependencies) — parallel with 4a

## What Exists

- `ChipDefinition` with name/kind/description
- `ChipRegistry` loaded from RON
- `ChipSelected` message sent by chip select screen
- 3 placeholder RON files (piercing.amp.ron, wide_breaker.augment.ron, surge.overclock.ron)

## Sub-Stages

This stage is too large for a single implementation pass. It touches the chips domain (types + handler) and 4 other domains (effect consumption). Split into two sub-stages that run in separate sessions.

### 4b.1: Effect Types, Handler, and Stacking (Session 1)

**Domain**: chips/

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

**What to build**:
- AmpEffect and AugmentEffect enums with Deserialize
- Effect application system: listen for ChipSelected, look up in registry, apply as component
- Stacking: if component exists, increment value (flat per-stack addition)
- Updated RON format with rarity, max_stacks, effect fields

**RON format**:
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

**Delegatable**: Yes — writer-tests → writer-code, scoped to chips/ domain.

### 4b.2: Per-Domain Effect Consumption (Session 2)

**Domains**: physics/, cells/, bolt/, breaker/

Modify existing systems to check for chip effect components. Each domain modification is independent and can parallelize:

| Effect | Domain | System to Modify | Change |
|--------|--------|-------------------|--------|
| Piercing | physics/ | bolt_cell_collision | Skip despawn for first N cells when `Piercing(n)` present |
| DamageBoost | cells/ | handle_cell_hit | Apply extra damage when `DamageBoost(f32)` present on bolt |
| SpeedBoost (bolt) | bolt/ | prepare_bolt_velocity | Increase speed when `BoltSpeedBoost(f32)` present |
| SizeBoost | bolt/ | init_bolt_params or spawn | Increase radius when `SizeBoost(f32)` present |
| WidthBoost | breaker/ | init_breaker_params or spawn | Increase width when `WidthBoost(f32)` present |
| SpeedBoost (breaker) | breaker/ | move_breaker | Increase max speed when `BreakerSpeedBoost(f32)` present |
| BumpStrength | breaker/ | grade_bump | Modify multiplier when `BumpStrength(f32)` present |
| DashDistance | breaker/ | update_breaker_state | Modify dash distance when `DashDistance(f32)` present |

**Delegatable**: Yes — one writer-tests → writer-code pair per domain (up to 4 parallel).

### Hot-Reload Propagation

- When a chip RON file changes: rebuild the chip registry
- If any active chips were modified: re-apply their effects to live entities
- Extend the `HotReloadPlugin` chain established in Phase 3c

Hot-reload can be implemented during either sub-stage or as a follow-up within Session 2.

## Scenario Coverage

### New Invariants
- **`ChipStacksInRange`** — chip stack counts never exceed `max_stacks` for any chip. Checked every frame during `Playing`.
- **`NoNaN`** — existing, but must now also cover new effect component values (bolt radius after SizeBoost, breaker width after WidthBoost).

### New Scenarios
- `mechanic/piercing_chaos.scenario.ron` — Chaos input with Piercing effect pre-applied. Verifies bolt doesn't get stuck, bounds are respected, and cells are destroyed. Uses a layout dense enough to trigger frequent piercing.
- `mechanic/wide_breaker_bounds.scenario.ron` — WidthBoost applied at max stacks. Verifies breaker still fits within playfield bounds (`BreakerInBounds`, `BreakerPositionClamped`).
- `stress/chip_stacking_stress.scenario.ron` — Scripted input that rapidly applies multiple chip effects. Verifies `NoEntityLeaks`, `NoNaN`, `ChipStacksInRange`.

### Existing Scenario Updates
- All existing stress scenarios should still pass with chip effect components present on entities (even if no chips are active — zero-value defaults must be safe).

## Acceptance Criteria

1. Selecting "Piercing Shot" adds a `Piercing(1)` component to the bolt
2. Selecting it again increments to `Piercing(2)`
3. Bolt actually pierces through cells when `Piercing` component is present
4. Wide Breaker visually widens the breaker
5. Hot-reload: changing `Piercing(1)` to `Piercing(2)` in RON updates live entities
6. Chip definitions with rarity and max_stacks parse from RON
