# Hazard: Drift

## Game Design

Wind pushes the bolt in a telegraphed direction, changing every ~8 seconds. Force scales with stacks. Player must read the wind indicator and compensate with breaker positioning + bump angles.

## Config Resource

```rust
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftConfig {
    pub force: f32,           // force magnitude at stack 1
    pub per_level_force: f32, // additional magnitude per stack beyond the first
    pub period_secs: f32,     // seconds between direction changes; does not scale with stacks
}
```

Populated from `HazardTuning::Drift`.

## Runtime Resource

```rust
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftWind {
    pub direction: Vec2, // unit vector
    pub timer: f32,      // seconds until next direction change
}
```

One global wind resource affecting all bolts.

## Messages
**Sends**: `ApplyBoltForce { bolt: Entity, force: Vec2 }` — one per active bolt per tick, emitted by `drift_apply_force`. Bolt-domain consumer (`apply_bolt_forces` in `BoltSystems::ApplyForces`) aggregates forces and writes `force * dt` to each bolt's `Velocity2D` before `SpatialSystems::ApplyVelocity`. Drift does NOT write `Velocity2D` directly.

## Systems

The two systems are chained and registered together via `.chain().before(BoltSystems::ApplyForces).run_if(hazard_active(HazardKind::Drift)).run_if(in_state(NodeState::Playing))`.

### `drift_update_wind`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Drift)` + `in_state(NodeState::Playing)` (from chained tuple).
- **Behavior**: `DriftWind.timer -= delta_secs`. If `timer <= 0`: picks a new unit vector via seeded `HazardRng`; resets `timer = config.period_secs`.
- **Ordering**: Before `drift_apply_force` (enforced by `.chain()`).

### `drift_apply_force`
- **Schedule**: `FixedUpdate`, `.before(BoltSystems::ApplyForces)` (from chained tuple).
- **run_if**: `hazard_active(HazardKind::Drift)` + `in_state(NodeState::Playing)` (from chained tuple).
- **Behavior**: For each bolt: computes `magnitude = config.force + config.per_level_force * (stack - 1)` via `DriftConfig::force_magnitude(stacks)`. Emits `ApplyBoltForce { bolt, force: DriftWind.direction * magnitude }`.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Drift does not participate in any `DeathPipelineSystems` set.
- **Trigger**: FixedUpdate tick.
- **Emits**: `ApplyBoltForce` — bolt-domain consumer (`apply_bolt_forces`) aggregates and writes `Velocity2D` in `BoltSystems::ApplyForces` before `SpatialSystems::ApplyVelocity`.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement. **No** direct `Velocity2D` write.

## Stacking Behavior

Linear: `force = config.force + config.per_level_force * (stack - 1)`.

| Stack | Force | Notes |
|-------|-------|-------|
| 1 | base (1×) | Easily compensated |
| 2 | 4/3 × base (1.33×) | Active compensation required |
| 3 | 5/3 × base (1.67×) | Significant drift |

`period_secs` is NOT stack-dependent — only force magnitude scales.

## Cross-Domain Dependencies
- **bolt**: Consumes `ApplyBoltForce` via `apply_bolt_forces` in `BoltSystems::ApplyForces`. Owns force-aggregation + `Velocity2D` write.
- **shared**: Reads `Time`. Uses seeded `HazardRng` for deterministic direction changes.

## Expected Behaviors (for test specs)

1. **Wind applies force at stack 1** — `force: 100.0`, direction `(1,0)`, one bolt: `ApplyBoltForce { force: (100, 0) }`.
2. **Stack 3 applies larger force** — `force: 100.0`, `per_level_force: 33.33`, direction `(0,-1)`, stack 3: `ApplyBoltForce { force: (0, -166.67) }`.
3. **Wind direction changes after interval** — `timer: 0.05`, `delta_secs: 0.1`: new unit direction, `timer = period_secs`.
4. **Direction stays constant within interval** — `timer: 4.0`, `delta_secs: 0.1`: direction unchanged; `timer: 3.9`.
5. **Force applied to each bolt separately** — 3 bolts: 3 × `ApplyBoltForce` messages.

## Edge Cases
- **Drift + Gravity Surge**: both emit `ApplyBoltForce`. Bolt-domain consumer sums forces. Player compensates for both.
- **Cleanup**: `DriftConfig` + `DriftWind` removed at run end.
- **Direction RNG**: seeded `HazardRng` — deterministic from run seed for replay.
- **`period_secs` is stack-independent**: only force magnitude scales. Keeps mechanic readable.
- **Zero bolts**: no messages emitted.
- **Multi-bolt**: all bolts receive the same wind force; divergence emerges from existing velocities.
