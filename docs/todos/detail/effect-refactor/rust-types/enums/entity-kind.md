# Name
EntityKind

# Derives
`Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`

# Syntax
```rust
enum EntityKind {
    Cell,
    Bolt,
    Wall,
    Breaker,
    Any,
}
```

# Description
- Cell: Matches cell entities
- Bolt: Matches bolt entities
- Wall: Matches wall entities
- Breaker: Matches breaker entities
- Any: Matches any entity type
