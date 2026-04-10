# QuickStop

## Config
```rust
struct QuickStopConfig {
    /// Deceleration multiplier (2.0 = 2x faster deceleration)
    multiplier: f32,
}
```
**RON**: `QuickStop(multiplier: 2.0)` or shorthand `QuickStop(2.0)`

## Reversible: YES

## Target: Breaker

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActiveQuickStops(Vec<f32>);
```
- **Aggregation**: product of all entries (default 1.0 when empty)
- `multiplier()` → `self.0.iter().product()` (or 1.0 if empty)

## Fire
1. If `ActiveQuickStops` absent on entity, insert default
2. Push `multiplier` onto Vec

## Reverse
1. Find first matching `multiplier` (epsilon compare)
2. `swap_remove` it

## Notes
- Breaker movement system reads `ActiveQuickStops::multiplier()` to scale deceleration
- Stacks multiplicatively
