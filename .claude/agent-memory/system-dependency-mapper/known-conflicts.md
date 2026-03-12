---
name: known-conflicts
description: Known query conflicts, ordering issues, and missing constraints identified in the brickbreaker system map (as of 2026-03-12)
type: reference
---

# Conflict Analysis

## Query Conflicts — FixedUpdate (PlayingState::Active)

### Transform Write Conflicts

Multiple systems in FixedUpdate write Transform on Bolt entities. They are correctly ordered:

1. `bolt_cell_collision` writes `Transform` (Bolt) — runs first via `.after(BoltSystems::PrepareVelocity)`
2. `bolt_breaker_collision` writes `Transform` (Bolt) — runs `.after(bolt_cell_collision)`
3. `bolt_lost` writes `Transform` (Bolt) — runs `.after(bolt_breaker_collision)`

No conflict: chain is total.

Multiple systems write Transform on Breaker entities:

1. `move_breaker` writes `Transform` (Breaker) — in_set(BreakerSystems::Move)
2. `animate_bump_visual` writes `Transform` (Breaker) — in **Update**, not FixedUpdate

No conflict: they are in different schedules.

### BoltVelocity Write Conflicts

In FixedUpdate:
1. `prepare_bolt_velocity` writes `BoltVelocity` — in_set(BoltSystems::PrepareVelocity)
2. `bolt_cell_collision` writes `BoltVelocity` — .after(BoltSystems::PrepareVelocity)
3. `bolt_breaker_collision` writes `BoltVelocity` — .after(bolt_cell_collision)
4. `bolt_lost` writes `BoltVelocity` — .after(bolt_breaker_collision)
5. `apply_bump_velocity` writes `BoltVelocity` — **NO ORDERING CONSTRAINT** ⚠️

**POTENTIAL CONFLICT**: `apply_bump_velocity` and `prepare_bolt_velocity`/`bolt_*_collision`/`bolt_lost` all write `BoltVelocity` on the same Bolt entities in the same FixedUpdate schedule with no ordering between them.

However, `apply_bump_velocity` only fires when there is a `BumpPerformed` message — and `BumpPerformed` is sent by:
- `grade_bump` (which runs .after(update_breaker_state) in the same tick a BoltHitBreaker arrives)
- `update_bump` (on timeout only)

`BoltHitBreaker` is sent by `bolt_breaker_collision`, so in the same tick that generates a bump message, `bolt_breaker_collision` has already set BoltVelocity. The concern is: does `apply_bump_velocity` see the final or intermediate velocity from that tick?

In Bevy 0.18, messages from the current update are visible to readers in the same update. So on the tick where `bolt_breaker_collision` fires and sends `BoltHitBreaker`:
- `grade_bump` (via BoltHitBreaker reader) sends `BumpPerformed`
- `apply_bump_velocity` (via BumpPerformed reader) multiplies the velocity

If `apply_bump_velocity` runs *before* `bolt_breaker_collision` in the same tick, it would multiply the old pre-reflection velocity and then `bolt_breaker_collision` would overwrite it — making the bump multiplier invisible that tick.

If `apply_bump_velocity` runs *after* `bolt_breaker_collision` (due to arbitrary Bevy scheduling), it correctly amplifies the reflected velocity.

**VERDICT**: Non-deterministic ordering. The bump velocity amplification could be applied to pre-reflection or post-reflection velocity depending on scheduler order. Needs `.after(bolt_breaker_collision)` or `.after(BoltSystems::PrepareVelocity)` at minimum.

### BreakerState Write Conflicts

In FixedUpdate:
1. `update_breaker_state` writes `BreakerState` — .after(move_breaker)
2. `perfect_bump_dash_cancel` writes `BreakerState` — .after(grade_bump) which is .after(update_breaker_state)

No conflict: fully ordered. `perfect_bump_dash_cancel` runs after `update_breaker_state`.

### BumpState Write

Only `update_bump` writes `BumpState`. No conflict.

### Assets<ColorMaterial> Write

In FixedUpdate (PlayingState::Active):
- `handle_cell_hit` writes `ResMut<Assets<ColorMaterial>>`

In OnEnter(GameState::Playing):
- `spawn_breaker` writes `ResMut<Assets<ColorMaterial>>`
- `spawn_bolt` writes `ResMut<Assets<ColorMaterial>>`
- `spawn_cells` writes `ResMut<Assets<ColorMaterial>>`

OnEnter and FixedUpdate don't overlap. No conflict.

## Ordering Issues

### `apply_bump_velocity` vs physics systems (CONFIRMED ISSUE)

`apply_bump_velocity` has no ordering relative to:
- `prepare_bolt_velocity` (in_set BoltSystems::PrepareVelocity)
- `bolt_cell_collision` (.after BoltSystems::PrepareVelocity)
- `bolt_breaker_collision` (.after bolt_cell_collision)
- `bolt_lost` (.after bolt_breaker_collision)

**Impact**: On any tick where a BumpPerformed message is present, the velocity multiplication may occur before the CCD physics step, making it get overwritten; or after, making it work correctly. Bevy will pick an order based on system graph ambiguity resolution (typically consistent per run but not guaranteed).

**Fix needed**: Add `.after(bolt_breaker_collision)` or `.after(BoltSystems::PrepareVelocity)` to `apply_bump_velocity` in BoltPlugin's registration.

### `spawn_bump_grade_text` vs `grade_bump` (MINOR)

`spawn_bump_grade_text` is registered `.after(update_bump)` but reads BumpPerformed which is also sent by `grade_bump` (which runs much later, after update_breaker_state). Within the same tick, `spawn_bump_grade_text` will see BumpPerformed messages from the *previous* tick's `grade_bump` (if Bevy 0.18 double-buffers messages), or the current tick's `update_bump` only. This is the expected double-buffer behavior in Bevy 0.18 messages — readers see messages written in the previous update, not the same update. So in practice `spawn_bump_grade_text` will see `grade_bump`'s messages with a 1-tick delay. This is cosmetic-only (text feedback), so acceptable but worth noting.

## Missing Message Consumers (Future Work)

These are intentional stubs, not bugs:

| Message | Expected Future Consumer | Phase |
|---------|--------------------------|-------|
| `CellDestroyed` | run (node clear tracking), upgrades, audio | Phase 2+ |
| `BoltLost` | breaker (penalty logic) | Phase 2+ |
| `BoltHitBreaker` | audio, upgrades, UI | Phase 2+ |
| `BoltHitCell` | upgrades, audio | Phase 2+ |
| `BumpPerformed` | audio, upgrades | Phase 2+ |
| `NodeCleared` | state machine, UI | Phase 2+ |
| `TimerExpired` | state machine | Phase 2+ |
| `UpgradeSelected` | upgrades plugin | Phase 3+ |
