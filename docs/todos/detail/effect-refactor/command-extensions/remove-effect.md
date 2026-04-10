# Name
remove_effect

# Struct
```rust
struct RemoveEffectCommand {
    entity: Entity,
    route_type: RouteType,
    source: String,
    tree: Tree,
}
```

# Description
Remove an effect tree entry from an entity's BoundEffects or StagedEffects.

If `route_type` is Bound, find and remove the first entry in BoundEffects matching `(source, tree)`. This is used by Once nodes after they fire — Once consumes itself from BoundEffects.

If `route_type` is Staged, find and remove the first entry in StagedEffects matching `(source, tree)`. This is used by the walking algorithm to consume matched staged entries.

Matching is by (source, tree) equality. If no matching entry is found, do nothing.

If the entity does not exist in the world, do nothing.

This command is the counterpart to `route_effect` — route installs, remove removes.
