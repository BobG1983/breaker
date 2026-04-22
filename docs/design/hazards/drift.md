# Hazard: Drift

## Game Design

Wind pushes the bolt in a telegraphed direction, changing every ~8 seconds. Force scales with stacks. Player must read the wind indicator and compensate with breaker positioning + bump angles.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DriftConfig {
    pub base_force: f32,
    pub force_per_level: f32,       // base_force / 3
    pub change_interval: f32,       // 8.0
}
```

Populated from `HazardTuning::Drift`.

## Components

```rust
#[derive(Resource, Debug)]
pub(crate) struct DriftWind {
    pub direction: Vec2,    // unit vector
    pub timer: f32,         // seconds until next direction change
}
```

Resource — one global wind affecting all bolts.

## Messages
**Reads**: `Time` for delta; `Option<Res<ActiveHazards>>` for stack; `Option<ResMut<GameRng>>` for new direction selection.
**Sends**: `ApplyBoltForce { bolt: Entity, force: Vec2 }` — one per active bolt per tick. Bolt-domain consumer aggregates forces in `FixedUpdate` before `BoltSystems::IntegrateMotion` (per TODO #8). Drift does NOT write `Velocity2D` directly.

## Systems

### `drift_update_wind`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Drift)` + `in_state(NodeState::Playing)`.
- **Behavior**: `DriftWind.timer -= delta_secs`. If `timer <= 0`: picks a new unit vector via seeded `GameRng`; resets `timer = change_interval`.
- **Ordering**: Before `drift_apply_force`.

### `drift_apply_force`
- **Schedule**: `FixedUpdate`, `.after(drift_update_wind)`, `.before(BoltSystems::IntegrateMotion)` (via the `ApplyBoltForce` pipeline ordering).
- **run_if**: `hazard_active(HazardKind::Drift)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each bolt: computes `magnitude = base_force + force_per_level * (stack - 1)`. Emits `ApplyBoltForce { bolt, force: DriftWind.direction * magnitude }`.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Drift does not participate in any `DeathPipelineSystems` set.
- **Trigger**: FixedUpdate tick.
- **Emits**: `ApplyBoltForce` — bolt-domain consumer aggregates and writes `Velocity2D` in `FixedUpdate` before `BoltSystems::IntegrateMotion`.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement. **No** direct `Velocity2D` write.

## Stacking Behavior

Linear: `force = base_force + force_per_level * (stack - 1)`.

| Stack | Force | Notes |
|-------|-------|-------|
| 1 | base (1×) | Easily compensated |
| 2 | 4/3 × base (1.33×) | Active compensation required |
| 3 | 5/3 × base (1.67×) | Significant drift |

`change_interval` is NOT stack-dependent — only force magnitude scales.

## Cross-Domain Dependencies
- **bolt**: Consumes `ApplyBoltForce`. Owns force-aggregation + `Velocity2D` write.
- **shared**: Reads `Time`. Uses seeded `GameRng` for deterministic direction changes.

## Expected Behaviors (for test specs)

1. **Wind applies force at stack 1** — `base_force: 100.0`, direction `(1,0)`, one bolt: `ApplyBoltForce { force: (100, 0) }`.
2. **Stack 3 applies larger force** — `base_force: 100.0`, `force_per_level: 33.33`, direction `(0,-1)`, stack 3: `ApplyBoltForce { force: (0, -166.67) }`.
3. **Wind direction changes after interval** — `timer: 0.05`, `delta_secs: 0.1`: new unit direction, `timer: 8.0`.
4. **Direction stays constant within interval** — `timer: 4.0`, `delta_secs: 0.1`: direction unchanged; `timer: 3.9`.
5. **Force applied to each bolt separately** — 3 bolts: 3 × `ApplyBoltForce` messages.

## Edge Cases
- **Drift + Gravity Surge**: both emit `ApplyBoltForce`. Bolt-domain consumer sums forces. Player compensates for both.
- **Cleanup**: `DriftConfig` + `DriftWind` removed at run end.
- **Direction RNG**: seeded `GameRng` — deterministic from run seed for replay.
- **Change interval is stack-independent**: only magnitude scales. Keeps mechanic readable.
- **Zero bolts**: no messages emitted.
- **Multi-bolt**: all bolts receive the same wind force; divergence emerges from existing velocities.
