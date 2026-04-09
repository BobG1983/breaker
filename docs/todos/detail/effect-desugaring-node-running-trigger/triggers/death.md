# Death Triggers

## Triggers
- `Died` — this entity died (LOCAL, fires on victim only)
- `Killed(KillTarget)` — this entity killed something (LOCAL, fires on killer only) **NEW**
- `DeathOccurred(DeathTarget)` — something died somewhere (GLOBAL, fires on all entities)

## KillTarget / DeathTarget
```rust
enum KillTarget { Cell, Bolt, Wall, Breaker, Any }
enum DeathTarget { Cell, Bolt, Wall, Breaker, Any }
```

## Participant Enum
```rust
enum DeathTarget { Victim, Killer }
```
- `On(DeathTarget::Victim, ...)` resolves to the entity that died
- `On(DeathTarget::Killer, ...)` resolves to the entity that killed it (if present)

## Source Message
`Destroyed<T: GameEntity>` — generic on victim type (T). Killer is `Option<Entity>`, type determined at runtime. Sent by domain kill handlers after processing `KillYourself<T: GameEntity>`.

## Bridge System: `bridge_destroyed<T: GameEntity>`
```
fn bridge_destroyed<T: GameEntity>(destroyed: MessageReader<Destroyed<T: GameEntity>>, ...) {
    for msg in destroyed.read() {
        let context = TriggerContext::Death(DeathContext {
            victim: msg.victim,
            killer: msg.killer,  // Option<Entity>
            source: ...,
            depth: 0,
        });
        
        // Died — fires on VICTIM only
        walk_effects(&Trigger::Died, &context, msg.victim, ...);
        
        // Killed(target) — fires on KILLER only (skip if killer is None)
        if let Some(killer) = msg.killer {
            // Verify killer still alive before firing
            if world.get_entity(killer).is_ok() {
                let kill_target = T::kill_target();  // Cell, Bolt, Wall, Breaker
                walk_effects(&Trigger::Killed(kill_target), &context, killer, ...);
            }
        }
        
        // DeathOccurred(target) — fires on ALL entities
        let death_target = T::death_target();
        for (entity, mut bound, mut staged) in &mut all_query {
            walk_effects(&Trigger::DeathOccurred(death_target), &context, entity, ...);
        }
    }
}
```

## Kill Attribution Chain
```
DamageDealt<T> → apply_damage (sets KilledBy on killing blow)
→ detect_deaths (queries KilledBy + HP <= 0)
→ KillYourself<T: GameEntity> { victim, killer }
→ domain handler (invuln check, animation)
→ Destroyed<T: GameEntity> { victim, killer, positions }
→ bridge_destroyed (fires Died, Killed, DeathOccurred)
→ DespawnEntity (deferred to PostFixedUpdate)
```

## Notes
- `Killed(KillTarget)` is NEW — the current system has no "I killed X" trigger
- `Killed` does NOT fire when killer is `None` (environmental death — timer, lifespan expiry)
- `Died` always fires on the victim regardless of killer presence
- `DeathOccurred(Any)` matches any entity type death
- `Killed(Any)` matches killing any entity type
- Killer entity is verified alive before firing `Killed` — if despawned mid-chain, skip silently
