# Protocol: Afterimage

## Category
`custom-system`

## Game Design
You WANT to dash AWAY from where the bolt will be, not toward it.

- Dash: phantom breaker appears at your starting position for 2s.
- Bolt bounces off phantom with normal rebound physics.
- Perfect Bump during a phantom bounce: bolt becomes Phantom for a limited duration.
- Phantom bolt passes through cells (damaging them) instead of bouncing off. Still bounces off walls and real breaker normally.
- When duration expires, bolt returns to normal.
- Duration does NOT reset if triggered again while already Phantom — must wait for it to return to normal first.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct AfterimageConfig {
    /// How long the phantom breaker entity persists after a dash (seconds, default 2.0).
    pub phantom_duration: f32,
    /// How long a bolt remains in Phantom state after a phantom Perfect Bump (seconds).
    pub phantom_bolt_duration: f32,
}
```

Populated from `ProtocolTuning::Afterimage { phantom_duration, phantom_bolt_duration }`.

## Components
```rust
/// Marks an entity as a phantom breaker spawned by the Afterimage protocol.
/// The phantom is a temporary collision body at the breaker's pre-dash position.
#[derive(Component, Debug)]
pub(crate) struct PhantomBreaker {
    /// Remaining lifetime (seconds). Entity despawned when this reaches 0.
    pub remaining: f32,
}

/// Marks a bolt that is in Phantom state: passes through cells (damaging them)
/// instead of bouncing, but still bounces off walls and real breaker.
#[derive(Component, Debug)]
pub(crate) struct PhantomBolt {
    /// Remaining duration of Phantom state (seconds). When this reaches 0,
    /// the component is removed and the bolt returns to normal collision behavior.
    pub remaining: f32,
}
```

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltImpactCell { cell, bolt }`, `BoltImpactBreaker { bolt, breaker }`
**Sends**: `DamageDealt<Cell> { cell, damage, source_chip }` (phantom bolt piercing damage)

## Systems

