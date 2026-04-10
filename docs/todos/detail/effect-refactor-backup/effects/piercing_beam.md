# PiercingBeam

## Config
```rust
struct PiercingBeamConfig {
    /// Damage multiplier applied to base damage
    damage_mult: f32,
    /// Beam width in world units
    width: f32,
}
```
**RON**: `PiercingBeam(damage_mult: 0.5, width: 16.0)`

## Reversible: NO (spawns entity — no-op reverse)

## Target: Bolt (fires beam in bolt's velocity direction)

## Fire
1. Read source entity's position, velocity direction, damage multiplier, base damage
2. Spawn beam request entity at source position
3. `process_piercing_beam` system processes the request: cast a rectangle along velocity direction, damage all cells intersecting the beam

## Reverse
No-op.

## Messages Sent
- `DamageDealt<Cell> { dealer: Some(source_bolt), target: cell, amount: damage, source_chip }` — for each cell intersecting the beam

## Notes
- Instant beam — fires once and damages all cells in the rectangle
- Beam extends from source position in velocity direction across the playfield
- Damage per cell = base_damage * damage_mult * effective_damage_multiplier
- Width determines the beam's cross-section
