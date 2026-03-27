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
- **seed_registry systems use idempotency via loaded flag**: `seed_registry::<R>` (in rantzsoft_defaults) tracks loaded state internally — safe to call multiple times per app lifetime. Loading only runs once.

## ECS Pitfalls Found
- `apply_bump_velocity` DELETED (2026-03-21) — velocity scaling now via Effect::SpeedBoost leaf → handle_speed_boost observer. The Vec-collection pattern for borrow conflicts was used here; apply it in any future systems with the same shape.
- `ChipSelected` message is consumed by `dispatch_chip_effects` (ChipsPlugin, C7-R). `apply_chip_effect` was the old consumer (DELETED in C7-R).
- `generate_chip_offerings` (chip select screen) uses `Res<ChipCatalog>` for the chip pool — guaranteed safe because Loading completes first. `ChipCatalog` is populated by `build_chip_catalog` after `ChipTemplateRegistry` and `EvolutionRegistry` are seeded.

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
- **bridge_bump_whiff can miss BumpWhiffed in same frame**: FIXED — `bridge_bump_whiff` now has `.after(BreakerSystems::GradeBump)` in `effect/plugin.rs` (was `behaviors/plugin.rs` before C7-R). Confirmed in code.

## Phase 4 Wave 2 OPEN Bugs — ALL FIXED (2026-03-19 second session)
All four bugs recorded as OPEN in Phase 4 Wave 2 are now confirmed FIXED in current codebase:
- `handle_node_cleared` now uses `NodeSequence.assignments.len()` ✓
- `spawn_cells_from_layout` now reads `def.behavior.locked` and `def.behavior.regen_rate` ✓
- `spawn_cells_from_layout` now uses `resolve_hp_mult()` for `hp_mult` ✓
- `init_node_timer` now reads `timer_mult` from `NodeSequence` ✓

## Full-tree Review Confirmed Bug (2026-03-19, second session) — FIXED (2026-03-19 third session)
- **spawn_run_end_screen shows wrong loss text for Aegis**: FIXED — `RunOutcome::Lost` split into `TimerExpired` and `LivesDepleted`. `handle_timer_expired` sets `TimerExpired`, `handle_run_lost` sets `LivesDepleted`. Screen match arm maps each to correct text. Confirmed clean.

## SeedableRegistry Phase 1 Confirmed Bug (develop, 2026-03-26) — FIXED
- **seed_registry empty-handles false-success**: FIXED — `rantzsoft_defaults/src/systems.rs:142-144` now has `if handles.handles.is_empty() { return Progress { done: 0, total: 1 }; }` after `handles.loaded = true`. The fix is present in current code. Do not re-flag.

## SeedableRegistry Review (develop, 2026-03-26) — FIXED IN feature/seedable-registry
- **build_chip_catalog uses Local<bool> — stale on hot-reload**: FIXED — `propagate_chip_catalog` system now exists in `build_chip_catalog.rs` (under `#[cfg(feature = "dev")]`), wired in `debug/hot_reload/plugin.rs` line 45. Do NOT re-flag.
- **seed_registry: handles.loaded=true set before empty-handles guard** — when the folder resolves to zero typed handles, `loaded=true` is set (line 136) then the `is_empty()` guard returns zero-progress (lines 142-144). On subsequent ticks, the `!handles.loaded` folder-resolution block is skipped but the `is_empty()` guard still fires — the retry loop is correct and permanent. If the folder gains new files at runtime after the first resolution, they are never picked up (handles.loaded=true prevents re-resolution). This is an edge case for hot-reload only, not a loading bug. Confirmed correct for normal loading.
- **propagate_registry update_all semantics**: calls `registry.seed()` which on `BreakerRegistry` uses `assert!(!contains_key(...), "duplicate breaker name")`. If two `.bdef.ron` files have the same `name` field, `update_all` on a hot-reload `Modified` event will panic. This is intentional per the `seed()` documentation.

## Overclock Engine Bugs (2026-03-20, fix/stress-count-and-dead-code)

- **ActiveChains never cleared between runs — RESOLVED (C7-R, 2026-03-25)**: `ActiveChains` → `ActiveEffects` → entirely replaced by `EffectChains` component per entity in C7-R. `dispatch_chip_effects` now pushes to `EffectChains` on bolt/breaker entities (not a global resource). `ChipInventory.clear()` in `reset_run_state` is the only run-reset needed. No global Vec resource to clear.

