# SizeBoost

## Config
```rust
struct SizeBoostConfig {
    /// Size multiplier (1.x format: 2.0 = double size)
    multiplier: f32,
}
```
**RON**: `SizeBoost(multiplier: 2.0)` or shorthand `SizeBoost(2.0)`

## Reversible: YES

## Target: Bolt (radius) or Breaker (width)

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActiveSizeBoosts(Vec<f32>);
```
- **Aggregation**: product of all entries (default 1.0 when empty)
- `multiplier()` → `self.0.iter().product()` (or 1.0 if empty)

## Fire
1. Guard: if entity despawned, return
2. If `ActiveSizeBoosts` absent, insert default
3. Push `multiplier` onto Vec

## Reverse
1. Find first matching `multiplier` (epsilon compare)
2. `swap_remove` it

## Notes
- Size systems read `ActiveSizeBoosts::multiplier()` and multiply base radius/width
- Stacks multiplicatively
