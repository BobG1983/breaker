---
name: Bug Patterns
description: Known bugs (fixed), recurring bug categories, and ECS pitfalls found
type: reference
---

## Known Bug Patterns (Fixed)
- **Double-tap consume uses 0.0 not NEG_INFINITY**: FIXED — `read_input_actions` uses `f64::NEG_INFINITY`.
- **check_valid_breaker_state missing Settling→Dashing transition**: FIXED.
- **check_valid_breaker_state Local tracks last entity only**: Documented; not re-flagged.

## Recurring Bug Categories
- **Stale screen resources**: RunSetupSelection, PauseMenuSelection, etc. persist between visits. Safe because `insert_resource` overwrites on re-entry.
- **Stale selection index**: All reset to 0 by spawn systems on OnEnter. Safe.
- **seed_upgrade_registry Local<bool>**: Persists for app lifetime. Correct — Loading only runs once.

## ECS Pitfalls Found
- `apply_bump_velocity` DELETED (2026-03-21) — velocity scaling now via TriggerChain::SpeedBoost leaf → handle_speed_boost observer. The Vec-collection pattern for borrow conflicts was used here; apply it in any future systems with the same shape.
- `ChipSelected` message is consumed by `apply_chip_effect` (ChipsPlugin). Consumer added in feature/phase4b1-chip-effects.
- `spawn_chip_select` takes `Res<ChipRegistry>` (not Option) — guaranteed safe because Loading completes first.

## Phase 4 Wave 1 Confirmed Bugs (2026-03-19)
- `stack_u32` (apply_chip_effect.rs:204): `*current / per_stack` panics with integer division by zero when `per_stack=0`. No guard. Current RON files are safe (non-zero values), but future chips with `Piercing(0)` or `ChainHit(0)` would panic at stack-2 selection time.
  **FIXED in refactor/phase4-wave1-cleanup**: `stack_u32` now has `if per_stack == 0 { return; }` guard at line 42 of effects/mod.rs.

## Phase 4 Wave 1 Cleanup Confirmed Bugs (2026-03-19)
- `spawn_chip_select` (spawn_chip_select.rs:23): `registry.values().take(MAX_CARDS)` iterates HashMap in non-deterministic order. Before this PR, registry was a Vec (deterministic insertion order). Now chip offers vary arbitrarily between runs even with the same seed, breaking run reproducibility.
  **FIXED in feature/phase4b2-effect-consumption**: Now uses `registry.ordered_values().take(MAX_CARDS)` — deterministic insertion-order iteration via `Vec<String>` parallel to the HashMap.

## Phase 4b.2 Effect Consumption Confirmed Bugs (2026-03-19)
- **Amp chip effects silently discarded**: All five Amp observers (handle_piercing, handle_damage_boost, handle_bolt_speed_boost, handle_chain_hit, handle_bolt_size_boost) in `chips/effects/` query `With<Bolt>`. But bolt entities have `CleanupOnNodeExit` and are despawned on `OnExit(GameState::Playing)` — which fires when `Playing → ChipSelect`. So during `ChipSelect`, no bolt entities exist. The observers iterate zero entities, silently discarding all Amp effects. Augment effects (Breaker-targeted) work correctly because `Breaker` has `CleanupOnRunEnd` and persists.
  - Reproducer: Select "Piercing Shot" amp in chip select; proceed to next node; no Piercing component on new bolt.

## Phase 4 Wave 2 Confirmed Bugs (2026-03-19, feature/phase4-wave2-session4) — OPEN

