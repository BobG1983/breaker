# Global Bump Triggers

## Triggers
- `PerfectBumpOccurred` — a perfect bump happened somewhere
- `EarlyBumpOccurred` — an early bump happened somewhere
- `LateBumpOccurred` — a late bump happened somewhere
- `BumpOccurred` — any successful bump happened somewhere
- `BumpWhiffOccurred` — a bump attempt missed the timing window
- `NoBumpOccurred` — bolt hit breaker with no bump input

## Locality: GLOBAL
Fires on **ALL entities** with BoundEffects/StagedEffects.

## Participant Enum
```rust
enum BumpTarget { Bolt, Breaker }
```
Same as local bump triggers. `On(BumpTarget::Bolt, ...)` resolves to the bolt from the event.

## Source Message
`BumpGraded` from bolt domain.

## Bridge System Behavior
```
fn bridge_<grade>_bump_occurred(bump_events: MessageReader<BumpGraded>, ...) {
    for event in bump_events.read() {
        if event.grade != <expected_grade> { continue; }
        
        let context = TriggerContext::Bump(BumpContext {
            bolt: event.bolt,
            breaker: event.breaker,
            depth: 0,
        });
        
        // Walk ALL entities with effects
        for (entity, mut bound, mut staged) in &mut all_effects_query {
            walk_effects(&Trigger::<Grade>BumpOccurred, &context, entity, &mut bound, &mut staged, world);
        }
    }
}
```

## Grade Matching
| Trigger | Fires when grade is |
|---------|-------------------|
| PerfectBumpOccurred | Perfect |
| EarlyBumpOccurred | Early |
| LateBumpOccurred | Late |
| BumpOccurred | Perfect, Early, or Late |
| BumpWhiffOccurred | Whiff |
| NoBumpOccurred | NoBump |

## Notes
- 6 global bump triggers (one per grade + Whiff + NoBump)
- BumpWhiff and NoBump have no local counterparts (they don't target specific entities)
- Global triggers walk every entity with effects, not just participants
- Useful for "whenever any perfect bump happens, boost all bolts" type effects
