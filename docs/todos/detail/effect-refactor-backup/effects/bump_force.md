# BumpForce

## Config
```rust
struct BumpForceConfig {
    /// Bump force multiplier (1.x format: 1.5 = 50% more force)
    multiplier: f32,
}
```
**RON**: `BumpForce(multiplier: 1.5)` or shorthand `BumpForce(1.5)`

## Reversible: YES

## Target: Breaker

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActiveBumpForces(Vec<f32>);
```
- **Aggregation**: product of all entries (default 1.0 when empty)
- `multiplier()` → `self.0.iter().product()` (or 1.0 if empty)

## Fire
1. Guard: if entity despawned, return
2. If `ActiveBumpForces` absent, insert default
3. Push `multiplier` onto Vec

## Reverse
1. Find first matching `multiplier` (epsilon compare)
2. `swap_remove` it

## Notes
- Bump system reads `ActiveBumpForces::multiplier()` to scale bump force
- Stacks multiplicatively
