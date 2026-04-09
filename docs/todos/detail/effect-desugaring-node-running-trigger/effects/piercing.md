# Piercing

## Config
```rust
struct PiercingConfig {
    /// Number of cells the bolt can pass through without bouncing
    count: u32,
}
```
**RON**: `Piercing(count: 3)` or shorthand `Piercing(3)`

## Reversible: YES

## Target: Bolt

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActivePiercings(Vec<u32>);
```
- **Aggregation**: sum of all entries (default 0 when empty)
- `total()` → `self.0.iter().sum()` (or 0 if empty)

## Fire
1. Guard: if entity despawned, return
2. If `ActivePiercings` absent, insert default
3. Push `count` onto Vec

## Reverse
1. Find first matching `count` (exact u32 equality)
2. `swap_remove` it

## Notes
- Collision system reads `ActivePiercings::total()` — bolt passes through cells without bouncing until piercing count is exhausted
- Unlike multiplier effects, this uses **sum** aggregation, not product
- Stacks additively: [3, 2] → total 5 piercings