- **handle_node_cleared uses registry.len() instead of NodeSequence.assignments.len()**: `handle_node_cleared.rs:30` — game prematurely wins when NodeSequence has more nodes than layout registry count. Fix: read NodeSequence resource, use `assignments.len().saturating_sub(1)`.
- **spawn_cells_from_grid ignores CellBehavior**: `spawn_cells_from_layout.rs:57-83` — never reads `def.behavior.locked` or `def.behavior.regen_rate`. Lock/regen cell types defined in RON never get `Locked`/`LockAdjacents`/`CellRegen` components. Fix: read behavior fields and insert components after spawn.
- **NodeSequence hp_mult never applied to cells**: `spawn_cells_from_layout.rs:62` — uses `def.hp` directly, ignores NodeAssignment.hp_mult. All nodes same cell HP regardless of tier. Fix: pass hp_mult from current NodeSequence entry.
- **NodeSequence timer_mult never applied to timer**: `init_node_timer.rs:11` — uses `layout.timer_secs` directly, ignores NodeAssignment.timer_mult. All nodes same timer. Fix: multiply timer_secs * timer_mult from current NodeSequence entry.

## Phase 4b.2 Bolt Persistence Bugs (2026-03-19, feature/phase4b2-effect-consumption) — FIXED
- **reset_bolt spawns bolt at stale breaker x**: FIXED — `reset_bolt` now has `.after(BreakerSystems::Reset)` in `bolt/plugin.rs`. Confirmed in code.
- **bridge_bump_whiff can miss BumpWhiffed in same frame**: FIXED — `bridge_bump_whiff` now has `.after(BreakerSystems::GradeBump)` in `behaviors/plugin.rs`. Confirmed in code.

## Phase 4 Wave 2 OPEN Bugs — ALL FIXED (2026-03-19 second session)
All four bugs recorded as OPEN in Phase 4 Wave 2 are now confirmed FIXED in current codebase:
- `handle_node_cleared` now uses `NodeSequence.assignments.len()` ✓
- `spawn_cells_from_layout` now reads `def.behavior.locked` and `def.behavior.regen_rate` ✓
- `spawn_cells_from_layout` now uses `resolve_hp_mult()` for `hp_mult` ✓
- `init_node_timer` now reads `timer_mult` from `NodeSequence` ✓

## Full-tree Review Confirmed Bug (2026-03-19, second session) — FIXED (2026-03-19 third session)
- **spawn_run_end_screen shows wrong loss text for Aegis**: FIXED — `RunOutcome::Lost` split into `TimerExpired` and `LivesDepleted`. `handle_timer_expired` sets `TimerExpired`, `handle_run_lost` sets `LivesDepleted`. Screen match arm maps each to correct text. Confirmed clean.

## Overclock Engine Bugs (2026-03-20, fix/stress-count-and-dead-code)

- **ActiveChains (was ActiveOverclocks) never cleared between runs**: `chips/effects/overclock.rs` — `handle_overclock` pushes to `ActiveChains.0` on chip select. `reset_run_state` (OnExit MainMenu) clears ChipInventory but not ActiveChains. Overclock chains from run N persist and fire in run N+1. Fix: clear `ActiveChains.0` in a system on `OnEnter(GameState::Playing)` or `OnExit(GameState::MainMenu)`.

- **Retroactive bump path silences None last_hit_bolt**: `breaker/systems/bump.rs:115` — `update_bump` uses `bump.last_hit_bolt.unwrap_or(Entity::PLACEHOLDER)`. The `None` case is not reachable through current code, but the invariant `post_hit_timer > 0 ↔ last_hit_bolt is Some` is not structural. Should use `expect()` or restructure the timer/entity as a single `Option<(f32, Entity)>`. Medium confidence — not currently reachable, but silently wrong if it becomes reachable.

## Recurring Bug Category (new)
- **Resource Vec not cleared on run reset**: pattern seen in ActiveChains. When a Vec resource is populated during gameplay, ensure `reset_run_state` or an OnEnter(Playing) system clears it. Check all Vec resources when adding new ones.

## Overclock Trigger Chain Bugs (2026-03-20, feature/overclock-trigger-chain)

