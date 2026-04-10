# Name
ScopedTerminal

# Syntax
```rust
enum ScopedTerminal {
    Fire(ReversibleEffectType),
    Stamp(Box<Tree>),
    Route(Box<Tree>),
}
```

# Description
- Fire: Execute a reversible effect immediately on the Owner. See [reversible-effect-type.md](enums/reversible-effect-type.md)
- Stamp: Add a tree permanently to the target's BoundEffects. See [tree.md](tree.md)
- Route: Add a tree one-shot to the target's StagedEffects. See [tree.md](tree.md)
