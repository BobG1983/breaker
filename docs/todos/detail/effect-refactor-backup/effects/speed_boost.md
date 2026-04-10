# SpeedBoost

## Config
```rust
struct SpeedBoostConfig {
    /// Speed multiplier (1.x format: 1.5 = 50% faster)
    multiplier: f32,
}
```
**RON**: `SpeedBoost(multiplier: 1.5)` or shorthand `SpeedBoost(1.5)`

## Reversible: YES

## Target: Bolt (primarily), but can target any entity with velocity

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActiveSpeedBoosts(Vec<f32>);
```
- **Aggregation**: product of all entries (default 1.0 when empty)
- `multiplier()` → `self.0.iter().product()` (or 1.0 if empty)

## Fire
1. Guard: if entity despawned, return
2. If `ActiveSpeedBoosts` absent on entity, insert default
3. Push `multiplier` onto the Vec
4. Recalculate bolt velocity: query `SpatialData`, call `apply_velocity_formula` with current boosts

## Reverse
1. Find first entry matching `multiplier` (epsilon compare: `(v - multiplier).abs() < f32::EPSILON`)
2. `swap_remove` it
3. Recalculate bolt velocity (same as fire)

## Notes
- Velocity recalculation uses `apply_velocity_formula` from bolt domain — preserves direction, adjusts magnitude
- Multiple boosts stack multiplicatively: [1.5, 2.0] → effective 3.0x
- Reverse removes exactly ONE matching entry (not all)
