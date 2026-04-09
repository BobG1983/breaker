# Local Impact Trigger

## Trigger
- `Impacted(EntityKind)` — you were in a collision with the specified entity type

## Locality: LOCAL
Fires on **both collision participants**.

## Participant Enum
```rust
enum ImpactTarget { Impactor, Impactee }
```
- `Impactor` = the entity that initiated the collision (bolt hitting cell, bolt hitting wall, etc.)
- `Impactee` = the entity that was hit

## Source Messages
| Collision | Impactor | Impactee | Message |
|-----------|----------|----------|---------|
| Bolt → Cell | Bolt | Cell | `BoltImpactCell` |
| Bolt → Wall | Bolt | Wall | `BoltImpactWall` |
| Bolt → Breaker | Bolt | Breaker | `BoltImpactBreaker` (from bump system) |
| Cell → Wall | Cell | Wall | `CellImpactWall` |

## Bridge System Behavior
One bridge per collision type. Each reads its collision message and fires `Impacted` on both participants:

```
fn bridge_bolt_impacted_cell(impacts: MessageReader<BoltImpactCell>, ...) {
    for msg in impacts.read() {
        let context = TriggerContext::Impact(ImpactContext {
            impactor: msg.bolt,
            impactee: msg.cell,
            depth: 0,
        });
        
        // Fire Impacted(Cell) on the bolt (bolt impacted a cell)
        walk_effects(&Trigger::Impacted(EntityKind::Cell), &context, msg.bolt, ...);
        
        // Fire Impacted(Bolt) on the cell (cell was impacted by a bolt)
        walk_effects(&Trigger::Impacted(EntityKind::Bolt), &context, msg.cell, ...);
    }
}
```

## Trigger Matching
The `EntityKind` parameter in the trigger matches the **other** entity type:
- `Impacted(Cell)` fires on the bolt when bolt hits cell
- `Impacted(Bolt)` fires on the cell when bolt hits cell
- `Impacted(Wall)` fires on the bolt when bolt hits wall
- `Impacted(Breaker)` fires on the bolt when bolt hits breaker

## Notes
- The `Impacted` trigger parameter names the OTHER entity — "I was impacted by a Cell"
- `On(ImpactTarget::Impactee, ...)` redirects to the impactee entity (e.g., the cell that was hit)
- `On(ImpactTarget::Impactor, ...)` redirects to the impactor entity (e.g., the bolt that hit)
- Key use case: `Route(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Transfer(...))))` — powder keg
