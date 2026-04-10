# Types

No new types needed for impact bridges.

## Existing types consumed

### Collision messages (all in their respective domain `messages.rs`)
- `BoltImpactCell { cell: Entity, bolt: Entity }` — bolt hit a cell. Source: `bolt/messages.rs`.
- `BoltImpactWall { bolt: Entity, wall: Entity }` — bolt hit a wall. Source: `bolt/messages.rs`.
- `BoltImpactBreaker { bolt: Entity, breaker: Entity }` — bolt hit the breaker. Source: `bolt/messages.rs`.
- `BreakerImpactCell { breaker: Entity, cell: Entity }` — breaker hit a cell. Source: `breaker/messages.rs`.
- `BreakerImpactWall { breaker: Entity, wall: Entity }` — breaker hit a wall. Source: `breaker/messages.rs`.
- `CellImpactWall { cell: Entity, wall: Entity }` — cell hit a wall. Source: `cells/messages.rs`.

### Effect types
- `ImpactTarget { Cell, Bolt, Wall, Breaker }` — entity kind involved in a collision. Source: `effect/core/types/definitions/enums/types.rs`.
- `Trigger::Impact(ImpactTarget)` — global trigger variant.
- `Trigger::Impacted(ImpactTarget)` — local trigger variant.

## Collision matrix

| Message | Participant A (kind) | Participant B (kind) |
|---------|---------------------|---------------------|
| BoltImpactCell | bolt (Bolt) | cell (Cell) |
| BoltImpactWall | bolt (Bolt) | wall (Wall) |
| BoltImpactBreaker | bolt (Bolt) | breaker (Breaker) |
| BreakerImpactCell | breaker (Breaker) | cell (Cell) |
| BreakerImpactWall | breaker (Breaker) | wall (Wall) |
| CellImpactWall | cell (Cell) | wall (Wall) |
