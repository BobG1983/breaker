# Local Bump Triggers

## Triggers
- `PerfectBumped` — perfect timing bump
- `EarlyBumped` — early timing bump
- `LateBumped` — late timing bump
- `Bumped` — any successful bump (perfect, early, or late)

## Locality: LOCAL
Fires on **both bolt AND breaker** participant entities. (Breaking change: current system fires only on bolt.)

## Participant Enum
```rust
enum BumpTarget { Bolt, Breaker }
```
Used by `On(BumpTarget::Bolt, ...)` or `On(BumpTarget::Breaker, ...)` to redirect effects to a specific participant.

## Source Message
`BumpGraded` from bolt domain — contains bolt entity, breaker entity, bump grade.

## Bridge System Behavior
```
fn bridge_<grade>_bumped(bump_events: MessageReader<BumpGraded>, ...) {
    for event in bump_events.read() {
        if event.grade != <expected_grade> { continue; }  // skip for specific grades
        // For Bumped: accept Perfect, Early, Late (any success)
        
        let context = TriggerContext::Bump(BumpContext {
            bolt: event.bolt,
            breaker: event.breaker,
            source: event.source.clone(),
            depth: 0,
        });
        
        // Walk BOTH participant entities
        for entity in [event.bolt, event.breaker] {
            if let Ok((mut bound, mut staged)) = query.get_mut(entity) {
                walk_effects(&Trigger::<Grade>Bumped, &context, entity, &mut bound, &mut staged, world);
            }
        }
    }
}
```

## Grade Matching
| Trigger | Fires when grade is |
|---------|-------------------|
| PerfectBumped | Perfect |
| EarlyBumped | Early |
| LateBumped | Late |
| Bumped | Perfect, Early, or Late |

## TriggerContext
```rust
BumpContext {
    bolt: Entity,     // the bolt entity
    breaker: Entity,  // the breaker entity
    source: SourceId, // chip attribution
    depth: u32,       // dispatch recursion depth
}
```

## Notes
- Each grade has its own bridge system (4 systems total: perfect_bumped, early_bumped, late_bumped, bumped)
- `Bumped` is a convenience trigger that matches any success grade — fires in addition to the specific grade trigger
- Both entities are walked — effects routed to Bolt fire on bolt, effects routed to Breaker fire on breaker
- `On(BumpTarget::Bolt, Fire(...))` from a breaker-routed effect redirects the fire to the bolt participant
