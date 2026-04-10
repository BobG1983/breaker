# EntropyEngine

## Config
```rust
struct EntropyConfig {
    /// Maximum effects fired per trigger activation
    max_effects: u32,
    /// Weighted pool of effects to choose from
    pool: Vec<(f32, Box<EffectType>)>,
}
```
**RON**: `EntropyEngine(max_effects: 3, pool: [(0.5, Shockwave(...)), (0.3, Explode(...)), (0.2, SpawnBolts(...))])`

## Reversible: YES (removes counter component)

## Target: Any

## Component
```rust
#[derive(Component)]
struct EntropyEngineState {
    cells_destroyed: u32,  // increments each fire, resets on node start
}
```

## Fire
1. Increment `cells_destroyed` counter (insert component if absent)
2. Select `min(cells_destroyed, max_effects)` random effects from the weighted pool
3. Fire each selected effect on the entity

## Reverse
1. Remove `EntropyEngineState` component

## Runtime System: `reset_entropy_engine_on_node_start`
Resets `EntropyEngineState.cells_destroyed` to 0 on node start.

## Notes
- Escalating chaos: fires more effects as more cells are destroyed
- Each trigger fires between 1 and max_effects random effects from the pool
- Counter is named `cells_destroyed` because this is typically triggered by cell death events
- Pool weights determine selection probability
- Random selection uses `GameRng`
- Effects from the pool are fired directly (they spawn their own entities as needed)
- Counter resets each node — escalation is per-node, not per-run
