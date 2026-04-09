# TetherBeam

## Config
```rust
struct TetherBeamConfig {
    /// Damage multiplier for beam contact
    damage_mult: f32,
    /// If true, chain mode connects all existing bolts instead of spawning new ones
    chain: bool,
}
```
**RON**: `TetherBeam(damage_mult: 0.5, chain: false)`

## Reversible: NO (spawns entities — no-op reverse)

## Target: Bolt

## Fire
- If `chain: false`: spawn two new bolts connected by a tether, with a damaging beam between them
- If `chain: true`: connect existing bolts with damaging beams (no new bolts spawned)

## Runtime System
Beam damage system runs each FixedUpdate tick:
1. For each tether beam, check cells intersecting the line segment between the two connected bolts
2. Send `DamageDealt<Cell>` for cells in contact

## Messages Sent
- `DamageDealt<Cell> { dealer: Some(source_bolt), target: cell, amount: damage, source_chip }`

## Notes
- Two free-moving bolts connected by a visible damaging beam
- Chain mode reuses existing bolts instead of spawning new ones
- Beam damages cells that pass through the line segment between connected bolts
- Damage applies each tick (not at-most-once like shockwave)
