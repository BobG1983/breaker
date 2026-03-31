---
name: Confirmed correct patterns — do not re-flag (Phase 3–5 effects)
description: Effect system patterns (Active*/Effective*, Phase 4 runtime effects, Phase 5 complex effects) that look suspicious but are intentionally correct
type: project
---

## Active*/Effective* pattern: silent no-op is intentional

`fire()` functions check `world.get_mut::<Active*>()` and silently do nothing if
the component isn't present. `recalculate_*` systems only match entities with both
`Active*` AND `Effective*`. `dispatch_chip_effects` is a real system (not a stub) that
fires chip effects via `BoundEffects`/`StagedEffects` — but `Active*` components are only
inserted when an effect's `fire()` actually runs on a bolt or breaker entity.
Consumers use `Option<&Effective*>` with `map_or(1.0)` fallback. The entire system
is structurally correct and connected end-to-end.

## Multiplicative stacking in Active*/Effective* — correct by design

`ActiveDamageBoosts.multiplier()` = product of all entries (not sum). Empty vec
returns 1.0. This is correct for the stated design (additive→multiplicative
migration in Phase 3). The `BASE * multiplier` formula in `bolt_cell_collision`
is correct: when no boost, multiplier=1.0, so damage = BASE * 1.0 = BASE.

## apply_attraction: nearest target across ALL types wins — intentional

`apply_attraction` tracks ONE nearest candidate across ALL active attraction types
and applies only that entry's force. Test `apply_attraction_multiple_types_nearest_target_wins`
explicitly asserts this is the intended behavior. Do NOT re-flag as "only one force
applied with multiple active attractions".

## Wall #[require(Spatial2D)] chain — Wall component auto-inserts GlobalPosition2D

`Wall` has `#[require(Spatial2D, CleanupOnNodeExit)]`. `Spatial2D` has
`#[require(GlobalPosition2D, ...)]`. Spawning `Wall` therefore auto-inserts
`GlobalPosition2D`, making it visible to `maintain_quadtree`. The `second_wind`
wall does not need to explicitly add `Spatial2D` because it includes `Wall` in the bundle.

## SecondWind double-despawn on same-frame double-bolt-hit — intentional, tested

