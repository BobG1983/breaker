# Protocol: Reckless Dash

## Category
`custom-system`

## Game Design
You WANT to be out of position and stretch your dash to the limit for a risky catch.

- "Risky catch" = bolt contacts breaker in the last portion of dash distance (stretched to reach it — you weren't in position).
- Risky catch: next cell impact deals multiplied damage (default 4x).
- If a bolt is lost during a dash: bolt loss triggers with double penalty (the `BoltLossBehavior` pipeline is invoked twice).
- Non-risky catch (bolt contacts in early/mid dash, or while stationary): normal bump — no bonus, no penalty.
- Telegraph: visual indicator of the "risky zone" near the bottom (Phase 5 VFX).

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct RecklessDashConfig {
    pub risky_zone_start: f32,      // 0.7 means last 30%
    pub damage_multiplier: f32,     // 4.0
    pub double_penalty: bool,       // true
}
```

## Components
None owned by Reckless Dash post-TODO #3 — the anti-feedback guard + `RiskyDamageBoost` component are retired. Damage amplification rides the shared `DamageBoostStack` (Pattern B, one-shot).

## Messages
**Reads**: `BumpPerformed { grade, bolt, breaker }` (breaker domain), `BoltLost { bolt }` (bolt-lifecycle).
**Sends**: None. Damage amplification goes through `DamageBoostStack::add_one_shot` (crate API). Double bolt-loss handled by calling the `BoltLossBehavior` handler twice (not by synthetic `BoltLost` message — the behaviour handler is idempotent-per-call, invoked twice on dash-active bolt loss).

## Systems

### `reckless_dash_on_bump`
- **Schedule**: `FixedUpdate`, `.after(BreakerSystems::GradeBump)`.
- **run_if**: `protocol_active(ProtocolKind::RecklessDash)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. For each bump, checks if the breaker's `DashState` is `Active` AND the current dash progress is `>= risky_zone_start`. If risky: `DamageBoostStack::add_one_shot(config.damage_multiplier)` on the bolt (Pattern B, consume-on-use).

### `reckless_dash_double_penalty`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::RecklessDash)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BoltLost`. Checks if the breaker was in `DashState::Active` at loss time. If `config.double_penalty` AND dashing: invokes the `BoltLossBehavior` handler a second time for the same bolt (simulating double penalty). The first invocation is the standard bolt-lifecycle handler; this system's second invocation is protocol-driven.
- **Ordering**: Runs after the standard bolt-lifecycle `BoltLost` handler.

## Pipeline position (dmg crate)

- **Trigger**: `BumpPerformed` (risky catch detection); `BoltLost` (double-penalty path).
- **Writes**: `DamageBoostStack::add_one_shot(multiplier)` on the bolt — Pattern B, consume-on-use.
- **Consumed in**: `DeathPipelineSystems::ApplyDamageBoosts` — when the next `DamageDealt<Cell>` for that bolt aggregates, the one-shot multiplies in and is cleared.
- **No damage emission** from Reckless Dash; no `VulnerableStack` interaction.
- **Bolt loss double-penalty** invokes the `BoltLossBehavior` handler (TODO #3) twice; not part of the death pipeline.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed`. Reads `DashState` (active state + progress).
- **bolt**: Reads `BoltLost`. Writes `DamageBoostStack::add_one_shot`.
- **damage crate (`rantzsoft_dmg`)**: `ApplyDamageBoosts` aggregates the one-shot.
- **state/run/breaker (bolt-loss pipeline)**: `BoltLossBehavior` handler invoked twice for dash-active loss.

## Expected Behaviors (for test specs)

1. **Risky catch pushes one-shot** — `risky_zone_start: 0.7`, dash at 80%, Perfect-or-any graded bump: `DamageBoostStack::one_shots.push(config.damage_multiplier)` on the bolt.
2. **Non-risky catch (early/mid dash) pushes nothing** — dash at 50%: no one-shot push.
3. **Stationary catch pushes nothing** — `DashState::Idle`: no one-shot.
4. **One-shot consumed on first cell impact** — bolt has pushed one-shot 4.0, base damage 10: `DamageDealt<Cell>` emits 40 via `ApplyDamageBoosts`; one-shot cleared.
5. **One-shot does NOT persist to second impact** — subsequent cell impact: normal damage (no boost).
6. **Bolt-lost during dash triggers double penalty** — `BoltLost` + `DashState::Active` + `double_penalty: true`: `BoltLossBehavior` handler called twice.
7. **Bolt-lost while stationary single penalty** — `DashState::Idle`: single invocation.

## Edge Cases
- **Risky zone boundary**: bolt contacts at exactly `risky_zone_start` (e.g. 70%) — does NOT trigger (strictly greater than).
- **Multiple bolts**: each bolt tracks its own one-shots. Risky catch on bolt A doesn't affect bolt B.
- **`double_penalty: false`**: single invocation even on dash-active loss.
- **Dash ends between bump and cell impact**: one-shot persists on the bolt regardless of dash state change — it was earned at bump time.
- **Bolt has one-shot but is lost before cell impact**: one-shot disappears with the bolt entity.
- **No `RiskyDamageBoost` component**: retired. All damage amplification lives in `DamageBoostStack::one_shots` (crate-owned).
- **No anti-feedback guard**: retired per TODO #3. `BoltLossBehavior` is idempotent-per-call; the second invocation is safe.
