# DamageBoost

## Config
```rust
struct DamageBoostConfig {
    /// Damage multiplier (1.x format: 2.0 = double damage)
    multiplier: f32,
}
```
**RON**: `DamageBoost(multiplier: 2.0)` or shorthand `DamageBoost(2.0)`

## Reversible: YES

## Target: Bolt (primarily)

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActiveDamageBoosts(Vec<f32>);
```
- **Aggregation**: product of all entries (default 1.0 when empty)
- `multiplier()` → `self.0.iter().product()` (or 1.0 if empty)

## Fire
1. Guard: if entity despawned, return
2. If `ActiveDamageBoosts` absent, insert default
3. Push `multiplier` onto Vec

## Reverse
1. Find first matching `multiplier` (epsilon compare)
2. `swap_remove` it

## Notes
- Damage systems read `ActiveDamageBoosts::multiplier()` and multiply base damage
- No velocity recalculation needed (unlike SpeedBoost)
- Stacks multiplicatively: [2.0, 1.5] → effective 3.0x