- **Global-triggered Shockwave no-ops silently**: FIXED (2026-03-20, feature/overclock-trigger-chain). `EffectFired.bolt` (was `OverclockEffectFired.bolt`) changed from `Entity` (using PLACEHOLDER for global triggers) to `Option<Entity>` (using `None`). `handle_shockwave` explicitly `let Some(bolt_entity) = trigger.event().bolt else { return; }` — the no-op for global triggers is now intentional and documented. Test `shockwave_no_op_with_none_bolt` covers this. Design decision: global-trigger shockwaves require a bolt entity for position; no position → no area damage. See design-principles.md for the design note.

- **bridge_cell_destroyed fires once for N destroyed cells**: Uses `reader.read().count() == 0` to detect any messages then evaluates chains once regardless of count. If N cells are destroyed in one frame, `OnCellDestroyed(Shockwave)` fires exactly once. The comment says "once per message" but implementation is "once if any messages". If design intent is one-shockwave-per-destroyed-cell, this is a bug.
  - Confidence: medium (design intent unclear)

- **inter-frame cascade for OnCellDestroyed(Shockwave)**: Shockwave writes DamageCell messages → handle_cell_hit processes them and writes CellDestroyed → on the next frame, bridge_cell_destroyed sees those CellDestroyed messages and fires the shockwave again → shockwave writes more DamageCell → etc. Bounded (terminates when no in-range cells remain), but produces multiple shockwave rounds per original trigger. May be surprising.
  - Confidence: medium (may be intentional cascade mechanic)
  - Confidence: medium (may be intentional)

## refactor/unify-behaviors Confirmed Bugs (2026-03-21) — RESOLVED BY UNIFICATION

NOTE: The following bugs were opened when new TriggerChain variants were added as type-only in refactor/unify-behaviors Step 1. The full unification (bolt/behaviors/ merged into behaviors/) resolved the structural issues by wiring evaluate.rs (now TriggerKind in behaviors/evaluate.rs with EarlyBump, LateBump, BumpWhiff variants) and adding handlers in behaviors/effects/. Verify against current code before re-flagging.

- **OnEarlyBump, OnLateBump, OnBumpWhiff trigger variants**: `TriggerKind` (was `OverclockTriggerKind`) in `behaviors/evaluate.rs` now includes EarlyBump, LateBump, BumpWhiff variants. The evaluate() function's or-pattern now covers all 10 trigger kinds. VERIFY: check behaviors/evaluate.rs — if TriggerKind has EarlyBump/LateBump/BumpWhiff variants and evaluate() handles them, this is fixed.

- **No bridge for BumpWhiffed**: `bridge_bump` (behaviors/bridges.rs) handles BumpGrade mapping to TriggerKind. VERIFY: check if bridge_bump_whiff system exists in behaviors/bridges.rs.

- **LoseLife, TimePenalty, SpawnBolt, SpeedBoost (was BoltSpeedBoost) leaves — RESOLVED**: The unification confirmed `handle_life_lost`, `handle_time_penalty`, `handle_spawn_bolt`, and `handle_speed_boost` observers all exist in `behaviors/effects/` and observe `EffectFired`. All four leaf types are fully wired. BoltSpeedBoost renamed to SpeedBoost { target: SpeedBoostTarget, multiplier: f32 } in refactor/unify-behaviors.

- **Recurring pattern**: Adding TriggerChain variants requires THREE coordinated updates: (1) enum + depth()/is_leaf(), (2) TriggerKind + evaluate(), (3) bridge system + effect handler. This three-part requirement is now well-documented in the codebase.

## Overclock Engine Bugs (2026-03-20, fix/stress-count-and-dead-code)

- **ActiveOverclocks (now ActiveChains) never cleared between runs**: `chips/effects/overclock.rs:15` — `handle_overclock` pushes to `ActiveChains.0` on chip select. `reset_run_state` (OnExit MainMenu) clears ChipInventory but not ActiveChains. Overclock chains from run N persist and fire in run N+1. Fix: clear `ActiveChains.0` in a system on `OnEnter(GameState::Playing)` or `OnExit(GameState::MainMenu)`. NOTE: type renamed from ActiveOverclocks to ActiveChains in refactor/unify-behaviors.