`despawn_second_wind_on_contact` may call `commands.entity(wall).despawn()` twice
if two bolts hit the same SecondWindWall in the same frame. The entity query check
passes for both (deferred despawn hasn't flushed yet), so two deferred despawn commands
are queued. In Bevy 0.18, the second despawn is a no-op (logs warning). The test
`despawn_second_wind_wall_two_bolts_same_frame` covers this edge case.

## tick_pulse_emitter uses Time<Fixed>::timestep() — equivalent to Time::delta_secs() in FixedUpdate

In Bevy 0.18 FixedUpdate, `Time<Fixed>::timestep()` and `Time::delta_secs()` produce
the same value. The inconsistency between `tick_pulse_emitter` (using `timestep()`)
and `tick_pulse_ring` (using `delta_secs()`) is cosmetic, not a runtime bug.

## Phase 5 tether_beam: zero-length beam uses origin_inside, not ray_vs_aabb

When both tether bolts share the same position, `beam_vec.length() == 0`, `direction == Vec2::ZERO`,
and `max_dist == 0`. `ray_vs_aabb` with `max_dist=0` always returns `None` (tmin starts at 0,
`tmin <= 0.0` guard triggers). The `origin_inside` check covers this case correctly.
Broadphase AABB for zero-length beam is `expand_by(beam_half_width)` on a degenerate AABB,
correctly producing a square search region. This is correct.

## Phase 5 chain_lightning rework: arcs==0 / range<=0 early returns (REWORKED)

The old `ChainLightningRequest`/`process_chain_lightning` design was replaced with
`ChainLightningChain`/`tick_chain_lightning` sequential arc design.

In the new implementation: `arcs==0` returns immediately (no DamageCell, no chain entity).
`range<=0` also returns immediately. Both are correct early exits. `arcs==1` damages first target
and returns without spawning a chain entity (remaining_jumps would be 0, chain not needed).

**Bug FIXED**: `arc_speed <= 0.0` now triggers an early return in `fire()` (effect.rs line 82-84).
No chain entity is spawned when arc_speed is zero or negative. The permanently-stuck-chain bug
no longer applies.

## Phase 5 entropy_engine: cells_destroyed increments even with empty pool

`entropy_engine::fire()` increments `cells_destroyed` (field on `EntropyEngineState`) before
the empty-pool guard. This means pool changes between node attempts still reflect the correct
cumulative count. Tests `fire_with_empty_pool_increments_cells_destroyed_but_fires_nothing` and
`fire_with_max_effects_zero_fires_nothing` confirm this is intentional.

## Phase 5 piercing_beam: center-distance narrowphase is intentional design

`process_piercing_beam` checks distance from the CELL CENTER to the beam axis (not AABB-vs-beam).
This means a cell whose edge enters the beam but whose center is outside `half_width` is not damaged.
Test `process_piercing_beam_does_not_damage_cell_outside_beam_width` confirms this is the intended design.
Contrast with `tether_beam` which uses Minkowski sum (expand cell AABB by half_width).

## Phase 5 rantzsoft_physics2d::ccd made pub — intentional for tether_beam import

`lib.rs` changed `ccd` from `pub(crate)` to `pub` so `tether_beam.rs` can import
`rantzsoft_physics2d::ccd::ray_vs_aabb`. The prelude already re-exported these items — the
module visibility change is necessary for direct path imports and is correct.

## dispatch_chip_effects: max-stacks continue is FIXED

`dispatch_chip_effects` now has `continue;` after the `add_chip` max-stacks warning (line 57-59).
The old bug (effects dispatched even on max-stack failure) is fixed. Confirmed by test
`chip_at_max_stacks_does_not_dispatch_effects`.

## bypass_menu_to_playing: PendingBreakerEffects FIXED

`bypass_menu_to_playing` now dispatches all four target types (Bolt/Breaker/Cell/Wall) through
`Pending*Effects` resources. `apply_pending_breaker_effects` is registered in `FixedUpdate`
after `tag_game_entities`. Both bugs from the prior review are fixed.

## apply_pending_bolt_effects: FIXED

`apply_pending_bolt_effects` (scenario-runner) now uses `insert_if_new((BoundEffects, StagedEffects))`
before extending, matching the cell/wall variants. Previously it queried `&mut BoundEffects` directly
and silently dropped effects if the component was absent.

## Stat-boost lazy-init: Effective* cache removed in cache-removal refactor

After the cache-removal refactor, `speed_boost`, `damage_boost`, `size_boost`, `bump_force`,
and `piercing` `fire()` functions no longer insert `Effective*` components (they were removed).
They now only lazy-init `Active*` with `insert(Active*::default())` if absent, then push
the value. The old two-step guard is now a single-step guard. Do NOT re-flag the absence
of `Effective*` insertion — it is correct post-refactor.

`quick_stop::fire()` DIFFERS: it does NOT lazy-init `ActiveQuickStops` if absent — it silently
no-ops. This is intentional: QuickStop only applies to entities that already have the component
(breaker spawned with `ActiveQuickStops`). However, no gameplay system reads
`ActiveQuickStops.multiplier()` for actual deceleration — confirmed open gap.

## TetherBeam chain mode: collect-before-despawn in fire_chain is correct

`fire_chain` (tether_beam/effect.rs line 105-111) collects existing `TetherChainBeam` entities
into a `Vec<Entity>` first, then iterates the vec calling `world.despawn()`. This is the
correct collect-before-despawn pattern for direct `&mut World` access. No aliasing issue.

## TetherBeam maintain_tether_chain: deferred despawn during query iteration is safe

`maintain_tether_chain` (tether_beam/effect.rs lines 274-276) iterates `chain_beams` query
and calls `commands.entity(beam_entity).despawn()`. In Bevy 0.18, `Commands` are deferred —
no execution happens during iteration. This is safe.

## TetherBeam chain mode: With<Bolt> query intentionally includes standard tether bolts

`fire_chain` (line 119) and `maintain_tether_chain` (line 265) both query `With<Bolt>` to
find all bolts for chain connection — including standard-mode tether bolts (which also have Bolt+ExtraBolt).
This is the intended design: chain mode connects ALL active bolts.

## SpawnBolts inherit: query_filtered (With<Bolt>, Without<ExtraBolt>) correctly finds primary bolt

`spawn_bolts/effect.rs:27` uses `query_filtered::<&BoundEffects, (With<Bolt>, Without<ExtraBolt>)>()`.
This correctly matches only the primary bolt (has Bolt, does NOT have ExtraBolt). The `.next()`
pick is intentional for the degenerate multi-primary-bolt case.

## ShieldActive charge-decrement: deferred remove + eager in-memory decrement is intentional

`handle_cell_hit` and `bolt_lost` both use the same pattern:
- Decrement `shield.charges` directly (eager, in-memory)
- Queue `commands.entity(...).remove::<ShieldActive>()` when charges reach 0 (deferred)

On the SAME frame's next message for the same entity, the in-memory charges value is already
0, so the `charges > 0` guard fails and subsequent hits fall through to normal damage.
This is correct. The `DamageVisualQuery` field was changed from `Has<ShieldActive>` to
`Option<&'static mut ShieldActive>` to support charge mutation in the system.

`bolt_lost` uses `shield_opt` captured once from `breaker_query.single_mut()`. The loop
over `lost_bolts` re-reads `shield_opt.as_mut().is_some_and(|s| s.charges > 0)` each
iteration — correctly seeing the updated in-memory value from the previous decrement.

Both behaviors are fully tested in `handle_cell_hit/tests/shield_tests.rs` and
`bolt_lost/tests/shield_tests.rs`.
