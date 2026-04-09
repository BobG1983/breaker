# Explode

## Config
```rust
struct ExplodeConfig {
    /// Blast radius in world units
    range: f32,
    /// Flat damage dealt to each cell in range
    damage: f32,
}
```
**RON**: `Explode(range: 48.0, damage: 10.0)`

## Reversible: NO (instant damage — no-op reverse)

## Target: Any entity with position (typically bolt or cell)

## Fire
1. Read source entity's position
2. Spawn explosion entity directly at that position with range, damage, source_chip
3. Runtime system processes the explosion immediately (same tick)

## Reverse
No-op.

## Spawned Entity Components
```rust
ExplodeMarker           // marker for the runtime system
ExplodeConfig { range, damage }
EffectSourceChip
Position2D              // at source entity's position
CleanupOnExit<NodeState>
```

## Runtime System: `process_explosions`
**Schedule**: FixedUpdate, after PhysicsSystems::MaintainQuadtree, run_if NodeState::Playing

1. For each explosion entity:
   - Query quadtree for cells within range (circle query, CELL_LAYER)
   - Send `DamageDealt<Cell>` for each cell found (flat damage, not multiplied by anything)
   - Spawn visual flash entity (circle mesh, red-orange HDR, 0.2s duration)
   - Despawn the explosion entity

## Messages Sent
- `DamageDealt<Cell> { dealer: Some(source_bolt), target: cell, amount: damage, source_chip }`

## Notes
- Unlike Shockwave, Explode is **instant** — all cells in range take damage at once, no expansion
- Damage is flat (not modified by damage boosts) — the `damage` field IS the final damage
- Flash visual is a separate entity with `EffectFlashTimer` for auto-despawn
- Key use case: powder keg (`When(Died, Fire(Explode(...)))` transferred onto a cell)