### `afterimage_spawn_phantom`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Detects when the breaker begins a dash (transition to `DashState::Active`). At dash start: spawns a phantom breaker entity at the breaker's current position with `PhantomBreaker { remaining: config.phantom_duration }`. The phantom has collision geometry matching the breaker's size but no movement. Only one phantom may exist at a time — if a previous phantom is still alive, despawn it before spawning the new one.
- **Entity components**: `PhantomBreaker`, `Transform` (at breaker's pre-dash position), collision AABB matching breaker size, visual sprite/mesh (semi-transparent breaker appearance).
- **Ordering**: After breaker dash initiation systems.

### `afterimage_tick_phantom`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Decrements `PhantomBreaker.remaining` by `delta_secs`. When `remaining <= 0.0`: despawns the phantom breaker entity.
- **Ordering**: After `afterimage_spawn_phantom`.

### `afterimage_phantom_bounce`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Handles bolt collision with the phantom breaker entity. The phantom uses the same collision body as a real breaker, so the bolt bounces with normal rebound physics. On collision: generates a `BoltImpactBreaker`-equivalent event with the phantom entity. This allows the bump grading system to evaluate the impact.
- **Implementation note**: The phantom needs to participate in the collision pipeline. Options: (a) give the phantom the same collision layer as the breaker, letting the existing bolt-breaker collision system handle it naturally; (b) run a separate collision check. Option (a) is preferred if the collision system can distinguish phantom from real breaker for bump grading purposes.
- **Ordering**: Within bolt collision detection.

### `afterimage_promote_to_phantom_bolt`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BumpPerformed` messages. If the bump grade is `Perfect` AND the bump was against a `PhantomBreaker` entity (not the real breaker): checks if the bolt already has `PhantomBolt`. If not: inserts `PhantomBolt { remaining: config.phantom_bolt_duration }` on the bolt. If the bolt already has `PhantomBolt` (duration has not expired): no-op (duration does not reset).
- **Ordering**: After `grade_bump`, after `afterimage_phantom_bounce`.

### `afterimage_tick_phantom_bolt`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Decrements `PhantomBolt.remaining` by `delta_secs` for all bolts with the component. When `remaining <= 0.0`: removes `PhantomBolt` component. Bolt returns to normal collision behavior.
- **Ordering**: Before bolt collision detection (so phantom state is current for this frame).

### `afterimage_phantom_bolt_pierce`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: When a bolt with `PhantomBolt` contacts a cell: deals damage (sends `DamageDealt<Cell>`) but does NOT bounce (velocity unchanged). The bolt passes through the cell. This overrides normal bolt-cell collision rebound for phantom bolts only. Bolt still bounces off walls and real/phantom breakers normally.
- **Implementation note**: This likely requires the bolt-cell collision system to check for `PhantomBolt` and skip the rebound step while still sending the damage message. The protocol domain provides the marker; the bolt domain reads it.
- **Ordering**: Within bolt-cell collision handling.

## Cross-Domain Dependencies
- **breaker domain**: Reads `DashState` transitions (to detect dash start). Reads breaker `Transform` and collision size (to position and size the phantom). Reads `BumpPerformed`.
- **bolt domain**: Reads `BoltImpactBreaker`, `BoltImpactCell` messages. Writes `PhantomBolt` on bolt entities. Bolt-cell collision system needs to check `PhantomBolt` to skip rebound.
- **cells domain**: Sends `DamageDealt<Cell>` message (phantom bolt piercing damage).
- **collision pipeline**: Phantom breaker needs to participate in bolt-breaker collision detection. Bolt with `PhantomBolt` needs modified cell collision response (damage without rebound).

## Expected Behaviors (for test specs)

1. **Phantom breaker spawns at dash start position**
   - Given: Breaker at position (100.0, 50.0). Afterimage protocol active, `phantom_duration = 2.0`.
   - When: Breaker begins a dash.
   - Then: `PhantomBreaker { remaining: 2.0 }` entity spawned at (100.0, 50.0) with breaker-sized collision body.

2. **Phantom breaker despawns after duration**
   - Given: `PhantomBreaker { remaining: 2.0 }` exists.
   - When: 2.0 seconds elapse.
   - Then: Phantom breaker entity despawned.

3. **Bolt bounces off phantom breaker**
   - Given: Phantom breaker at (100.0, 50.0). Bolt moving toward it at velocity (0.0, -400.0).
   - When: Bolt contacts phantom breaker.
   - Then: Bolt rebounds with normal physics (velocity y-component inverted).

4. **Perfect bump on phantom promotes bolt to Phantom state**
   - Given: Bolt contacts phantom breaker. Bump graded as `Perfect`. Bolt does not have `PhantomBolt`. `phantom_bolt_duration = 3.0`.
   - When: `afterimage_promote_to_phantom_bolt` processes the bump.
   - Then: Bolt receives `PhantomBolt { remaining: 3.0 }`.

5. **Non-perfect bump on phantom does NOT promote bolt**
   - Given: Bolt contacts phantom breaker. Bump graded as `Early`.
   - When: `afterimage_promote_to_phantom_bolt` processes the bump.
   - Then: Bolt does NOT receive `PhantomBolt`.

6. **Phantom bolt passes through cells dealing damage**
   - Given: Bolt has `PhantomBolt { remaining: 2.0 }`. Bolt velocity = (200.0, 400.0). Bolt base damage = 10.0.
   - When: Bolt contacts a cell.
   - Then: `DamageDealt<Cell>` sent with damage = 10.0. Bolt velocity remains (200.0, 400.0) (no rebound). Bolt continues on its trajectory.

7. **Phantom bolt still bounces off walls**
   - Given: Bolt has `PhantomBolt`. Bolt moving toward a wall.
   - When: Bolt contacts wall.
   - Then: Normal wall rebound (velocity component inverted). `PhantomBolt` unaffected.

8. **Phantom bolt still bounces off real breaker**
   - Given: Bolt has `PhantomBolt`. Bolt moving toward the real breaker (not phantom).
   - When: Bolt contacts real breaker.
   - Then: Normal bump physics. `PhantomBolt` unaffected.

9. **Phantom bolt duration expires, bolt returns to normal**
   - Given: Bolt has `PhantomBolt { remaining: 0.1 }`.
   - When: 0.1 seconds elapse.
   - Then: `PhantomBolt` removed. Next cell contact results in normal rebound.

10. **Duration does NOT reset while already Phantom**
    - Given: Bolt has `PhantomBolt { remaining: 1.0 }`. Bolt contacts phantom breaker again with Perfect bump.
    - When: `afterimage_promote_to_phantom_bolt` processes the bump.
    - Then: `PhantomBolt.remaining` stays at 1.0 (no reset). Must wait for expiration before re-triggering.

## Edge Cases
- New dash while phantom exists: old phantom despawned, new one created at new dash start position.
- Phantom breaker does not move — it stays at the pre-dash position regardless of breaker movement.
- Phantom bolt damages multiple cells in a single pass if trajectory crosses them — each contact sends a separate `DamageDealt<Cell>`.
- Bolt-lost while Phantom: normal bolt-lost behavior (Phantom state doesn't prevent bolt-lost).
- Phantom breaker collision with non-bolt entities: phantom only interacts with bolts. Does not block breaker, cells, or other entities.
- Bump grading on phantom: the phantom needs enough state for the bump grading system to evaluate quality (position, width). It uses the breaker's size at the time of dash.
- Multiple bolts: each bolt independently tracks `PhantomBolt`. One bolt becoming Phantom does not affect others.
