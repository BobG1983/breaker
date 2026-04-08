# Protocol: Reckless Dash

## Category
`custom-system`

## Game Design
You WANT to be out of position and stretch your dash to the limit for a risky catch.

- "Risky catch" = bolt contacts breaker in the last 10-30% of dash distance (you had to stretch to reach it — weren't in position).
- Risky catch: next cell impact deals 4x damage.
- If bolt is lost during a dash: bolt loss triggers twice (double penalty).
- Non-risky catch (bolt contacts in early/mid dash, or while stationary): normal bump, no bonus, no penalty.
- Telegraph: TBD — needs visual indicator when bolt is in the "risky zone" near bottom.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct RecklessDashConfig {
    /// Fraction of dash distance where the risky zone begins (0.7 = last 30%).
    pub risky_zone_start: f32,
    /// Damage multiplier applied on risky catch (default 4.0).
    pub damage_multiplier: f32,
    /// Whether bolt-lost during dash triggers double penalty.
    pub double_penalty: bool,
}
```

Populated from `ProtocolTuning::RecklessDash { risky_zone_start, damage_multiplier, double_penalty }`.

## Components
```rust
/// Marks a bolt whose next cell impact should deal boosted damage from a risky catch.
#[derive(Component, Debug, Default)]
pub(crate) struct RiskyDamageBoost {
    /// Multiplier to apply on next cell impact (consumed on use).
    pub multiplier: f32,
}
```

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltImpactBreaker { bolt, breaker }`, `BoltLost`, `DamageDealt<Cell> { cell, damage, source_chip }`
**Sends**: `DamageDealt<Cell>` (modified damage), `BoltLost` (duplicate on dash bolt-lost)

## Systems

### `reckless_dash_on_bump`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::RecklessDash)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BumpPerformed` messages. For each bump, checks whether the breaker was in `DashState::Active` at the time of impact and whether the bolt contacted the breaker within the risky zone (last `1.0 - risky_zone_start` fraction of the dash distance). If risky: inserts `RiskyDamageBoost { multiplier: config.damage_multiplier }` on the bolt entity. If not risky (stationary, early/mid dash): no action.
- **Ordering**: After breaker `grade_bump` (needs `BumpPerformed` populated).

### `reckless_dash_amplify_damage`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::RecklessDash)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BoltImpactCell` messages. For each cell impact, checks if the bolt has a `RiskyDamageBoost` component. If so: sends a new `DamageDealt<Cell>` with `damage * multiplier` and removes the `RiskyDamageBoost` component (consumed on first cell impact).
- **Ordering**: After bolt collision detection, before cell damage handling.

### `reckless_dash_double_penalty`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::RecklessDash)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BoltLost` messages. Checks if the breaker was in `DashState::Active` when the bolt was lost. If `config.double_penalty` is true and breaker was dashing: sends an additional `BoltLost` message so that downstream consumers (life loss, time penalty) process the loss twice.
- **Ordering**: After bolt `bolt_lost` system, before effect `bridge_bolt_lost`.

## Cross-Domain Dependencies
- **breaker domain**: Reads `DashState` component (to determine if breaker is dashing and current dash progress), reads `Breaker` marker. Reads `BumpPerformed` message.
- **bolt domain**: Reads `BoltImpactBreaker`, `BoltImpactCell`, `BoltLost` messages. Writes to bolt entities (`RiskyDamageBoost` component).
- **cells domain**: Sends `DamageDealt<Cell>` message (amplified damage).

## Expected Behaviors (for test specs)

1. **Risky catch grants damage boost**
   - Given: Breaker in `DashState::Active`, dash 80% complete (risky_zone_start = 0.7). Bolt contacts breaker at dash position.
   - When: `reckless_dash_on_bump` processes `BumpPerformed`
   - Then: Bolt entity receives `RiskyDamageBoost { multiplier: 4.0 }`

2. **Non-risky catch grants no boost**
   - Given: Breaker in `DashState::Active`, dash 50% complete (risky_zone_start = 0.7). Bolt contacts breaker.
   - When: `reckless_dash_on_bump` processes `BumpPerformed`
   - Then: Bolt entity has no `RiskyDamageBoost` component.

3. **Stationary catch grants no boost**
   - Given: Breaker in `DashState::Idle` (not dashing). Bolt contacts breaker.
   - When: `reckless_dash_on_bump` processes `BumpPerformed`
   - Then: Bolt entity has no `RiskyDamageBoost` component.

4. **Damage boost consumed on first cell impact**
   - Given: Bolt has `RiskyDamageBoost { multiplier: 4.0 }`. Bolt base damage = 10.0.
   - When: Bolt impacts a cell.
   - Then: `DamageDealt<Cell>` sent with damage = 40.0. `RiskyDamageBoost` removed from bolt.

5. **Damage boost does not persist to second cell impact**
   - Given: Bolt had `RiskyDamageBoost`, consumed on first cell impact.
   - When: Same bolt impacts a second cell.
   - Then: `DamageDealt<Cell>` sent with normal damage (10.0). No boost applied.

6. **Bolt-lost during dash triggers double penalty**
   - Given: Breaker in `DashState::Active`, `double_penalty = true`. Bolt falls below playfield.
   - When: `reckless_dash_double_penalty` processes `BoltLost`
   - Then: An additional `BoltLost` message is sent (total 2 `BoltLost` events for downstream consumers).

7. **Bolt-lost while stationary triggers single penalty**
   - Given: Breaker in `DashState::Idle`. Bolt falls below playfield.
   - When: `reckless_dash_double_penalty` processes `BoltLost`
   - Then: No additional `BoltLost` message sent (only the original 1 event).

## Edge Cases
- Risky zone boundary: bolt contacts breaker at exactly `risky_zone_start` fraction (e.g., 70.0%) — should NOT trigger boost (risky zone is strictly beyond the threshold).
- Multiple bolts: each bolt independently tracked for `RiskyDamageBoost`. One risky catch does not affect other bolts.
- `double_penalty = false` in config: bolt-lost during dash still only triggers once (config toggle).
- Dash ends between bump and cell impact: `RiskyDamageBoost` persists on the bolt regardless of breaker state change — it was earned at bump time.
- Bolt has `RiskyDamageBoost` but is lost before hitting a cell: boost is simply lost with the bolt entity (no special handling needed).
