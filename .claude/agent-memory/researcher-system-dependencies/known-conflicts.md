---
name: known-conflicts
description: Known query conflicts, ordering issues, and missing constraints identified in the brickbreaker system map (as of 2026-03-17, post-hot_reload addition)
type: reference
---

# Known Conflicts and Ordering Issues

Last updated: 2026-03-17 (feature/scenario-coverage-expansion — deep scan: 9 new invariant systems catalogued; enforce_frozen_positions ordering gap confirmed LOW severity; check_valid_state_transitions uses ResMut<PreviousGameState> not Local — no conflict; new scenarios use only new InvariantKind variants, no data access conflicts)

---

## HOT-RELOAD — cross-frame chain: propagate_breaker_defaults writes BreakerConfig → propagate_breaker_config reads it next frame

`propagate_breaker_defaults` (PropagateDefaults set) writes `ResMut<BreakerConfig>` directly.
`propagate_breaker_config` (PropagateConfig set) is gated by `run_if(resource_changed::<BreakerConfig>)`.

Both run in Update. PropagateConfig runs `.after(PropagateDefaults)`.

**Within one frame:** `resource_changed` is evaluated by Bevy before systems run (as a run condition).
If `propagate_breaker_defaults` modifies `BreakerConfig` in frame N, the change flag is set.
`propagate_breaker_config` runs after it in the SAME frame (PropagateConfig is ordered after PropagateDefaults),
BUT whether `resource_changed` sees the mutation depends on when the flag is reset.

**Key behavior:** Bevy's `resource_changed` checks the `last_changed` tick vs the system's `last_run` tick.
`propagate_breaker_defaults` writes `ResMut<BreakerConfig>` directly (not via Commands), so the change
is immediate and synchronous within the same frame. `propagate_breaker_config` runs in the same frame
in PropagateConfig (which is ordered after PropagateDefaults). The run condition evaluates against
the pre-run `last_changed` tick. Since the write happened earlier in the same frame (earlier tick),
`resource_changed` WILL detect it.

**Verdict:** The chain IS reliable within a single frame. The set ordering `.after(PropagateDefaults)`
guarantees `propagate_breaker_defaults` completes before `propagate_breaker_config` evaluates its
run condition. Direct resource write (not Commands) means no deferred flush needed.

**Note:** `propagate_bolt_defaults` uses `commands.insert_resource` (deferred via Commands), so
BoltConfig is written at command-flush time (end of update), and `propagate_bolt_config` which is
also conditioned on `resource_changed::<BoltConfig>` will NOT see it in the same frame — it sees
it the NEXT frame. This is a one-frame delay for the bolt path vs. zero-frame delay for the breaker
path. See LOW SEVERITY note below.

---

## HOT-RELOAD LOW SEVERITY — bolt/cell/playfield/input/timerui/mainmenu/chipselect defaults use Commands.insert_resource (one-frame delay)

These 7 systems all use `commands.insert_resource(...)` (deferred):
- `propagate_bolt_defaults` → `BoltConfig`
- `propagate_cell_defaults` → `CellConfig`
- `propagate_playfield_defaults` → `PlayfieldConfig`
- `propagate_input_defaults` → `InputConfig`
- `propagate_timer_ui_defaults` → `TimerUiConfig`
- `propagate_main_menu_defaults` → `MainMenuConfig`
- `propagate_chip_select_defaults` → `ChipSelectConfig`

Since these use Commands (deferred), the resource is not updated until command flush (PostUpdate).
`propagate_bolt_config` in PropagateConfig (same frame, same Update) will NOT see the new BoltConfig
value — `resource_changed::<BoltConfig>` checks against the pre-flush state.

**Consequence:** Bolt/cell/etc. component propagation is delayed by one frame relative to the asset
modification. For hot-reload purposes (human-scale edits), one frame (~16ms) is imperceptible.

**Vs. breaker path:** `propagate_breaker_defaults` writes `ResMut<BreakerConfig>` directly, so
`propagate_breaker_config` fires in the same frame.

**Severity:** Low. Hot-reload is for development tooling — one extra frame of latency is fine.
Not worth fixing unless the asymmetry causes confusion.

---

## HOT-RELOAD NO CONFLICT — propagate_archetype_changes writes BreakerConfig AND ActiveBehaviors in same system

`propagate_archetype_changes` holds both `ResMut<BreakerConfig>` and `ResMut<ActiveBehaviors>`.
No other system in PropagateDefaults touches either of these resources (the breaker defaults system
also takes `ResMut<BreakerConfig>`, but they are UNORDERED within PropagateDefaults).