- **Retroactive bump path silences None last_hit_bolt**: `breaker/systems/bump.rs:115` — `update_bump` uses `bump.last_hit_bolt.unwrap_or(Entity::PLACEHOLDER)`. The `None` case is not reachable through current code, but the invariant `post_hit_timer > 0 ↔ last_hit_bolt is Some` is not structural. Should use `expect()` or restructure the timer/entity as a single `Option<(f32, Entity)>`. Medium confidence — not currently reachable, but silently wrong if it becomes reachable.

## Recurring Bug Category (new)
- **Resource Vec not cleared on run reset**: pattern from ActiveChains era. When a Vec resource is populated during gameplay, ensure `reset_run_state` or an OnEnter(Playing) system clears it. Check all Vec resources when adding new ones.

## Overclock Trigger Chain Bugs (2026-03-20, feature/overclock-trigger-chain)

- **Global-triggered Shockwave no-ops silently**: FIXED (2026-03-20, feature/overclock-trigger-chain). `EffectFired.bolt` (was `OverclockEffectFired.bolt`) changed from `Entity` (using PLACEHOLDER for global triggers) to `Option<Entity>` (using `None`). `handle_shockwave` explicitly `let Some(bolt_entity) = trigger.event().bolt else { return; }` — the no-op for global triggers is now intentional and documented. Test `shockwave_no_op_with_none_bolt` covers this. Design decision: global-trigger shockwaves require a bolt entity for position; no position → no area damage. See design-principles.md for the design note.

- **bridge_cell_destroyed fires once for N destroyed cells**: Uses `reader.read().count() == 0` to detect any messages then evaluates chains once regardless of count. If N cells are destroyed in one frame, `OnCellDestroyed(Shockwave)` fires exactly once. The comment says "once per message" but implementation is "once if any messages". If design intent is one-shockwave-per-destroyed-cell, this is a bug.
  - Confidence: medium (design intent unclear)

- **inter-frame cascade for OnCellDestroyed(Shockwave)**: Shockwave writes DamageCell messages → handle_cell_hit processes them and writes CellDestroyed → on the next frame, bridge_cell_destroyed sees those CellDestroyed messages and fires the shockwave again → shockwave writes more DamageCell → etc. Bounded (terminates when no in-range cells remain), but produces multiple shockwave rounds per original trigger. May be surprising.
  - Confidence: medium (may be intentional cascade mechanic)
  - Confidence: medium (may be intentional)

## refactor/unify-behaviors Confirmed Bugs (2026-03-21) — ALL RESOLVED IN C7-R

The full unification (bolt/behaviors/ merged into behaviors/, then renamed to effect/ in C7-R) resolved all structural issues. `TriggerChain` is entirely deleted from the codebase; the `EffectNode`/`Effect`/`Trigger` tree replaced it. `TriggerKind` deleted; `Trigger` enum used directly. All bridge/effect handlers observe typed per-effect events. Do NOT re-flag any `TriggerChain`-based bugs — the type no longer exists.

## SpeedBoost Generalization Bugs (2026-03-21, refactor/unify-behaviors or follow-on branch)

- **init_archetype wipes overclock chains — RESOLVED (C7-R, 2026-03-25)**: `init_breaker` (was `init_archetype`) no longer touches `ActiveEffects` at all. It pushes `EffectNode` entries to `EffectChains` component on the breaker entity. `dispatch_chip_effects` also pushes to `EffectChains`. Both use `push()` — no replacement occurs. The chip-chains-wiped-on-node-entry bug is structurally impossible with the component-based design.

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

## B1-B3 TriggerChain Flatten Bugs — ALL RESOLVED (C7-R, 2026-03-25)

`TriggerChain` is entirely deleted from the codebase. `ChipEffectApplied` (observer trigger) is also deleted. The chip dispatch pipeline (`dispatch_chip_effects`) now routes via `RootEffect::On` → `EffectNode` → typed per-effect events. `apply_chip_effect` no longer exists. Attraction is wired via `AttractionApplied` typed event and `handle_attraction` in `effect/effects/attraction.rs`. Do NOT re-flag any `TriggerChain`/`ChipEffectApplied`-based bugs.

## Memorable Moments Wave E Bugs (2026-03-24, feature/spatial-physics-extraction)

