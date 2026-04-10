# Name
Dead

# Syntax
```rust
#[derive(Component)]
struct Dead;
```

# Description
Marker component inserted on an entity that has been confirmed dead by its domain kill handler. Prevents double-processing — systems use `Without<Dead>` to skip entities that are already dying.

Inserted by:
- Domain kill handlers when they process `KillYourself<T>` and decide the entity should die. The kill handler is the decision point — it can reject a kill request (e.g., invulnerability, second wind) by not inserting `Dead` and not sending `Destroyed<T>`.

Read by:
- `detect_*_deaths` systems via `Without<Dead>` filter — prevents re-sending KillYourself for an entity that is already dead
- `apply_damage::<T>` systems via `Without<Dead>` filter — no point applying damage to a dead entity

The entity still exists in the world after `Dead` is inserted. It survives through trigger evaluation and death bridges. It is finally despawned by `process_despawn_requests` in PostFixedUpdate.

# Location
`src/shared/components/dead.rs`
