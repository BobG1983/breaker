# Name
fire_effect

# Struct
```rust
struct FireEffectCommand {
    entity: Entity,
    effect: EffectType,
    source: String,
}
```

# Description
Execute an effect immediately on an entity.

Look up the entity in the world. Apply the effect to it right now — modify components, spawn child entities, send messages, whatever the effect does. Nothing is stored in BoundEffects or StagedEffects. The effect happens once and is done.

The source string identifies which chip or definition caused the effect, used for tracking and debugging.

If the entity does not exist in the world, do nothing.