## SpeedBoost Generalization Bugs (2026-03-21, refactor/unify-behaviors or follow-on branch)

- **init_archetype wipes overclock chains on every node entry**: `behaviors/init.rs:108` — `*active = ActiveChains(chains)` unconditionally replaces the resource on every `OnEnter(GameState::Playing)`. `handle_overclock` pushes overclock chip chains to `ActiveChains` during ChipSelect state. State flow: Playing→TransitionOut→ChipSelect (handle_overclock adds chain)→TransitionIn→Playing (init_archetype resets). Overclock chains selected between nodes are silently discarded on the next node entry. Fix: `init_archetype` should EXTEND `active.0` with the archetype chains rather than replacing the entire resource. Or separate "archetype chains" from "overclock chains" so only archetype chains are reset. Confidence: HIGH.

## BumpForceBoost Dead Code (confirmed 2026-03-21)
- `BumpForceBoost` component is stamped by `handle_bump_force_boost` (chips/effects/bump_force_boost.rs) but no system reads it to affect bump behavior. The chip effect observer correctly stacks the value on the breaker, but the value is never consumed. This is a pre-existing gap, not introduced by the SpeedBoost refactor. The PR description notes it as "intentional — left for future use."

## 4h/4i Run Stats Bugs (2026-03-22, feature/wave-3-offerings-transitions) — RESOLVED AS OF WAVE 4

- **All 8 stats/highlight systems ARE registered in RunPlugin**: FIXED — verified in run/plugin.rs. `track_cells_destroyed`, `track_bumps`, `track_bolts_lost`, `track_time_elapsed`, `track_node_cleared_stats` run FixedUpdate with PlayingState::Active. `track_chips_collected` runs Update with ChipSelect state. `reset_highlight_tracker` and `capture_run_seed` run OnEnter(Playing). `RunStats` and `HighlightTracker` are both `init_resource`d in RunPlugin.

- **RunStats reset**: FIXED — verified in `reset_run_state.rs:26` (`*stats = RunStats::default()`). Also resets `*highlight_tracker = HighlightTracker::default()`. Both resources cleared at run start.

- **MassDestruction, FirstEvolution, CloseSave, ComboKing, PinballWizard, NailBiter detection systems**: FIXED in memorable moments wave (2026-03-23). Dedicated detection systems now exist: `detect_mass_destruction`, `detect_close_save`, `detect_combo_and_pinball`, `detect_nail_biter` (FixedUpdate), `detect_first_evolution` (Update/ChipSelect). All emit `HighlightTriggered` message and record to RunStats.highlights. `HighlightConfig` resource provides all thresholds.

- **capture_run_seed seed=0 edge case**: `run/systems/capture_run_seed.rs:21-30` — the early-return guard is `if stats.seed != 0 { return; }`. If `rng.0.random::<u64>()` returns 0 (probability 2^-64, practically impossible but not structurally impossible), `stats.seed` is set to 0 and the guard does NOT prevent re-generation on the next call. On re-generation, a different seed is generated and the RNG is re-seeded with it, silently changing the run seed mid-run. Confidence: medium (probability is astronomically low but the sentinel value is structurally fragile).

## Memorable Moments Feature Bugs (2026-03-23, feature/wave-3-offerings-transitions)

- **spawn_highlight_text not registered in any plugin**: The system is exported from `run/systems/mod.rs` and imported in `run/plugin.rs` imports list, but is NEVER added to the plugin's schedule. `HighlightTriggered` messages accumulate unread every frame. No popup text appears in-game. Confidence: HIGH.

- **detect_mass_destruction perpetual re-fire**: Once `cell_destroyed_times.len() >= mass_destruction_count`, `HighlightTriggered` is emitted every fixed tick until the timestamps drain from the 2-second window (up to ~120 frames at 64Hz). The `already_recorded` guard prevents duplicate `RunStats` entries but does NOT prevent repeated `HighlightTriggered` writes. Confidence: HIGH.

