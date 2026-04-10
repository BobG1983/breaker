# Name
RootNode

# Derives
`Debug, Clone, PartialEq, Eq, Serialize, Deserialize`

# Syntax
```rust
enum RootNode {
    Stamp(StampTarget, Tree),
    Spawn(EntityKind, Tree),
}
```

# Description
- Stamp: Install a tree on a target entity or entity group. See [stamp-target.md](enums/stamp-target.md), [tree.md](tree.md)
- Spawn: Watch for new entities of a given kind and apply a tree to each one. See [entity-kind.md](enums/entity-kind.md), [tree.md](tree.md)
