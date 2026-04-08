# Protocol: Burnout

## Category
`custom-system`

## Game Design
You WANT to alternate frantic movement and deliberate stillness.

- Heat gauge fills while moving (4s to full), drains while stationary (2s to empty).
- Full heat: next bump deals 4x damage + fires a shockwave.
- Standing still for 1.5s: instant full drain + 2s breaker speed boost.
- Rhythm: move -> build heat -> stop -> drain -> speed boost -> move -> mega-bump.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct BurnoutConfig {
    /// Seconds of continuous movement to fill heat gauge from 0 to 1 (default 4.0).
    pub fill_duration: f32,
    /// Seconds for heat to drain from 1 to 0 while stationary (default 2.0).
    pub drain_duration: f32,
    /// Seconds of standing still before instant drain + speed boost triggers (default 1.5).
    pub still_threshold: f32,
    /// Damage multiplier applied on next bump when heat is full (default 4.0).
    pub full_heat_damage_multiplier: f32,
    /// Duration of breaker speed boost after still-threshold drain (default 2.0).
    pub speed_boost_duration: f32,
}
```

Populated from `ProtocolTuning::Burnout { fill_duration, drain_duration, still_threshold, full_heat_damage_multiplier, speed_boost_duration }`.

## Components
```rust
/// Tracks the breaker's heat gauge for the Burnout protocol.
/// Inserted on the breaker entity at protocol activation.
#[derive(Component, Debug)]
pub(crate) struct BurnoutHeat {
    /// Current heat level, 0.0 (empty) to 1.0 (full).
    pub heat: f32,
    /// How long the breaker has been stationary (seconds).
    pub still_timer: f32,
    /// Whether the full-heat mega-bump is charged and ready.
    pub mega_bump_charged: bool,
}

/// Marks that the breaker has a temporary speed boost from Burnout still-drain.
/// Removed when the timer expires.
#[derive(Component, Debug)]
pub(crate) struct BurnoutSpeedBoost {
    /// Remaining duration of the speed boost (seconds).
    pub remaining: f32,
}

/// Marks a bolt whose next cell impact should deal boosted damage from a mega-bump.
#[derive(Component, Debug)]
pub(crate) struct BurnoutDamageBoost {
    pub multiplier: f32,
}
```

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltImpactCell { cell, bolt }`
**Sends**: `DamageDealt<Cell>` (amplified damage from mega-bump), shockwave spawn (via effect system or direct entity spawn)

## Systems

### `burnout_update_heat`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Burnout)`, `in_state(NodeState::Playing)`
- **Behavior**: Each frame, checks breaker velocity. If breaker is moving (velocity magnitude > small epsilon): increases `heat` by `delta_secs / fill_duration`, clamps to 1.0, resets `still_timer` to 0.0. If `heat` reaches 1.0: sets `mega_bump_charged = true`. If breaker is stationary: increases `still_timer` by `delta_secs`, drains `heat` by `delta_secs / drain_duration`, clamps to 0.0. If `still_timer` >= `still_threshold`: instantly sets `heat = 0.0`, resets `still_timer = 0.0`, inserts `BurnoutSpeedBoost { remaining: speed_boost_duration }` on breaker, sets `mega_bump_charged = false`.
- **Ordering**: After breaker movement systems (needs current frame velocity).

### `burnout_on_bump`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Burnout)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BumpPerformed` messages. If `mega_bump_charged` is true: inserts `BurnoutDamageBoost { multiplier: full_heat_damage_multiplier }` on the bolt, resets `mega_bump_charged = false`, resets `heat = 0.0`. Also triggers a shockwave effect (same as existing shockwave effect dispatch — sends the appropriate effect command from the breaker's position).
- **Ordering**: After breaker `grade_bump`.

### `burnout_amplify_damage`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Burnout)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BoltImpactCell` messages. If bolt has `BurnoutDamageBoost`: sends `DamageDealt<Cell>` with `damage * multiplier`, removes `BurnoutDamageBoost` (consumed on first cell impact).
- **Ordering**: After bolt collision detection, before cell damage handling.

