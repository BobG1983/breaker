---
name: known-conflicts
description: Known query conflicts, ordering issues, and missing constraints identified in the brickbreaker system map (as of 2026-03-16, post-behaviors-extraction)
type: reference
---

# Known Conflicts and Ordering Issues

Last updated: 2026-03-16 (behaviors domain extraction — BehaviorsPlugin standalone; spawn_additional_bolt now orders .after(BehaviorSystems::Bridge))

---

## RESOLVED — apply_bump_velocity ordering vs bolt_lost

`apply_bump_velocity` runs `.after(BreakerCollision).before(BoltLost)`. Correct and confirmed.

---

## RESOLVED — handle_run_lost ordering vs handle_node_cleared / handle_timer_expired

**run/plugin.rs current registration:**
```rust
handle_run_lost
    .after(handle_node_cleared)
    .after(handle_timer_expired),
```
The previously-flagged ordering gap is now fixed. Win (node cleared) takes priority over loss.

---

## CONFIRMED — animate_bump_visual and animate_tilt_visual write Transform on Breaker in Update

Both run in Update, both write `&mut Transform` on `With<Breaker>`. No ordering constraint.
- `animate_bump_visual` writes `transform.translation.y`
- `animate_tilt_visual` writes `transform.rotation`

**Severity:** Low. Different fields. Bevy serializes them. No logical conflict.

---

## LOW SEVERITY — handle_cell_hit has no ordering vs track_node_completion

`handle_cell_hit` (CellsPlugin) sends `CellDestroyed`. `track_node_completion` reads it.
No cross-plugin ordering constraint. One-tick delay acceptable — messages persist.

---

## NEW LOW SEVERITY — apply_time_penalty and handle_timer_expired are unordered

**Files:** `src/run/node/plugin.rs`, `src/run/plugin.rs`

`apply_time_penalty` is registered `.after(NodeSystems::TickTimer)`.
`handle_timer_expired` is registered `.after(NodeSystems::TickTimer).after(handle_node_cleared)`.

Both depend on `NodeSystems::TickTimer` completing, but neither is ordered relative to the other.
When `apply_time_penalty` sends a `TimerExpired` message (penalty drives timer to zero), that
message may not be read by `handle_timer_expired` until the next tick if `handle_timer_expired`
runs before `apply_time_penalty` in the same tick.

**Practical consequence:** A time-penalty-driven timer expiry may be delayed by one fixed tick
(~16ms at 60Hz). This is imperceptible in gameplay — `tick_node_timer` will catch any remaining
time on the next tick anyway.

**Severity:** Low. One-tick delay on time-penalty-induced timer expiry.

**Fix (optional):** Add `apply_time_penalty.before(handle_timer_expired)` constraint in NodePlugin,
OR restructure so `apply_time_penalty` is in its own system set that `handle_timer_expired` is ordered after. For example in node/plugin.rs:
```rust
apply_time_penalty
    .after(NodeSystems::TickTimer)
    .before(handle_timer_expired),  // ensure same-tick propagation
```
But `handle_timer_expired` is in run/plugin.rs, so the cross-plugin ordering would need to be
established from NodePlugin's side referencing the RunPlugin system function directly — which
breaks plugin encapsulation. The 1-tick delay is the correct trade-off to preserve encapsulation.

---

## NO CONFLICT — interpolation pipeline schedule ordering

`restore_authoritative` (FixedFirst) runs before ALL FixedUpdate systems by schedule.
`store_authoritative` (FixedPostUpdate) runs after ALL FixedUpdate systems by schedule.
`interpolate_transform` (PostUpdate) runs after ALL Update systems by schedule.

These are distinct schedules with no overlap. No ordering constraint needed — the schedule
hierarchy itself enforces the correct pipeline:
```
FixedFirst:        restore_authoritative     ← restores Transform = physics.current
FixedUpdate:       [all physics/gameplay]    ← moves bolts via Transform
FixedPostUpdate:   store_authoritative       ← captures Transform → physics.current
                   clear_input_actions
PostUpdate:        interpolate_transform     ← lerps Transform between previous/current
```

---

## NO CONFLICT — interpolate_transform (PostUpdate) vs animate_bump_visual / animate_tilt_visual (Update)

`interpolate_transform` writes `Transform.translation.x/y` on Bolt entities (With<InterpolateTransform>).
`animate_bump_visual` writes `Transform.translation.y` on Breaker entities (With<Breaker>).
`animate_tilt_visual` writes `Transform.rotation` on Breaker entities (With<Breaker>).

Bolt and Breaker are different entities. No archetype overlap. No conflict.

---

## NO CONFLICT — restore_authoritative (FixedFirst) vs physics mutation systems