**Potential Bevy conflict:** `propagate_breaker_defaults` and `propagate_archetype_changes` both take
`ResMut<BreakerConfig>`. They are in the same PropagateDefaults set with no ordering between them.
Bevy will serialize them (cannot run in parallel) due to the mutable resource access.

**Severity:** None — Bevy handles this correctly. Serialization means no race. In practice, both
systems are event-gated (they only act when their respective AssetEvents fire), and it would be
unusual for a breaker defaults change and an archetype change to arrive in the exact same frame.
If they do, the last one to run wins. The run order within PropagateDefaults is non-deterministic
but the effect is stable (both write the same BreakerConfig structure from their respective sources).

**Note:** No ordering constraint is needed or recommended — the asset events that trigger each system
come from different file-watching sources.

---

## HOT-RELOAD NO CONFLICT — PropagateDefaults systems within the same set are unordered but independent

The 11 systems in PropagateDefaults are unordered relative to each other (except BreakerConfig
mutable access forcing serialization between propagate_breaker_defaults and propagate_archetype_changes).

Each system guards itself with an asset event check and returns early if no matching Modified event
was seen. They act on disjoint resources (BoltConfig, BreakerConfig, CellConfig, PlayfieldConfig,
InputConfig, TimerUiConfig, MainMenuConfig, ChipSelectConfig, CellTypeRegistry, NodeLayoutRegistry,
ArchetypeRegistry/ActiveBehaviors). No logical dependency between them.

---

## HOT-RELOAD ORDERING NOTE — init_bolt_params/init_breaker_params run in OnEnter, propagate_config runs in Update

`init_bolt_params` runs `OnEnter(GameState::Playing)` and uses `Without<BoltBaseSpeed>` filter.
`propagate_bolt_config` runs in `Update` and has NO filter — it always overwrites.

These run in different schedules with no temporal overlap concern:
- OnEnter fires once when state transitions. By the time Update runs, OnEnter is complete.
- propagate_bolt_config only fires when BoltConfig resource_changed is true — which is NOT true
  just because OnEnter ran (OnEnter does not mutate BoltConfig).

**No conflict.** The `Without<BoltBaseSpeed>` on init_bolt_params is a safety guard for
bolt respawn (re-entering the state), not a conflict with propagate_bolt_config.

**Key asymmetry:** init_bolt_params is "stamp if missing"; propagate_bolt_config is "force overwrite".
If a user edits bolt.ron while Playing, propagate_bolt_config will overwrite components stamped by
init_bolt_params. This is the intended hot-reload behavior.

---

## HOT-RELOAD NO CONFLICT — propagate_cell_type_changes and propagate_node_layout_changes registry conflict

`propagate_cell_type_changes` writes `ResMut<CellTypeRegistry>`.
`propagate_node_layout_changes` reads `Res<CellTypeRegistry>` (for spawning cells after layout change).

Both are in PropagateDefaults with no ordering between them.

**Potential issue:** If both fire in the same frame (a cell type and a node layout were both modified),
`propagate_node_layout_changes` might use a stale CellTypeRegistry (if it runs before
`propagate_cell_type_changes`).

**Severity:** Low. This is a rare race (two different RON file saves in the same 16ms window during
development). The result would be that the respawned cells use the OLD cell type definitions for one
frame. On the next frame, if the cell type change fires again, the cells would be corrected.
In practice, Bevy's file watcher coalesces rapid saves, making simultaneous firing very unlikely.

**Fix (optional):** Add explicit ordering within PropagateDefaults:
`propagate_cell_type_changes.before(propagate_node_layout_changes)`
This would guarantee the registry is always up-to-date when layouts are respawned.

---

## HOT-RELOAD REGISTRY CONFLICT — gameplay systems read registries that hot-reload writes in Update

The following gameplay systems read registries that hot-reload systems write:
- `spawn_cells_from_layout` (RunPlugin, OnEnter) reads `Res<CellTypeRegistry>` — hot-reload writes it in Update
- `set_active_layout` (RunPlugin, OnEnter) reads `Res<NodeLayoutRegistry>` — hot-reload writes it in Update
- `handle_node_cleared` (RunPlugin, FixedUpdate) reads `Res<NodeLayoutRegistry>`
- `bridge_bump` / `bridge_bolt_lost` (BehaviorsPlugin, FixedUpdate) check `Res<ActiveBehaviors>` — hot-reload writes it in Update

**No conflict** in the strict ECS sense — Update and FixedUpdate run in different schedule slots.
OnEnter also does not overlap with Update.

