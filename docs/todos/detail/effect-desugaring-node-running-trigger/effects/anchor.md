# Anchor

## Config
```rust
struct AnchorConfig {
    /// Bump force multiplier when planted
    bump_force_multiplier: f32,
    /// Perfect window multiplier when planted
    perfect_window_multiplier: f32,
    /// Seconds breaker must remain stationary before planting
    plant_delay: f32,
}
```
**RON**: `Anchor(bump_force_multiplier: 2.0, perfect_window_multiplier: 1.5, plant_delay: 0.5)`

## Reversible: YES

## Target: Breaker

## Components
```rust
#[derive(Component)]
struct AnchorActive {
    bump_force_multiplier: f32,
    perfect_window_multiplier: f32,
    plant_delay: f32,
}

#[derive(Component)]
struct AnchorTimer(f32);  // countdown timer

#[derive(Component)]
struct AnchorPlanted;  // marker — breaker is anchored
```

## Fire
1. Guard: if entity despawned, return
2. Insert `AnchorActive` with config values (overwrites if existing)

## Reverse
1. Guard: if entity despawned, return
2. Remove `AnchorActive`, `AnchorTimer`, and `AnchorPlanted` from entity

## Runtime System: `tick_anchor`
**Schedule**: FixedUpdate, run_if NodeState::Playing

State machine driven by `Velocity2D` and `DashState`:
- **Stationary**: zero velocity AND (Idle or Settling dash state)
- **Moving**: nonzero velocity OR Dashing/Braking dash state

Transitions:
1. **Moving → cancel**: remove `AnchorTimer` and `AnchorPlanted`. On un-plant, pop the anchor's `bump_force_multiplier` from `ActiveBumpForces`
2. **Stationary + timer active → tick**: decrement timer by dt. When timer reaches zero: remove timer, insert `AnchorPlanted`, push `bump_force_multiplier` onto `ActiveBumpForces`
3. **Stationary + no timer + not planted → start**: insert `AnchorTimer(plant_delay)`
4. **Stationary + planted → steady state**: no-op

## Notes
- The bump system reads `AnchorPlanted` to know when the anchor multipliers are active
- `perfect_window_multiplier` is read directly from `AnchorActive` by the bump timing system
- Anchor interacts with `ActiveBumpForces` — pushes/pops its multiplier on plant/unplant
