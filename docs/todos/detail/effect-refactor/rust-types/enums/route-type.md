# Name
RouteType

# Syntax
```rust
enum RouteType {
    Bound,
    Staged,
}
```

# Description
- Bound: Permanently install the tree into the target's BoundEffects. The tree re-arms after each trigger match.
- Staged: Install the tree as a one-shot into the target's StagedEffects. Consumed after one trigger match.