**Behavioral note:** If the registry is updated in Update frame N, FixedUpdate may run 0 or more
times before the next Update. The registry change is visible to FixedUpdate from the next FixedUpdate
tick after Update frame N. The latency is at most 1 fixed tick (~16ms at 60Hz) — imperceptible.

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

---

## SCENARIO RUNNER — ScenarioLifecycle FixedUpdate systems (added 2026-03-17)

These systems run in `FixedUpdate` alongside gameplay systems (but are in `breaker-scenario-runner`, not `breaker-game`).

### Lifecycle group (chained, unordered vs gameplay):
```
tick_scenario_frame → inject_scenario_input → check_frame_limit   [.chain()]
```

### Invariant check systems (unordered, run independently):
- check_bolt_in_bounds, check_bolt_speed_in_range, check_bolt_count_reasonable
- check_breaker_in_bounds, check_no_nan, check_timer_non_negative
- check_valid_state_transitions, check_valid_breaker_state
- check_timer_monotonically_decreasing, check_breaker_position_clamped
- check_physics_frozen_during_pause, check_no_entity_leaks
- enforce_frozen_positions, tag_game_entities

### tag_game_entities also runs in OnEnter(GameState::Playing) — no conflict (different schedule).

---

## RESOLVED — inject_scenario_input now orders .before(BreakerSystems::Move)

**Status: RESOLVED** (fixed in feature/scenario-coverage-expansion)

`inject_scenario_input` writes `ResMut<InputActions>` in `FixedUpdate`.
`move_breaker` and `update_bump` read `Res<InputActions>` in `FixedUpdate`.

The lifecycle chain now has the correct constraint:
```rust
(tick_scenario_frame, inject_scenario_input, check_frame_limit)
    .chain()
    .before(breaker::breaker::sets::BreakerSystems::Move),
```
Injected input is guaranteed to be written before `move_breaker` and `update_bump` consume it in the same tick.

---

## NEW NOTE — bolt_breaker_collision upward-bolt guard moved to top of function

**Status: No conflict — behavioral change to track**

Previously the guard "only reflect if bolt is moving downward" was INSIDE the top/bottom hit branch.
Now it is at the TOP of the per-bolt loop body (before any face-type check). Side hits by upward-moving
bolts are now also skipped — previously they were reflected.

This is an intentional physics correctness fix: a bolt moving upward that clips the breaker's side
should not be deflected (it is on its way up from a bump). The new tests `upward_bolt_side_hit_is_not_reflected`
and `downward_bolt_side_hit_is_reflected` document this behavior in `bolt_breaker_collision.rs`.

No ordering issue. No ECS conflict. Data flow unchanged (BoltHitBreaker message still sent only on
reflection). Just document so future analysis does not flag the changed guard logic as a regression.

---

## NEW NOTE — toggle_pause now routes through InputActions

**Status: No conflict — routing change to track**

`toggle_pause` (PauseMenuPlugin, Update) previously read `Res<ButtonInput<KeyCode>>` directly for
the Escape key. It now reads `Res<InputActions>` and checks `GameAction::TogglePause`.

`GameAction::TogglePause` is produced by `read_input_actions` (InputPlugin, PreUpdate) when Escape
is pressed. `toggle_pause` reads the populated `InputActions` in Update.

The execution order is: PreUpdate (`read_input_actions`) → Update (`toggle_pause`) — unchanged
and correct. The scenario runner also maps `TogglePause` correctly in the action table.

ChaosMonkey in the scenario runner now includes `TogglePause` in its `GAMEPLAY_ACTIONS` pool.
This means chaos scenarios can inject random pause/unpause events. The `check_physics_frozen_during_pause`
invariant validates that physics stops during pause — this is now exercised by chaos scenarios.

---

## LOW — enforce_frozen_positions needs explicit .after(PhysicsSystems::BoltLost)

**Status: Low — functionally correct by accident, implicit ordering**

`enforce_frozen_positions` writes `&mut Transform` on entities with `ScenarioPhysicsFrozen`.
Physics systems write `&mut Transform` on bolt entities.
When `ScenarioPhysicsFrozen` is present it is on a bolt entity (same archetype as `Bolt`).
Bevy serializes these. The correct execution order is physics first, then pin-reset.
No explicit `.after(PhysicsSystems::BoltLost)` constraint is declared.

**Fix (optional):** Add `enforce_frozen_positions.after(PhysicsSystems::BoltLost)` in `lifecycle.rs:140`.
