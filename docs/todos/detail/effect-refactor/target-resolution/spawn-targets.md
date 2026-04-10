# Resolving Spawn Targets

When a definition's `effects: [...]` list contains `Spawn(kind, tree)`, a spawn watcher is registered for the given EntityKind. No entities are resolved at dispatch time — the watcher is purely forward-looking.

When a new entity of the matching kind appears in the world, the tree is stamped onto it. Each spawned entity gets its own independent copy of the tree.

| EntityKind | Watches for |
|------------|-------------|
| Bolt | New entities with the Bolt component |
| Cell | New entities with the Cell component |
| Wall | New entities with the Wall component |
| Breaker | New entities with the Breaker component |
| Any | New entities of any of the above types |

The watcher stays active for as long as the source (chip, breaker definition, etc.) is active. When the source is removed (chip unequipped, entity despawned), the watcher is deregistered and no further entities receive the tree. Trees already stamped onto existing entities are not removed by deregistration — they follow the normal removal path (reverse_effect or source-based cleanup).