`restore_authoritative` runs in FixedFirst and completes before any FixedUpdate system starts.
All physics systems (bolt_cell_collision, bolt_breaker_collision, bolt_lost, hover_bolt, etc.)
run in FixedUpdate. They see the restored authoritative position, not the interpolated one.
This is exactly the correct invariant. No conflict.

---

## NO CONFLICT — store_authoritative (FixedPostUpdate) vs clear_input_actions

`store_authoritative` reads `&Transform` and writes `PhysicsTranslation`.
`clear_input_actions` writes `ResMut<InputActions>`.
Completely disjoint data access. Both run in FixedPostUpdate with no ordering needed.

---

## NO CONFLICT — spawn_additional_bolt query vs physics bolt queries

`spawn_additional_bolt` reads `Query<&Transform, With<Breaker>>` (read-only).
Physics systems read `Query<&Transform, (With<Breaker>, Without<Bolt>)>` (same — read-only).

Both are read-only on Breaker Transform. Bevy allows multiple simultaneous readers.
No conflict even if run in parallel.

`spawn_additional_bolt` spawns new entities via Commands — deferred, applied after FixedUpdate.
The spawned ExtraBolt entities will not be visible to physics queries in the same tick.
This is correct: the new bolt appears on the next tick, which is the intended behavior.

---

## RESOLVED — spawn_additional_bolt now orders after BehaviorSystems::Bridge

`spawn_additional_bolt` previously ordered `.after(PhysicsSystems::BreakerCollision)`.
It now orders `.after(BehaviorSystems::Bridge)` — which runs after BreakerCollision.
This guarantees the SpawnAdditionalBolt message written by the bridge observer is readable
in the same tick.

`apply_bump_velocity` orders `.after(BreakerCollision).before(BoltLost)`.
`spawn_additional_bolt` orders `.after(BehaviorSystems::Bridge)`.
No explicit ordering between them — but no conflict because spawn_additional_bolt uses
only Commands (deferred). The spawned entity is not visible in the current tick.

---

## ORDERING REFERENCE — Full FixedUpdate Chain (PlayingState::Active)

```
FixedFirst:
  restore_authoritative  [InterpolatePlugin]

FixedUpdate:
  update_bump  (BreakerPlugin)
    → move_breaker (.after(update_bump), BreakerSystems::Move)
        → update_breaker_state (.after(move_breaker))
        → hover_bolt (.after(BreakerSystems::Move))
        → prepare_bolt_velocity (.after(BreakerSystems::Move), BoltSystems::PrepareVelocity)
            → bolt_cell_collision (.after(BoltSystems::PrepareVelocity))
                → bolt_breaker_collision (.after(bolt_cell_collision), BreakerCollision set)
                    → apply_bump_velocity (.after(BreakerCollision), .before(BoltLost))
                    → grade_bump (.after(update_bump) AND .after(BreakerCollision))
                    → bridge_bump (.after(BreakerCollision), BehaviorSystems::Bridge, conditional)
                        → [observer: handle_time_penalty] → ApplyTimePenalty message
                        → [observer: handle_spawn_bolt] → SpawnAdditionalBolt message
                    → track_bump_result (.after(BreakerCollision), dev only)
                    → bolt_lost (.after(bolt_breaker_collision), BoltLost set)
                        → bridge_bolt_lost (.after(BoltLost), BehaviorSystems::Bridge, conditional)
                            → [observer: handle_life_lost] → RunLost message
                            → [observer: handle_time_penalty] → ApplyTimePenalty message
                    → spawn_additional_bolt (.after(BehaviorSystems::Bridge))
                        [reads SpawnAdditionalBolt message written by bridge observer — Commands only]
              → grade_bump continuations: perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text (.after(grade_bump))

  [unordered floaters in same run_if group]:
    launch_bolt  — ServingBoltFilter (disjoint)
    spawn_bolt_lost_text  — reads BoltLost message
    trigger_bump_visual  — reads InputActions, Commands
    handle_cell_hit  — reads BoltHitCell, sends CellDestroyed

  [NodePlugin ordered chain]:
    track_node_completion (NodeSystems::TrackCompletion)  [unordered vs handle_cell_hit]
    tick_node_timer (NodeSystems::TickTimer)
    apply_time_penalty (.after(NodeSystems::TickTimer))  [unordered vs handle_timer_expired — 1-tick delay acceptable]

  [RunPlugin ordered chain]:
    handle_node_cleared (.after(NodeSystems::TrackCompletion))
    handle_timer_expired (.after(NodeSystems::TickTimer), .after(handle_node_cleared))
    handle_run_lost (.after(handle_node_cleared), .after(handle_timer_expired))

FixedPostUpdate:
  store_authoritative  [InterpolatePlugin]
  clear_input_actions  [InputPlugin]

PostUpdate:
  interpolate_transform  [InterpolatePlugin]
```
