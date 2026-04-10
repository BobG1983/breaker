# Vulnerable

## Config
```rust
struct VulnerableConfig {
    /// Damage multiplier applied to incoming hits (2.0 = double damage taken)
    multiplier: f32,
}
```
**RON**: `Vulnerable(multiplier: 2.0)`

**In EffectType enum**: `Vulnerable(f32)` -- bare f32, no config struct wrapper (matches SpeedBoost(f32) pattern).

## Reversible: YES

## Target: Cell (primarily)

## Component
```rust
#[derive(Component, Debug, Default, Clone)]
struct ActiveVulnerability(Vec<f32>);
```
- **Aggregation**: product of all entries (default 1.0 when empty)
- `multiplier()` → `self.0.iter().product()` (or 1.0 if empty)

## Fire
1. Guard: if entity despawned, return
2. If `ActiveVulnerability` absent, insert default
3. Push `multiplier` onto Vec

## Reverse
1. Find first matching `multiplier` (epsilon compare)
2. `swap_remove` it

## Notes
- Damage system reads `ActiveVulnerability::multiplier()` and multiplies incoming damage
- Applied to cells to make them take more damage
- Stacks multiplicatively
