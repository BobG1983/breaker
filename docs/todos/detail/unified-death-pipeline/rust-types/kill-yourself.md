# Name
KillYourself\<T\>

# Syntax
```rust
#[derive(Message, Clone, Debug)]
struct KillYourself<T: GameEntity> {
    victim: Entity,
    killer: Option<Entity>,
    _marker: PhantomData<T>,
}
```

# Description
Sent when an entity should die. Replaces `RequestCellDestroyed` and `RequestBoltDestroyed`.

- victim: The entity to kill.
- killer: The entity that caused the death. Read from KilledBy.dealer by the death detection system. None for environmental deaths.

Sent by: death detection systems (`detect_cell_deaths`, `detect_bolt_deaths`, etc.) when Hp ≤ 0. Also sent directly by `Fire(Die)` in the effect system.

Consumed by: per-domain kill handlers that perform domain-specific death logic (check invulnerability, shields, etc.) before confirming the kill via `Destroyed<T>`.

DO NOT despawn the entity when sending KillYourself. The entity must stay alive through domain handling, trigger evaluation, and death animation.
