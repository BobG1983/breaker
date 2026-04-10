# Name
Hp

# Syntax
```rust
#[derive(Component, Debug, Clone)]
struct Hp {
    current: f32,
    starting: f32,
    max: Option<f32>,
}
```

# Description
Unified health component for all damageable entities. Replaces `CellHealth`.

- current: Hit points remaining. Damage decrements this. Death occurs when current ≤ 0.
- starting: The HP this entity spawned with. Used for visual damage feedback (health fraction = current / starting).
- max: Optional upper bound. If Some, healing cannot exceed this value. If None, no upper bound. Cells typically have max = None (no healing). Future entities with regen or overshield may set max.

DO use Hp on every entity that can take damage (cells, walls with HP, breakers with life-as-HP).
DO NOT use CellHealth anymore — Hp replaces it entirely.