- **spawn_highlight_text culling skips new messages**: `spawn_highlight_text.rs:84-101` — `to_cull = total_after_spawn - max_visible`, but culling only iterates pre-existing popup entities. If `messages.len() > max_visible` with 0 existing popups, `to_cull > 0` but `existing_sorted` is empty, so zero culls happen and all messages spawn unconstrained. Fix: cap the number of messages spawned to `max_visible` in the spawn loop, or factor new-message spawns into the cull candidates.

- **track_node_cleared_stats PerfectStreak re-records every node clear**: `track_node_cleared_stats.rs:64-74` — `best_perfect_streak` is a cross-node field. After the streak threshold is exceeded on node N, every subsequent `NodeCleared` passes `best >= threshold` and pushes another `PerfectStreak` highlight. A 10-node run with one streak on node 1 records 9 duplicate `PerfectStreak` highlights. Fix: check only if this node contributed a NEW best streak (i.e., `consecutive_perfect_bumps > previous_best`) or record once per run using a flag analogous to `first_evolution_recorded`.

## feature/spatial-physics-extraction Code-Reuse Review (2026-03-24)

- **is_inside_aabb boundary semantics diverge from Aabb2D::contains_point**: `bolt_breaker_collision.rs:44` — local helper uses strict `> / <`; library uses inclusive `>= / <=`. Boundary-touching bolts skip overlap resolution. Medium confidence / low practical impact (boundary states are transient). Main agent should decide whether boundary inclusion is intended before fixing.
- **apply_speed_scale duplicates prepare_bolt_velocity clamping**: `effect/effects/speed_boost.rs` (was `behaviors/effects/speed_boost.rs`) — two-step floor+ceiling using normalize_or_zero. `prepare_bolt_velocity` uses `clamp_length` (atomic). Equivalent for valid data (base < max), but diverges if clamping contract changes. Code-reuse gap, not a confirmed runtime bug.
- Confirmed correct: `handle_multi_bolt` formula, `detect_most_powerful_evolution` max_by(total_cmp), all 15 HighlightKind arms in spawn_run_end_screen, track_evolution_damage accumulation. Do not re-flag.

## Wave 2a (feature/spatial-physics-extraction, 2026-03-25) — UPDATED AFTER CODE-REUSE REVIEW

- **Double-counting cells_destroyed**: RESOLVED — `CellDestroyed` type is completely removed; only `RequestCellDestroyed` (internal) + `CellDestroyedAt` (downstream) exist. No dual-reader path. Do not re-flag.

- **bridge_timer_threshold fires on zero-total timer**: FIXED — `bridges.rs` now explicitly handles `timer.total == 0.0` by assigning `ratio = 0.0`. The zero-total path treats all thresholds as satisfied (ratio < threshold for any positive threshold). This is the documented design: no timer = immediate trigger. Do not re-flag.

- **ActiveDamageBoosts.0 grows past max_stacks cap**: CONFIRMED STILL OPEN — `damage_boost.rs` — `handle_damage_boost` calls `stack_f32` (capped) but always pushes `per_stack` to `ActiveDamageBoosts.0` even after cap is reached. `multiplier()` returns an ever-growing product past `max_stacks`. Until reversal also affected. Confidence: HIGH.

- **bridge_bump skips breaker EffectChains when BumpPerformed.bolt is None**: NEW (2026-03-25) — `bridges.rs:72-74` — `let Some(bolt_entity) = performed.bolt else { continue; }` exits the entire loop iteration including the breaker entity `EffectChains` evaluation block at lines 130-138. A retroactive bump with `bolt: None` silently skips any `OnBump` / `OnPerfectBump` chip on the breaker. Regression spec hint written. Confidence: HIGH.

- **chains_query missing With<Bolt> filter**: NEW (2026-03-25) — `bridge_cell_impact`, `bridge_breaker_impact`, `bridge_wall_impact` all use `Query<&mut EffectChains>` with no entity filter. Access is via `.get_mut(bolt_entity)` today (safe), but query matches cell entities with `EffectChains` too. Latent risk if code ever iterates the query. Medium priority.

- **Until machinery not end-to-end wired** (observation): `tick_until_timers`, `check_until_triggers`, `apply_speed_boosts`, `reverse_children` all function correctly but are inert in production (no code inserts `UntilTimers`/`UntilTriggers` outside tests). This is Wave 2b scope. Do not re-flag.

