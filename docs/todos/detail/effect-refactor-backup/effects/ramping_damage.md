# RampingDamage

## Config
```rust
struct RampingDamageConfig {
    /// Damage bonus added per trigger activation
    damage_per_trigger: f32,
}
```
**RON**: `RampingDamage(damage_per_trigger: 1.5)`

## Reversible: YES (reverse removes entire state)

## Target: Bolt

## Component
```rust
#[derive(Component, Debug, Clone)]
struct RampingDamageState {
    damage_per_trigger: f32,
    accumulated: f32,
    trigger_count: u32,
}
```

## Fire
1. If `RampingDamageState` exists on entity: increment `accumulated` by `damage_per_trigger`, increment `trigger_count` by 1
2. If absent: insert fresh state with `accumulated = damage_per_trigger`, `trigger_count = 1`

## Reverse
1. Remove entire `RampingDamageState` component (full reset, not decremental)

## Notes
- Unlike other passive effects, this does NOT use Vec stacking — it maintains a single accumulator
- Each trigger activation adds flat damage bonus
- Reverse is a full reset — removes all accumulated damage
- Damage system reads `RampingDamageState.accumulated` as flat bonus added to base damage
- Linear accumulation: 4 triggers at 0.5 each → 2.0 accumulated
