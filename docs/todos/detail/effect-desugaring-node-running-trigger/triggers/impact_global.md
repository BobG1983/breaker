# Global Impact Trigger

## Trigger
- `ImpactOccurred(ImpactTarget)` — a collision involving the specified entity type happened somewhere

## Locality: GLOBAL
Fires on **ALL entities** with BoundEffects/StagedEffects.

## Participant Enum
```rust
enum ImpactTarget { Impactor, Impactee }
```

## Source Messages
Same collision messages as local Impacted trigger.

## Bridge System Behavior
Same collision types, but walks all entities instead of just participants:

```
fn bridge_bolt_impact_cell_occurred(impacts: MessageReader<BoltImpactCell>, ...) {
    for msg in impacts.read() {
        let context = TriggerContext::Impact(ImpactContext {
            impactor: msg.bolt,
            impactee: msg.cell,
            source: msg.source.clone(),
            depth: 0,
        });
        
        // Walk ALL entities with effects
        for (entity, mut bound, mut staged) in &mut all_query {
            walk_effects(&Trigger::ImpactOccurred(ImpactTarget::Cell), &context, entity, ...);
        }
    }
}
```

## Notes
- Less commonly used than local `Impacted` — most impact effects are on the participants
- Useful for "whenever any bolt hits any cell, do X on the breaker" patterns
- `On(ImpactTarget::Impactor, ...)` still resolves to the bolt from the event context