## Wave 2a Confirmed Correct Patterns (2026-03-25)
- `bridge_cell_death` / `cleanup_destroyed_cells` ordering: cleanup runs `.after(EffectSystems::Bridge)`. Entity lives through bridge evaluation. Correct.
- `bridge_timer_threshold` index-based removal in reverse order: correct.
- `BoltLostWriters` `Result<MessageWriter<RequestBoltDestroyed>>` fallback: the `Err` arm (legacy despawn) runs only when `RequestBoltDestroyed` is not registered; in production it is always registered. Low-impact design choice, not a bug.
- `apply_speed_boosts` empty-vec product = 1.0: idempotent at base speed. Correct.
- `bridge_cell_death` + `bridge_bolt_death` near-duplicate structure: intentional; no shared helper. Do not re-flag as wrong — flag as code-reuse gap only.

## Wave 3 Chip Select / Transition Bugs (2026-03-22) — PARTIALLY RESOLVED

- **spawn_chip_select overwrites ChipOffers**: FIXED — verified in current code. `spawn_chip_select` now reads `Res<ChipOffers>` directly (does not touch ChipRegistry or insert ChipOffers). The offering algorithm is no longer bypassed.

- **advance_node short-circuits TransitionIn animation**: FIXED — `advance_node` no longer calls `NextState(Playing)`. It only increments `run_state.node_index` and resets `transition_queued`. The `animate_transition` system in FxPlugin drives the `TransitionIn → Playing` state change on timer completion.

## C7-R RootEffect Migration Bugs (2026-03-26, refactor/rantzsoft-prelude-and-defaults) — FIXED IN feature/seedable-registry

- **Trigger::Impacted(*), Trigger::Died, Trigger::DestroyedCell have no bridge systems**: FIXED — All three now have bridge modules in `effect/triggers/`: `impacted.rs` (bridge_cell_impacted, bridge_breaker_impacted, bridge_wall_impacted), `died.rs` (bridge_bolt_died, bridge_cell_died), `destroyed_cell.rs` (bridge_destroyed_cell). All registered in `triggers/mod.rs`. Do NOT re-flag.

## feature/seedable-registry New Bugs (2026-03-27)

- **Effect::Explode missing from enum**: `powder_keg.chip.ron:11` references `Do(Explode(range: 48.0, damage_mult: 1.0))` but `Effect` in `effect/definition/types.rs` has no `Explode` variant. Deserialization will fail at runtime when `powder_keg.chip.ron` is loaded. The design doc `docs/design/effects/explode.md` says "Status: Not yet implemented." Fix: add `Effect::Explode { range: f32, damage_mult: f32 }` variant and handler, OR remove `powder_keg.chip.ron` until Explode is implemented. Confidence: HIGH.

- **split_decision.evolution.ron RON syntax error**: `split_decision.evolution.ron:8-9` — `When(...)` and `On(...)` are each closed with `],` (closing the `then` array) but the tuple struct closing `)` is missing. Valid pattern (from cascade.chip.ron) is `]),`. Fix: change line 8 from `],` to `]),` and line 9 from `],` to `]),`. Confidence: HIGH.

- **Evolution recipe ingredient names never match held chips (pre-existing, perpetuated)**: All evolution RON files (including new `split_decision.evolution.ron`) use the template `name` field (e.g. `"Splinter"`, `"Piercing Shot"`) as `chip_name` in ingredients. But `expand_chip_template` produces expanded names (`"Minor Splinter"`, `"Basic Piercing Shot"`, etc.). `eligible_recipes` compares via `inventory.stacks(chip_name)` — bare template names have 0 stacks forever. All evolution recipes are permanently ineligible. Pre-existing in `entropy_engine.evolution.ron`. Confidence: HIGH.

- **dispatch_chip_effects passive Do ignores target field (medium confidence)**: `dispatch_chip_effects.rs:46-52` calls `fire_passive_event(eff, max_stacks, name, &mut commands)` without passing `target`. `SizeBoostApplied` is observed by both `handle_bolt_size_boost` (With<Bolt>) and `handle_width_boost` (With<Breaker>), so `Effect::SizeBoost` always applies to both entity types regardless of the declared `target` field. The `target` field on `RootEffect::On` is semantically irrelevant for passive `Do` children. May be intentional dual-application design — main agent should verify design intent before filing as bug.
