# Name
Terminal

# Syntax
```rust
enum Terminal {
    Fire(EffectType),
    Stamp(Box<Tree>),
    Route(Box<Tree>),
}
```

# Description
- Fire: Execute an effect immediately on the Owner. See [effect-type.md](enums/effect-type.md)
- Stamp: Add a tree permanently to the target's BoundEffects. See [tree.md](tree.md)
- Route: Add a tree one-shot to the target's StagedEffects, consumed after one trigger match. See [tree.md](tree.md)