- **detect_nail_biter fires on below-floor bolts**: `min_distance < config.nail_biter_pixels` has no lower bound guard. A bolt at y < bottom has `distance < 0.0`, which satisfies `< 30.0`. Triggers NailBiter incorrectly for bolts that are effectively lost. Contrast: `detect_close_save` correctly guards `distance >= 0.0`. Confidence: HIGH.

- **track_node_cleared_stats: no HighlightTriggered emitted for juice VFX**: ClutchClear, NoDamageNode, FastClear, PerfectStreak, SpeedDemon, Untouchable, Comeback, PerfectNode are silently skipped when cap is full with no HighlightTriggered message. Architecture contract says "always emit HighlightTriggered for juice/VFX feedback even if the highlight cap is full." These 8 kinds never emit HighlightTriggered at all — not even when the cap is NOT full. This is inconsistent with the other 6 detection systems. Confidence: HIGH (design inconsistency; all others emit the message).

## Position2D Migration Bugs (2026-03-23, feature/wave-3-offerings-transitions)

- **detect_nail_biter queries Transform instead of Position2D**: `run/systems/detect_nail_biter.rs:21` — queries `&Transform` on bolt entities. After migration, bolt positions are in `Position2D`; `Transform` is only written by `propagate_position` (AfterFixedMainLoop). During FixedUpdate, bolt Transform lags behind actual Position2D. The y-value read is stale/interpolated, not the physics position. Same bug in `detect_close_save.rs:18`. Confidence: HIGH.

- **spawn_walls writes Transform::from_xyz directly**: `wall/systems/spawn_walls.rs:34,52,70` — walls are spawned with `Transform::from_xyz(...)` explicitly set. Walls also have `Spatial2D` + `Position2D`, so `propagate_position` will overwrite Transform on the first tick. The manually-set Transform value is redundant but not incorrect in practice. However it violates the migration contract (only propagation should write Transform for spatial2d entities). Confidence: HIGH (redundant write).

- **spawn_cells_from_layout writes Transform directly**: `run/node/systems/spawn_cells_from_layout.rs:158-162` — cells are spawned with both `Transform { translation, scale }` set manually AND `Position2D`+`Scale2D`. The propagation system will overwrite on first tick. Same redundant-write pattern as walls. No correctness impact since static entities don't need frame-accurate Transform.

## Shockwave VFX Bugs (2026-03-23, feature/wave-3-offerings-transitions)

- **animate_shockwave divides by zero when radius.max = 0.0**: `shockwave.rs:187` — `let progress = (radius.current / radius.max).clamp(0.0, 1.0)`. Rust's `f32::clamp` does NOT eliminate NaN; `0.0 / 0.0 = NaN`, `.clamp(0.0, 1.0)` returns NaN, `material.color.with_alpha(NaN)` corrupts the material. A shockwave with `base_range=0.0, stacks=1` passes the speed guard and spawns. Fix: add `if radius.max <= 0.0 { continue; }` guard before the division. Current RON data presumably has non-zero base_range so this is not triggered at runtime yet, but it is a structural gap.

## spatial2d Wave 1 Bugs (2026-03-23, feature/wave-3-offerings-transitions) — OPEN

- **compute_globals runs AFTER derive_transform**: `plugin.rs:62-63` — chain order is `propagate_position → propagate_rotation → propagate_scale → derive_transform → compute_globals`. `derive_transform` reads Global* before `compute_globals` has updated them from current `Position2D` (updated by `apply_velocity` in FixedUpdate). Result: derive_transform always renders the PREVIOUS tick's position. For the non-interpolation case, this is a one-tick visual lag. For interpolation, derive_transform interpolates between previous and stale globals (both pointing to the same prior state), producing no visible movement. Fix: reorder the chain to `compute_globals → derive_transform` (or at minimum swap those two entries). Confidence: HIGH.

