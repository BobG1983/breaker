# Name
Terminal

# Syntax
```rust
enum Terminal {
    Fire(EffectType),
    Route(RouteType, Box<Tree>),
}
```

# Description
- Fire: Execute an effect immediately on the Owner. See [effect-type.md](enums/effect-type.md)
- Route: Install a tree on another entity. RouteType controls permanence. See [route-type.md](enums/route-type.md), [tree.md](tree.md)
