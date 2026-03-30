# CircuitBreaker

Charge-and-release effect. Counts perfect bumps, then fires a spawn + shockwave burst when the counter fills.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `bumps_required` | `u32` | Perfect bumps needed to complete the circuit |
| `spawn_count` | `u32` | Number of bolts to spawn on completion |
| `inherit` | `bool` | Whether spawned bolts inherit the source bolt's effects |
| `shockwave_range` | `f32` | Shockwave radius on completion |
| `shockwave_speed` | `f32` | Shockwave expansion speed |

## Behavior

1. `fire()` inserts `CircuitBreakerCounter { remaining: bumps_required, config }` on the entity
2. Each time the entity receives a `PerfectBumped` trigger, the counter's `remaining` decrements by 1
3. When `remaining` reaches 0:
   - Fire `SpawnBolts(count: spawn_count, inherit)` via commands
   - Fire `Shockwave(shockwave_range, shockwave_speed)` via commands
   - Reset `remaining` to `bumps_required` (the circuit can charge again)
4. The cycle repeats for the duration of the node

## Reversal

Removes `CircuitBreakerCounter` from the entity. No further charges or releases.

## Ingredients

Feedback Loop x1 + Bump Force x2.

## VFX

- Persistent: Three-node triangle indicator near bolt, rendered as faint connected dots
- Charge: Each perfect bump lights a node (dim → bright)
- On completion: All three nodes flash white-hot (HDR >1.5), collapse inward, circuit closes
- Spawned bolt + shockwave fire with amplified VFX (larger than base shockwave)
- Screen flash + medium shake on circuit close
- The charge phase is subtle; the payoff is dramatic
