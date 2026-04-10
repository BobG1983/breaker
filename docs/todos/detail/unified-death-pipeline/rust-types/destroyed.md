# Name
Destroyed\<T\>

# Syntax
```rust
#[derive(Message, Clone, Debug)]
struct Destroyed<T: GameEntity> {
    victim: Entity,
    killer: Option<Entity>,
    victim_pos: Vec2,
    killer_pos: Option<Vec2>,
    _marker: PhantomData<T>,
}
```

# Description
Sent after the domain kill handler confirms the kill. Replaces `CellDestroyedAt`.

- victim: The entity that died. Still alive at this point — not yet despawned.
- killer: The entity that caused the death. None for environmental deaths.
- victim_pos: World position of the victim at time of death. Extracted while entity is still alive.
- killer_pos: World position of the killer, if it exists. Used for directional VFX.

Consumed by: `on_destroyed::<T>` which dispatches Died/Killed/DeathOccurred triggers to the effect system. Also consumed by VFX, audio, tracking systems.

The entity is still alive when Destroyed is sent. It survives through trigger evaluation and death animation. Despawn happens later via DespawnEntity in PostFixedUpdate.

DO NOT despawn the entity when sending Destroyed. The bridge needs to walk the entity's BoundEffects/StagedEffects.
