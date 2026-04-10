# Name
ScopedTerminal

# Syntax
```rust
enum ScopedTerminal {
    Fire(ReversibleEffectType),
    Route(RouteType, Box<Tree>),
}
```

# Description
- Fire: Execute a reversible effect immediately on the Owner. See [reversible-effect-type.md](enums/reversible-effect-type.md)
- Route: Install a tree on another entity. RouteType controls permanence. See [route-type.md](enums/route-type.md), [tree.md](tree.md)
