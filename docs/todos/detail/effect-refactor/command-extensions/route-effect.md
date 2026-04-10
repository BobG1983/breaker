# Name
route_effect

# Struct
```rust
struct RouteEffectCommand {
    entity: Entity,
    source: String,
    tree: Tree,
    route_type: RouteType,
}
```

# Description
Install an effect tree on an entity, with the route type controlling where it goes.

If the entity does not have BoundEffects or StagedEffects, insert both (they are always inserted as a pair).

If route_type is Bound, append the tree to BoundEffects — it stays permanently and re-arms after each trigger match. This is what On(..., Route(Bound, tree)) produces at runtime.

If route_type is Staged, append the tree to StagedEffects — it fires once when its trigger matches, then is consumed and removed. This is what On(..., Route(Staged, tree)) produces at runtime.

The source string tags the entry for later identification and removal.

If the entity does not exist in the world, do nothing.