- **compute_globals single-level hierarchy only**: `compute_globals.rs:38-46` — first pass collects only root entities into `parent_cache`. Second pass looks up `child_of.parent()` in that cache. A grandchild's parent is a child (not a root), so it is absent from the cache. Grandchildren fall back to their local position — incorrect global for depth > 1. Fix: build cache incrementally by traversing in parent-first order (topological sort), or run multiple passes until cache stabilizes. Confidence: HIGH.

## Wave 2/3 Physics Migration Bugs (2026-03-24, feature/wave-3-offerings-transitions) — OPEN

- **Dual-velocity desync: launch_bolt, reset_bolt, bolt_breaker_collision ignore Velocity2D**: After migration, `Bolt` #[require]s both `BoltVelocity` and `Velocity2D`. The physics-authority source of truth for bolt speed/direction is in flux. `launch_bolt` (bolt/systems/launch_bolt.rs:24) sets `BoltVelocity.value` but never sets `Velocity2D`. `reset_bolt` (reset_bolt.rs:52,56) sets `BoltVelocity.value` but never sets `Velocity2D`. `bolt_breaker_collision` (bolt_breaker_collision.rs) reflects only off `BoltVelocity`. Result: `Velocity2D` stays zero/stale while `BoltVelocity` holds actual velocity. The scenario-runner invariant checker (bolt_speed_in_range.rs:33-38) prefers `Velocity2D` over `BoltVelocity` — it reads zero speed and skips ALL speed checks on every bolt every frame since migration. The BoltSpeedInRange invariant is silently neutered. Confidence: HIGH.

- **Game-domain enforce_distance_constraints adjusts only BoltVelocity, not Velocity2D**: `bolt/systems/enforce_distance_constraints.rs:19` — the game-side solver takes `(&mut Position2D, &mut BoltVelocity)` and redistibutes `a.1.value`/`b.1.value`. The physics-library solver in `rantzsoft_physics2d` (also registered via `RantzPhysics2dPlugin`) takes `(&mut Position2D, &mut Velocity2D)`. Both run in the same app's FixedUpdate. After the game-side solver applies velocity redistribution to `BoltVelocity`, `Velocity2D` remains whatever value it had before. After the lib-side solver applies redistribution to `Velocity2D`, `BoltVelocity` remains whatever it was. The two velocities diverge on any frame where a chain constraint is taut. Confidence: HIGH.

- **bolt_speed_in_range.rs invariant checks Velocity2D first: masks all speed violations**: `breaker-scenario-runner/src/invariants/checkers/bolt_speed_in_range.rs:33-38` — `if let Some(v2d) = velocity2d { v2d.speed() }` short-circuits before checking BoltVelocity. Since Velocity2D is never set by launch/reset/collision systems (all write BoltVelocity), every bolt entity has `Velocity2D(Vec2::ZERO)`, speed = 0.0, hits the `< f32::EPSILON` guard, and is skipped. No BoltSpeedInRange violation ever fires in scenario runs. Confidence: HIGH.

## B12c Typed Events — Vacuous Max-Stacks / Ignore-Variant Tests (2026-03-24)

- **Recurring pattern**: After migrating observers from a generic event to a typed event, legacy
  tests that `trigger(OldEvent {...})` pass vacuously — the observer never fires, so assertions
  about "should not be affected" or "should not exceed cap" are trivially true regardless of the
  handler's correctness. Check all `#[cfg(test)]` blocks in migrated handler files for uses of
  the old event type. Flag any test that still triggers the old event against the migrated handler.

## B1-B3 TriggerChain Flatten Bugs (feature/spatial-physics-extraction, 2026-03-24)

