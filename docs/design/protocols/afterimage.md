# Protocol: Afterimage

## Category
`custom-system`

## Game Design
You WANT to dash AWAY from where the bolt will be, not toward it.

- Dash: a phantom breaker appears at your starting position for a short duration.
- Bolt bounces off phantom with normal rebound physics.
- Perfect Bump during a phantom bounce: the **real bolt** becomes Phantom for a limited duration (it mutates in place — no new bolt is spawned).
- Phantom bolt passes through cells (damaging them) instead of bouncing off. Still bounces off walls and the real breaker normally.
- When the phantom duration expires, the bolt reverts to normal (`LifetimeEndBehavior::RevertToNormalBolt`).
- Duration does NOT reset if triggered again while already Phantom — must wait for it to revert first.

## Config Resource
```rust
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct AfterimageConfig {
    /// Seconds a `PhantomBreaker` entity exists before its lifetime tick despawns it.
    pub phantom_duration: f32,
    /// Seconds a spawned phantom-bolt entity exists before `tick_phantom_lifetime` despawns it.
    pub phantom_bolt_duration: f32,
}
```

Populated from `ProtocolTuning::Afterimage`.

## Components
Afterimage owns no per-mechanic components. It spawns the phantom breaker via `Breaker::builder().phantom(BreakerPhantomParams { lifespan: config.phantom_duration, .. })`, which attaches `Lifespan` + `PhantomFlicker` + `PhantomBreaker` marker from the breaker domain.

For the bolt mutation, it calls `Bolt::become_phantom(PhantomParams { lifespan, end_behavior: LifetimeEndBehavior::RevertToNormalBolt })` on the real bolt (TODO #2 builder API), attaching `PhantomBolt` + `Lifespan` + `LifetimeEndBehavior::RevertToNormalBolt`. On lifespan expiry, `PhantomBolt::become_normal` reverts the bolt in place.

## Messages
**Reads**: `BumpPerformed { grade, bolt, breaker }` (breaker domain), `DashStateChanged` (breaker domain).
**Sends**: None directly. Damage through cells flows through the `PhantomBolt` collision path which emits `DamageDealt<Cell>` into `DeathPipelineSystems::EmitDamage` via the standard bolt-cell collision system.

## Systems

### `afterimage_spawn_phantom_breaker`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Afterimage)` + `in_state(NodeState::Playing)`.
- **Behavior**: Detects `DashState` transition to `Dashing`. Despawns any existing phantom breaker (`Query<Entity, With<PhantomBreaker>>`), then spawns a new one via `Breaker::builder().phantom(BreakerPhantomParams { lifespan: config.phantom_duration, .. }).spawn()` at the breaker's current position. The shared `tick_phantom_flicker` (in `Update`) and `tick_phantom_breaker_lifespan` (in `FixedUpdate`) handle flicker VFX and despawn.
- **Ordering**: After breaker movement systems (to read pre-dash position).

### `afterimage_promote_bolt_to_phantom`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Afterimage)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. On `BumpGrade::Perfect` where `breaker` is a `PhantomBreaker` entity AND the real bolt does NOT already have `PhantomBolt`: calls `Bolt::become_phantom` on the bolt entity with `PhantomParams { lifespan: config.phantom_bolt_duration, end_behavior: LifetimeEndBehavior::RevertToNormalBolt }`. If the bolt IS already phantom, skip — uniqueness guard, duration NOT reset.
- **Ordering**: `.after(BreakerSystems::GradeBump)`.

Lifespan tick-down and revert-on-expiry are owned by the shared bolt/phantom infrastructure (TODO #5 `tick_lifespan` + `handle_lifetime_end_behavior`). Afterimage itself has no tick system.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Afterimage owns no systems in any `DeathPipelineSystems` set.
- **Trigger**: `DashStateChanged` (spawn phantom), `BumpPerformed` Perfect on phantom breaker (promote bolt).
- **Damage flow**: phantom bolts pierce cells and emit `DamageDealt<Cell>` through the standard bolt-cell collision path in `DeathPipelineSystems::EmitDamage`. No amplification — base damage only.
- **No** `DamageBoostStack` / `VulnerableStack` / direct `DamageDealt` emission from Afterimage itself.

## Cross-Domain Dependencies
- **breaker domain**: Reads `DashState` transitions. Uses `Breaker::builder().phantom(...)` (shared phantom infra).
- **bolt domain**: Uses `Bolt::become_phantom` + `PhantomBolt::become_normal` (shared mutation API). Relies on `LifetimeEndBehavior::RevertToNormalBolt` for expiry behaviour.
- **cells domain**: Phantom bolts emit `DamageDealt<Cell>` on cell contact; no rebound (handled by existing `PhantomBolt` collision branch).
- **collision pipeline**: Phantom breaker participates in bolt-breaker collision detection. Bolt with `PhantomBolt` bypasses cell rebound while still rebounding off walls and real breaker.

## Expected Behaviors (for test specs)

1. **Phantom breaker spawns at dash start position** — dash begins at breaker position (100.0, 50.0); a new entity with `PhantomBreaker` + `Lifespan(config.phantom_duration)` + `PhantomFlicker` spawns at (100.0, 50.0).
2. **Phantom breaker despawns after lifespan** — shared `tick_lifespan` decrements; at `remaining <= 0`, entity despawns.
3. **Bolt bounces off phantom breaker normally** — phantom carries the same collision layer as the real breaker; bolt-breaker collision reflects the bolt with normal rebound physics.
4. **Perfect bump on phantom mutates real bolt into Phantom** — `BumpPerformed { grade: Perfect, breaker: <phantom entity> }`; `Bolt::become_phantom` attaches `PhantomBolt` + `Lifespan` + `LifetimeEndBehavior::RevertToNormalBolt`.
5. **Non-perfect bump on phantom does NOT promote** — `BumpPerformed { grade: Early | Late }` on phantom: no `become_phantom` call.
6. **Phantom bolt passes through cells dealing damage** — bolt with `PhantomBolt` emits `DamageDealt<Cell>` on contact but does not rebound; velocity unchanged.
7. **Phantom bolt still bounces off walls and real breaker** — wall and real-breaker collision branches don't consult `PhantomBolt`.
8. **Phantom lifespan expiry reverts bolt to normal** — at `Lifespan::remaining <= 0`, `handle_lifetime_end_behavior` sees `RevertToNormalBolt` and calls `PhantomBolt::become_normal`; next cell contact resumes normal rebound.
9. **Duration does NOT reset while already Phantom** — second Perfect bump on phantom breaker is a no-op while `PhantomBolt` still present.

## Edge Cases
- **New dash while phantom exists**: old phantom despawned immediately, new one spawned at new dash position.
- **Phantom breaker does not move** — it stays at pre-dash position regardless of real breaker movement.
- **Phantom bolt damages multiple cells in a single pass** if trajectory crosses them — each contact emits a separate `DamageDealt<Cell>`.
- **Bolt-lost while Phantom**: standard bolt-lost fires; phantom state doesn't prevent it.
- **Phantom breaker collides only with bolts** — does not block cells or other entities.
- **Multiple bolts**: each tracks `PhantomBolt` independently; one becoming phantom doesn't affect others.
