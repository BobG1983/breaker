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
/// Zero-field marker — presence alone distinguishes phantom from real breaker.
#[derive(Component, Debug)]
pub(crate) struct PhantomBreaker;

/// Tracks the remaining lifetime of a phantom breaker (seconds).
/// Entity is despawned when this reaches 0. Separate from the marker so the
/// marker can be queried without carrying a mutable float.
#[derive(Component, Debug)]
pub(crate) struct PhantomBreakerLifetime(pub f32);
```

**Note**: `PhantomBolt`, `PhantomLifetime`, and `PhantomOwner` are NOT defined locally. They are re-used from `crate::effect_v3::effects::phantom_bolt::components`. The `PhantomBolt` marker causes the bolt to bypass cell rebound (passing through cells). `PhantomLifetime` tracks remaining duration — lifetime tick-down is delegated to `tick_phantom_lifetime` registered by `EffectV3Plugin` via `SpawnPhantomConfig::register`. `PhantomOwner` links the phantom bolt back to the real bolt that spawned it (uniqueness guard: one phantom per real bolt at a time).

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltImpactCell { cell, bolt }`, `BoltImpactBreaker { bolt, breaker }`
**Sends**: `DamageDealt<Cell> { cell, damage, source_chip }` (phantom bolt piercing damage)

## Systems

**Note**: 4 runtime systems are implemented (not 6). Phantom bolt lifetime tick-down is delegated to `tick_phantom_lifetime` in `EffectV3Plugin`. Cleanup is via `CleanupOnExit::<NodeState>::default()` on all spawned phantom entities — no custom cleanup system required.

### `afterimage_tick_phantom_breaker`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Decrements `PhantomBreakerLifetime` by `delta_secs` for all phantom breaker entities. When `remaining <= 0.0`: despawns the phantom breaker entity.
- **Ordering**: Chained before `afterimage_spawn_phantom_breaker`.

### `afterimage_spawn_phantom_breaker`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Detects when the breaker begins a dash (transition to `DashState::Active`). At dash start: despawns any existing phantom breaker, then spawns a new phantom breaker entity at the breaker's pre-dash position with `PhantomBreaker` marker + `PhantomBreakerLifetime(config.phantom_duration)`. The phantom has collision geometry matching the breaker's size (same BREAKER_LAYER so bolt-breaker collision fires naturally) but does not move.
- **Ordering**: Chained after `afterimage_tick_phantom_breaker`, chained before `afterimage_check_phantom_bounce`.

### `afterimage_check_phantom_bounce`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BoltImpactBreaker` messages. When the contacted breaker entity carries `PhantomBreaker`: re-emits a new `BoltImpactBreaker` message with `breaker = phantom_entity` so the bump grading system evaluates the phantom contact with normal bump logic.
- **Ordering**: `.before(BreakerSystems::GradeBump)`, `.after(BoltSystems::CellCollision)` — must run after collision detection fires (to see `BoltImpactBreaker`) but before `grade_bump` consumes them.

### `afterimage_spawn_phantom_bolt`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Afterimage)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BumpPerformed` messages. For each Perfect-graded bump whose `breaker` field references a `PhantomBreaker` entity: checks `PhantomOwner` components on existing phantom bolts — skips if any phantom already references the same real bolt (uniqueness guard, duration NOT reset). If no existing phantom: spawns a new phantom bolt via `Bolt::builder().extra().headless()` at the real bolt's position/velocity. Inserts `PhantomBolt`, `PhantomLifetime(config.phantom_bolt_duration)`, `PhantomOwner(real_bolt_entity)`. Phantom bolt receives `BOLT_LAYER` in its collision mask (so it hits other bolts in addition to cells/walls/breaker).
- **Ordering**: `.after(BreakerSystems::GradeBump)`, `.before(EffectV3Systems::Tick)` — must run after grade_bump resolves the Perfect grade and before effect tick processes the new phantom bolt entity.

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
