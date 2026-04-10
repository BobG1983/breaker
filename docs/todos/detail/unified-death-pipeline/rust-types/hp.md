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
Unified health component for all damageable entities. Replaces `CellHealth` and `LivesCount`.

- current: Hit points remaining. Damage decrements this. Death occurs when current ≤ 0.
- starting: The HP this entity spawned with. Used for visual damage feedback (health fraction = current / starting).
- max: Optional upper bound. If Some, healing cannot exceed this value. If None, no upper bound.

Mapping from old types:
- `CellHealth::new(30.0)` → `Hp { current: 30.0, starting: 30.0, max: None }`
- `LivesCount(Some(3))` → `Hp { current: 3.0, starting: 3.0, max: None }`
- `LivesCount(None)` (infinite lives) → no Hp component. An entity without Hp cannot die from damage.
- `LoseLife` effect → sends `DamageDealt<Breaker> { amount: 1.0, dealer: None, ... }`

DO use Hp on every entity that can take damage (cells, walls with HP, breakers with finite lives).
DO NOT add Hp to breakers with infinite lives — they cannot die.
DO NOT use CellHealth or LivesCount anymore — Hp replaces both entirely.
