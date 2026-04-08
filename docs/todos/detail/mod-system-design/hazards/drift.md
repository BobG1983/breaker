# Hazard: Drift

## Game Design

Wind pushes the bolt in a telegraphed direction, changing every ~8 seconds. Force scales as `base_force + base_force / 3 / level`. The player must read the wind direction indicator and compensate with breaker positioning and bump angles. Direction changes are telegraphed (visual indicator) so the player can anticipate.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DriftConfig {
    /// Base wind force magnitude at stack 1.
    pub base_force: f32,
    /// Additional force per stack (base_force / 3).
    pub force_per_level: f32,
    /// Seconds between wind direction changes.
    pub change_interval: f32,
}
```

Extracted from `HazardTuning::Drift { base_force, force_per_level, change_interval }` at activation time.

## Components

### `DriftWind`

```rust
/// Current wind state for the Drift hazard. Inserted as a resource at activation.
#[derive(Resource, Debug)]
pub(crate) struct DriftWind {
    /// Current wind direction (unit vector).
    pub direction: Vec2,
    /// Timer until next direction change.
    pub timer: f32,
}
```

This is a resource, not a per-entity component. There is one global wind direction affecting all bolts.

## Messages

**Reads**: None (reads `Time` for delta, `ActiveHazards` for stack count)
**Sends**: `ApplyBoltForce { bolt: Entity, force: Vec2 }` — new message, owned by `bolt` domain. Sent once per bolt per tick.

## Systems

### `drift_update_wind`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Drift).and(in_state(NodeState::Playing))`
- **Ordering**: Before `drift_apply_force` (same hazard, ordering within the module)
- **Behavior**: Tick `DriftWind.timer` down. When it expires, pick a new random direction (unit vector from seeded RNG), reset timer to `change_interval`. The direction change should be telegraphed visually (separate FX system in `Update`).

### `drift_apply_force`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Drift).and(in_state(NodeState::Playing))`
- **Ordering**: `.after(drift_update_wind)`, before bolt physics integration
- **Behavior**: For each bolt entity, compute force magnitude from config + stack count, multiply by `DriftWind.direction`, and send `ApplyBoltForce { bolt, force }`.
- **Formula**: `force_magnitude = base_force + force_per_level * (stack - 1)`

The bolt domain receives `ApplyBoltForce` and adds it to the bolt's velocity (or acceleration, depending on physics model).

## Stacking Behavior

Linear scaling: `force = base_force + force_per_level * (stack - 1)` where `force_per_level = base_force / 3`.

| Stack | Force magnitude | Notes |
|-------|----------------|-------|
| 1 | `base_force` (1.0x) | Noticeable push, easily compensated |
| 2 | `base_force * 4/3` (1.33x) | Requires active compensation |
| 3 | `base_force * 5/3` (1.67x) | Significant drift, hard to ignore |

Each additional stack adds `base_force / 3` to the force. The `change_interval` does NOT change with stacking — only force magnitude increases.

## Cross-Domain Dependencies

| Domain | Direction | Message |
|--------|-----------|---------|
| `bolt` | sends to | `ApplyBoltForce` — applies wind force to bolt velocity |

Drift never reads or writes bolt velocity directly. The bolt domain owns velocity and applies the force.

## Expected Behaviors (for test specs)

1. **Wind applies force to bolt at stack 1**
   - Given: Drift active at stack 1, `base_force=100.0`, `force_per_level=33.33`, wind direction `(1.0, 0.0)`, one bolt entity
   - When: `drift_apply_force` runs
   - Then: `ApplyBoltForce { bolt, force: Vec2(100.0, 0.0) }` is sent

2. **Wind applies stronger force at stack 3**
   - Given: Drift active at stack 3, same config, wind direction `(0.0, -1.0)`, one bolt entity
   - When: `drift_apply_force` runs
   - Then: `ApplyBoltForce { bolt, force: Vec2(0.0, -166.67) }` is sent (100.0 + 33.33 * 2)

3. **Wind direction changes after interval**
   - Given: Drift active, `change_interval=8.0`, `DriftWind.timer=0.05`, `delta_secs=0.1`
   - When: `drift_update_wind` runs
   - Then: `DriftWind.direction` changes to a new unit vector, `DriftWind.timer` resets to 8.0

4. **Wind direction stays constant within interval**
   - Given: Drift active, `DriftWind.timer=4.0`, `delta_secs=0.1`
   - When: `drift_update_wind` runs
   - Then: `DriftWind.direction` unchanged, `DriftWind.timer=3.9`

5. **Force applied to all bolts**
   - Given: 3 bolt entities exist
   - When: `drift_apply_force` runs
   - Then: 3 separate `ApplyBoltForce` messages sent, one per bolt

## Edge Cases

- **Drift + Gravity Surge synergy**: Two forces on the bolt simultaneously. Both send `ApplyBoltForce` — the bolt domain sums all forces. The player must compensate for both, which is readable but demanding.
- **Node-end cleanup**: `DriftConfig` and `DriftWind` resources removed at run end via `hazards::cleanup()`.
- **Wind direction RNG**: Uses seeded `GameRng` so direction changes are deterministic from run seed. Important for replay/scenario reproducibility.
- **Change interval is not stack-dependent**: The interval stays constant regardless of stacks. Only force magnitude scales. This keeps the mechanic readable — the player learns the rhythm.
- **Zero bolts**: If no bolts exist (between spawns), `drift_apply_force` sends no messages. No special handling needed.
- **Multi-bolt scenarios**: Each bolt receives the same wind force. Bolts diverge based on their existing velocities, not differential wind.
