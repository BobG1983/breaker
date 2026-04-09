# BoltLost Trigger

## Trigger
- `BoltLostOccurred` — a bolt was lost (fell off the bottom)

## Locality: GLOBAL
Fires on **ALL entities** with BoundEffects/StagedEffects.

## Participant Enum
```rust
enum BoltLostTarget { Bolt, Breaker }
```

## Source Message
`BoltLost` from bolt domain — contains bolt entity and breaker entity.

## Bridge System Behavior
```
fn bridge_bolt_lost_occurred(events: MessageReader<BoltLost>, ...) {
    for msg in events.read() {
        let context = TriggerContext::BoltLost(BoltLostContext {
            bolt: msg.bolt,
            breaker: msg.breaker,
            depth: 0,
        });
        
        // Walk ALL entities with effects
        for (entity, mut bound, mut staged) in &mut all_query {
            walk_effects(&Trigger::BoltLostOccurred, &context, entity, ...);
        }
    }
}
```

## Notes
- Global only — no local `BoltLost` trigger (bolt is being despawned, can't fire effects on it)
- Key use case: `Route(Breaker, When(BoltLostOccurred, Fire(LoseLife)))` — lose a life when bolt lost
- `On(BoltLostTarget::Bolt, ...)` resolves to the lost bolt entity (but it's about to be despawned — use carefully)
- `On(BoltLostTarget::Breaker, ...)` resolves to the breaker entity
