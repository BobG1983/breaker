# Name
stamp_effect

# Struct
```rust
struct StampEffectCommand {
    entity: Entity,
    source: String,
    tree: Tree,
}
```

# Description
Permanently install an effect tree on an entity's BoundEffects. Sugar for `route_effect(entity, source, tree, RouteType::Bound)`.

If the entity does not have BoundEffects or StagedEffects, insert both (they are always inserted as a pair). Then append the tree to BoundEffects, tagged with the source string so it can be identified later for removal.

This is the runtime equivalent of a definition-level `Stamp(target, tree)` — the difference is that `stamp_effect` targets a specific entity, while the RON `Stamp` targets a StampTarget that gets resolved to entities.

stamp_effect always appends. It does not check for existing entries with the same source. Calling stamp_effect twice with the same source produces duplicate entries. Callers are responsible for not double-stamping.

If the entity does not exist in the world, do nothing.
