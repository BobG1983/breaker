# Name
KilledBy

# Syntax
```rust
#[derive(Component, Default, Debug)]
struct KilledBy {
    dealer: Option<Entity>,
}
```

# Description
Set by `apply_damage<T>` on the killing blow only — the damage message that crosses Hp from positive to zero. Read by death detection systems to determine who killed the entity.

- dealer: The entity that originated the damage. Some(entity) for attributed kills, None for environmental deaths (timer, lifespan, etc.).

The dealer is the originating entity, not the intermediate effect. A shockwave spawned by a bolt credits the bolt as dealer. An explosion from a powder keg credits the bolt that killed the cell that exploded.

DO set KilledBy only on the killing blow. If an entity takes 3 damage messages in one frame and the second one kills it, only the second sets KilledBy.
DO NOT overwrite KilledBy once set — first kill wins if multiple messages cross zero in the same frame.
DO insert KilledBy(default) on all entities that have Hp, so the component is always present for query access.
