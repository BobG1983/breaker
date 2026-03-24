---
name: collision-message-flow
description: Physics collision message types, fields, system chain, ordering, and consumers for bolt-cell/bolt-breaker/bolt-wall interactions
type: reference
---

## Collision Messages (all FixedUpdate, run_if PlayingState::Active)

### Physics sends (bolt_cell_collision, bolt_breaker_collision)
- `BoltHitCell { cell: Entity, bolt: Entity }` -- every cell contact (not just destruction)
- `BoltHitBreaker { bolt: Entity }` -- every breaker contact
- `BoltHitWall { bolt: Entity }` -- every wall reflection
- `DamageCell { cell: Entity, damage: f32, source_bolt: Entity, source_chip: Option<String> }` -- paired with each BoltHitCell (source_chip always None from physics; may be Some from shockwave)
- `BoltLost` -- bolt falls below playfield

### Cells sends (handle_cell_hit)
- `CellDestroyed { was_required_to_clear: bool }` -- only when cell HP reaches 0

### Breaker sends (grade_bump / update_bump)
- `BumpPerformed { grade: BumpGrade, bolt: Entity }` -- graded bump on breaker contact
- `BumpWhiffed` -- forward window expired without hit

## Ordering Chain
```
BoltSystems::PrepareVelocity
  -> bolt_cell_collision (sends BoltHitCell, DamageCell, BoltHitWall)
    -> bolt_breaker_collision (sends BoltHitBreaker) [BoltSystems::BreakerCollision]
      -> clamp_bolt_to_playfield
        -> bolt_lost (sends BoltLost) [BoltSystems::BoltLost]
```

handle_cell_hit (cells) has NO ordering relative to physics -- reads DamageCell, writes CellDestroyed.

grade_bump (breaker) runs .after(BoltSystems::BreakerCollision) -- reads BoltHitBreaker, writes BumpPerformed.

## Multi-bounce
bolt_cell_collision uses CCD loop (MAX_BOUNCES=4). One bolt can hit up to 4 cells/walls per frame, each producing its own BoltHitCell + DamageCell.

## Consumer Map
| Message | Consumers |
|---------|-----------|
| BoltHitCell | bridge_cell_impact |
| BoltHitBreaker | grade_bump, bridge_breaker_impact |
| BoltHitWall | bridge_wall_impact |
| DamageCell | handle_cell_hit |
| CellDestroyed | track_cells_destroyed, bridge_cell_destroyed, track_node_completion |
| BoltLost | bridge_bolt_lost |
| BumpPerformed | bridge_bump, track_bumps, perfect_bump_dash_cancel |
