# Name
stage_effect

# Struct
```rust
struct StageEffectCommand {
    entity: Entity,
    source: String,
    tree: Tree,
}
```

# Description
Install an effect tree into StagedEffects on an entity. Sugar for `route_effect(entity, source, tree, RouteType::Staged)`.

Used by the tree walker when arming nested trigger gates — see `walking-effects/arming-effects.md`.

If the entity does not exist in the world, do nothing.