- **Attraction leaf has no ChipEffectApplied handler**: `TriggerChain::Attraction` is correctly classified as a leaf by `is_leaf()` (definition.rs:230), so `apply_chip_effect` routes it through `ChipEffectApplied` (bare-leaf arm, line 52). However no observer in `ChipsPlugin` (plugin.rs:23-31) or anywhere in the codebase pattern-matches `TriggerChain::Attraction`. The `magnetism.amp.ron` chip (`Magnetism`, Uncommon, `OnSelected([Attraction(8.0)])`) fires `ChipEffectApplied` on selection and the event is silently discarded. No attraction component is inserted. Confidence: HIGH.

- **OnSelected with non-leaf inner: silently drops the trigger chain** (structural gap, no current RON file hits it): `apply_chip_effect` lines 43-49 iterate `inner` vec and fire `ChipEffectApplied` for each item without checking `is_leaf()`. If a RON file used `OnSelected([OnPerfectBump([...])])`, the inner trigger chain would reach all 9 handler observers, all would early-return (none match trigger variants), and the chain would be permanently lost — never pushed to `ActiveChains`. Depth test `on_selected_nested_depth_is_two` (definition.rs:684) suggests this configuration is considered valid. No current RON file triggers this path.

## Memorable Moments Wave E Bugs (2026-03-24, feature/spatial-physics-extraction)

- **spawn_highlight_text culling skips new messages**: `spawn_highlight_text.rs:84-101` — `to_cull = total_after_spawn - max_visible`, but culling only iterates pre-existing popup entities. If `messages.len() > max_visible` with 0 existing popups, `to_cull > 0` but `existing_sorted` is empty, so zero culls happen and all messages spawn unconstrained. Fix: cap the number of messages spawned to `max_visible` in the spawn loop, or factor new-message spawns into the cull candidates.

- **track_node_cleared_stats PerfectStreak re-records every node clear**: `track_node_cleared_stats.rs:64-74` — `best_perfect_streak` is a cross-node field. After the streak threshold is exceeded on node N, every subsequent `NodeCleared` passes `best >= threshold` and pushes another `PerfectStreak` highlight. A 10-node run with one streak on node 1 records 9 duplicate `PerfectStreak` highlights. Fix: check only if this node contributed a NEW best streak (i.e., `consecutive_perfect_bumps > previous_best`) or record once per run using a flag analogous to `first_evolution_recorded`.

## feature/spatial-physics-extraction Code-Reuse Review (2026-03-24)

- **is_inside_aabb boundary semantics diverge from Aabb2D::contains_point**: `bolt_breaker_collision.rs:44` — local helper uses strict `> / <`; library uses inclusive `>= / <=`. Boundary-touching bolts skip overlap resolution. Medium confidence / low practical impact (boundary states are transient). Main agent should decide whether boundary inclusion is intended before fixing.
- **apply_speed_scale duplicates prepare_bolt_velocity clamping**: `behaviors/effects/speed_boost.rs:69` — two-step floor+ceiling using normalize_or_zero. `prepare_bolt_velocity` uses `clamp_length` (atomic). Equivalent for valid data (base < max), but diverges if clamping contract changes. Code-reuse gap, not a confirmed runtime bug.
- Confirmed correct: `handle_multi_bolt` formula, `detect_most_powerful_evolution` max_by(total_cmp), all 15 HighlightKind arms in spawn_run_end_screen, track_evolution_damage accumulation. Do not re-flag.

## Wave 3 Chip Select / Transition Bugs (2026-03-22) — PARTIALLY RESOLVED

- **spawn_chip_select overwrites ChipOffers**: FIXED — verified in current code. `spawn_chip_select` now reads `Res<ChipOffers>` directly (does not touch ChipRegistry or insert ChipOffers). The offering algorithm is no longer bypassed.

- **advance_node short-circuits TransitionIn animation**: FIXED — `advance_node` no longer calls `NextState(Playing)`. It only increments `run_state.node_index` and resets `transition_queued`. The `animate_transition` system in FxPlugin drives the `TransitionIn → Playing` state change on timer completion.