### `burnout_tick_speed_boost`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Burnout)`, `in_state(NodeState::Playing)`
- **Behavior**: Decrements `BurnoutSpeedBoost.remaining` by `delta_secs`. When `remaining <= 0.0`: removes the `BurnoutSpeedBoost` component. While `BurnoutSpeedBoost` is present, the breaker movement system should read it and apply a speed multiplier (cross-domain integration point).
- **Ordering**: Before breaker movement systems (so speed boost is current for this frame).

## Cross-Domain Dependencies
- **breaker domain**: Reads breaker `Velocity2D` (to determine moving vs stationary). Writes `BurnoutHeat`, `BurnoutSpeedBoost` on breaker entity. Breaker movement system needs to check for `BurnoutSpeedBoost` component and apply speed multiplier.
- **bolt domain**: Reads `BoltImpactCell` messages. Writes `BurnoutDamageBoost` on bolt entities.
- **cells domain**: Sends `DamageDealt<Cell>` message (amplified damage).
- **effect domain**: Dispatches shockwave effect on mega-bump (uses existing shockwave effect infrastructure).

## Expected Behaviors (for test specs)

1. **Heat fills while moving**
   - Given: Breaker with `BurnoutHeat { heat: 0.0, still_timer: 0.0, mega_bump_charged: false }`, `fill_duration = 4.0`. Breaker velocity magnitude = 200.0.
   - When: 2.0 seconds elapse with continuous movement.
   - Then: `heat` = 0.5.

2. **Heat reaches full and charges mega-bump**
   - Given: Breaker with `heat = 0.9`, `fill_duration = 4.0`. Breaker moving.
   - When: 0.5 seconds elapse with continuous movement.
   - Then: `heat` = 1.0, `mega_bump_charged = true`.

3. **Heat drains while stationary**
   - Given: Breaker with `heat = 1.0`, `drain_duration = 2.0`. Breaker velocity = 0.
   - When: 1.0 second elapses while stationary.
   - Then: `heat` = 0.5, `mega_bump_charged` unchanged (still true if was true).

4. **Standing still triggers instant drain + speed boost**
   - Given: Breaker with `heat = 0.8`, `still_threshold = 1.5`, `speed_boost_duration = 2.0`. Breaker stationary.
   - When: `still_timer` reaches 1.5s.
   - Then: `heat = 0.0`, `still_timer = 0.0`, `mega_bump_charged = false`, `BurnoutSpeedBoost { remaining: 2.0 }` inserted on breaker.

5. **Mega-bump applies 4x damage on next cell impact**
   - Given: `mega_bump_charged = true`, `full_heat_damage_multiplier = 4.0`. Bolt base damage = 10.0.
   - When: Bump performed, then bolt impacts cell.
   - Then: `DamageDealt<Cell>` with damage = 40.0. `BurnoutDamageBoost` removed from bolt. `mega_bump_charged` reset to false, `heat` reset to 0.0.

6. **Speed boost expires after duration**
   - Given: `BurnoutSpeedBoost { remaining: 2.0 }` on breaker.
   - When: 2.0 seconds elapse.
   - Then: `BurnoutSpeedBoost` component removed from breaker.

7. **Mega-bump also fires shockwave**
   - Given: `mega_bump_charged = true`.
   - When: Bump performed.
   - Then: Shockwave effect dispatched from breaker position (in addition to damage boost on bolt).

## Edge Cases
- Heat clamps at 1.0 — moving beyond full does not overflow or accumulate extra charge.
- Heat clamps at 0.0 — draining beyond empty does not go negative.
- Still-threshold drain resets `mega_bump_charged` even if heat was full (you chose to stand still instead of using the charge).
- Movement after still-threshold drain: `still_timer` resets to 0 on next movement frame; heat begins filling again from 0.
- Multiple bolts: mega-bump boost only applies to the bolt from the bump that consumed the charge. Other bolts are unaffected.
- Speed boost stacking: if still-threshold triggers again while boost is active, reset the timer to `speed_boost_duration` (do not stack multipliers).
- Mega-bump consumed even if bolt is lost before hitting a cell: the charge is spent at bump time, not at impact time. The `BurnoutDamageBoost` component is lost with the bolt.
