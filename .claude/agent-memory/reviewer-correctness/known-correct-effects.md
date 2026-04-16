---
name: Confirmed correct patterns — do not re-flag (Phase 3–5 effects)
description: Effect system patterns (Active*/Effective*, Phase 4 runtime effects, Phase 5 complex effects) that look suspicious but are intentionally correct
type: project
---

## Active* pattern: silent no-op is intentional (post Effective* cache removal)

`fire()` functions check `world.get_mut::<Active*>()` and silently do nothing if
the component isn't present. There are NO `recalculate_*` systems and NO `Effective*`
components — these were all removed in the Effective* cache-removal refactor (2026-03-30).
`dispatch_chip_effects` is a real system (not a stub) that fires chip effects via
`BoundEffects`/`StagedEffects` — but `Active*` components are only inserted when an
effect's `fire()` actually runs on a bolt or breaker entity.
Consumers call `Active*.multiplier()` / `Active*.total()` on demand. The entire system
is structurally correct and connected end-to-end. Do NOT flag absence of `Effective*`
components or `recalculate_*` systems — they were intentionally removed.

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

## Wave 4 pulse: PulseConfig::fire() guards entity existence — intentional difference from shockwave

`PulseConfig::fire()` returns early if `world.get_entity(entity).is_err()`. Shockwave does NOT
have this guard (it spawns a shockwave entity with defaults even for despawned source). The difference
is correct: pulse installs a persistent `PulseEmitter` on the source entity which must exist; shockwave
spawns a self-contained entity at fire time with no ongoing dependency on the source.

## Wave 4 pulse: single-gate `if timer <= 0.0` (not `while`) — intentional, Behavior #37

`tick_pulse` uses `if emitter.timer <= 0.0 { emitter.timer += emitter.interval; spawn_ring(); }`.
At most one ring per tick regardless of dt. Large dt does NOT burst multiple rings. Test #37 in both
systems.rs and config.rs locks this invariant. Do NOT flag absence of `while` loop as a burst bug.

## Wave 4 pulse: snapshot-at-ring-spawn, not at fire() — correct per D2

`tick_pulse` reads `BoltBaseDamage` and `EffectStack<DamageBoostConfig>` from the emitter entity
fresh each time a ring is spawned (inside the per-emitter loop). `PulseConfig::fire()` does NOT
snapshot these — it only sets `source_chip`. Mid-emitter damage changes affect future rings but not
already-spawned rings. Intentional. Tests #19–#27 cover this.

## tick_pulse / tick_pulse_ring: both use Time::delta_secs() — consistent (Wave 4 rewrite, 2026-04-13)

After the Wave 4 structural separation rewrite, `tick_pulse` and `tick_pulse_ring` both use
`time.delta_secs()`. The prior note about a `timestep()` vs `delta_secs()` inconsistency
is obsolete — it no longer exists. Do NOT re-flag this.

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

## BoltBuilder typestate: build() silent OptionalBoltData drop is NOT a production bug

`build()` terminals in `bolt/builder.rs` silently drop `spawned_by`, `lifespan`, `with_effects`,
`inherited_effects`. But `bolt_params` IS captured in the returned tuple via `build_core()`.
Actually: `build_core()` reads `optional.radius` — so radius IS preserved in `build()`.
But `bolt_params` is only inserted via `spawn_inner()` — so `BoltSpawnOffsetY` etc. are absent
from `build()` output even when `config()` was called.

The test `build_without_from_config_has_no_bolt_params` is NOT a vacuous test — it tests the
no-config path which genuinely has no bolt_params. The with-config `build()` path (lifespan dropped)
has no test, but `build()` has zero production callers. Do NOT flag as active bug.

## BoltBuilder config() radius ordering: .or() semantics are correct

`config()` uses `optional.radius = optional.radius.or(Some(config.radius))`. This preserves
any radius set via `.with_radius()` called BEFORE `.config()`. When `.with_radius()` is called
AFTER `.config()`, it overwrites `optional.radius` (since `with_radius` does `self.optional.radius = Some(r)`
unconditionally). Both orderings are correct and tested.

## BoltBuilder: spawn() sends BoltSpawned even when bolt already exists — intentional

`spawn_bolt` system returns early (sending `BoltSpawned`) when `existing_count > 0`. This is
intentional and tested: `check_spawn_complete` consumes `BoltSpawned` as a spawn-complete
signal regardless of whether a new entity was created.

## attraction::apply_attraction and gravity_well::apply_gravity_pull steering model — CONFIRMED CORRECT (2026-04-01)

Both systems use: `spatial.velocity.0 = (velocity + steering).normalize_or_zero(); apply_velocity_formula(...)`.
Intentional steering model: blend direction then normalize, then scale to base_speed via formula.
Commit "fix: attraction and gravity well use steering model with velocity formula" introduced this intentionally.

`apply_gravity_pull` uses `Res<Time>` in FixedUpdate: correct (acts as Time<Fixed> per confirmed-patterns.md).
`apply_gravity_pull` uses `spatial.position.0` (Position2D) not `global_position.0`: correct for bolts
(root entities, no parent hierarchy). Do NOT re-flag.

## f32::EPSILON matching in reverse() — CONFIRMED CORRECT pattern (2026-04-01)

`(v - value).abs() < f32::EPSILON` in `attraction::reverse()`, `speed_boost::reverse()`,
`anchor/tick_anchor` un-plant. Values pushed verbatim from caller-provided f32 constants —
no arithmetic transformation between push and pop — same bit-pattern guaranteed. Do NOT re-flag.

## circuit_breaker bumps_required=1 immediate reward path — CONFIRMED CORRECT (2026-04-01)

`bumps_required=1`: first call inserts counter with remaining=0, fires reward immediately, resets to 1.
Subsequent calls decrement from 1 to 0, fire reward, reset to 1. Fires on EVERY call. Tested. Correct.

## Wave 2a guard change: last_hit_bolt.is_some() replaces post_hit_timer > 0.0 — CONFIRMED CORRECT (2026-04-13)

`update_bump` (system.rs:100): old guard `post_hit_timer > 0.0` was changed to `last_hit_bolt.is_some()`.
These are semantically equivalent except at the exact tick boundary where the timer ticks from `1/dt` to
exactly `0.0` (clamped). Under the old guard, input block was skipped on the expiry frame even though
`last_hit_bolt` was still set. Under the new guard, `is_some()` is true and retroactive path fires
correctly. The NoBump expiry block (line 123) still runs AFTER the input block — if `last_hit_bolt.take()`
fires in the input block first, `.take()` returns `None` in the expiry block, no duplicate NoBump is sent.
The two fields can diverge only if `post_hit_timer > 0` but `last_hit_bolt.is_none()` — this is an
impossible combination: `post_hit_timer` is only set in `grade_bump` together with `last_hit_bolt`
(lines 175-176), and only cleared together (lines 111-112 or take on 124). No state inconsistency possible.

## Wave 2b: on_node_end_occurred in OnEnter(Teardown).after(cleanup_on_exit) — CONFIRMED CORRECT (2026-04-13)

`.after(cleanup_on_exit::<NodeState>)` is NOT a cross-schedule no-op: both systems are registered in
`OnEnter(NodeState::Teardown)` — `cleanup_on_exit` in `node/plugin.rs:47` and `on_node_end_occurred`
in `effect_v3/triggers/node/register.rs:27-30`. Bevy 0.18 honors `.after()` within the same schedule.
Tests mirror production wiring by registering `cleanup_on_exit::<NodeState>` BEFORE `register::register`
in the same `OnEnter(NodeState::Teardown)` schedule. The `drive_to_teardown` two-step helper correctly
isolates `OnExit(Playing)` from `OnEnter(Teardown)`.

## ShieldActive — ELIMINATED (Shield refactor, 2026-04-02)

`ShieldActive` NO LONGER EXISTS. The charge-based shield mechanism was entirely redesigned.
Shield is now a timed visible floor wall (`ShieldWall` + `ShieldWallTimer`). `bolt_lost` and
`handle_cell_hit` no longer reference `ShieldActive`. Do NOT re-flag the absence of
ShieldActive charge-decrement patterns — the component and its logic were deleted.

See `reviewer-architecture/shield_cross_domain_write.md` for the full elimination record.

## Wave 5 circuit_breaker SpawnBolts + Shockwave double-dispatch — CONFIRMED CORRECT (2026-04-13)

`tick_circuit_breaker` fires `Shockwave` then `SpawnBolts` via `fire_dispatch` inside the
`if counter.remaining == 0` block, before `counter.remaining = counter.bumps_required`.
Both dispatches read from the locally-owned cloned `counter` struct — not from World — so
values are never stale regardless of what either dispatch does to the world. Neither dispatch
touches `CircuitBreakerCounter` on the source entity. The write-back `if let Some(...)` guard
at line 68 is a no-op protection — the entity is never despawned by either dispatch.
Counter wrapping fires SpawnBolts N times per frame with the same stable `spawn_count` each time.
Tested by `wrapping_twice_in_one_frame_fires_spawn_bolts_twice` and adjacent tests.
Do NOT re-flag the fire-order or stale-data concerns.

## Wave 6 TetherBeamWidth required-component query — CONFIRMED CORRECT (2026-04-13)

`tick_tether_beam` requires `&TetherBeamWidth` in its query. Any `TetherBeamSource` entity
spawned without `TetherBeamWidth` is silently skipped. This is the intended required-component
contract — not an Option fallback. Test `beam_without_tether_beam_width_is_silently_skipped_by_query`
explicitly locks this behavior. The only beam-spawning code paths are `fire_spawn` and `fire_chain`
in `config.rs` — both stamp `TetherBeamWidth(self.width.0)`. There is no path that spawns
`TetherBeamSource` without `TetherBeamWidth`. Do NOT re-flag the silent-skip as a bug.

## Wave 6 TetherBeamWidth destructure `&TetherBeamWidth(beam_width)` — CONFIRMED CORRECT (2026-04-13)

`systems.rs:29` pattern `&TetherBeamWidth(beam_width)` on a `&TetherBeamWidth` query result
is valid Rust. `f32` is `Copy`; the pattern deconstructs through the shared reference and
binds `beam_width: f32` by copy. Correct.

## Wave 6 `across <= beam_width` with width=0.0 — CONFIRMED CORRECT (2026-04-13)

`across` is `offset.dot(perp).abs()` which is 0.0 for cells exactly on the beam line.
`0.0 <= 0.0` is true — correctly includes the on-line cell. Test
`width_zero_damages_only_cells_exactly_on_beam_line` covers this. Do NOT re-flag.

## Wave 6 tether_beam_stress.scenario.ron chain:false — CONFIRMED CORRECT (2026-04-13)

The scenario comment says "spawns two free-moving bolts connected by a damaging beam".
`chain: false` means `fire_spawn` (spawn-a-new-bolt mode), which matches the stated intent.
The `width: 10.0` field was added alongside `chain: false` to complete the RON after
`TetherBeamConfig.width` became a required field. Do NOT re-flag chain:false as wrong.

## Wave 7a: EffectStack::retain_by_source semantics — CONFIRMED CORRECT (2026-04-13)

`retain_by_source(source)` calls `self.entries.retain(|(s, _)| s != source)`.
`Vec::retain` keeps entries for which the closure returns `true`. The closure returns `true`
when `s != source` — i.e., keeps entries that do NOT match the source. This is correct:
the method removes all entries whose source equals the argument.

## Wave 7a: RampingDamage::reverse_all_by_source borrow pattern — CONFIRMED CORRECT (2026-04-13)

`reverse_all_by_source` holds `stack` via `world.get_mut()`, calls `stack.retain_by_source()`,
checks `stack.is_empty()`, then calls `world.entity_mut(entity).remove()` while `stack` is
still in scope. This is the same borrow pattern as the existing `reverse()` method (which
has been compiling and passing tests for multiple waves). Bevy's `Mut<T>` is a mutable
reference to world-cell-backed storage — the compiler allows `world.entity_mut()` on a
different component type while `Mut<EffectStack<T>>` is still live because Bevy's safety
model (world cell / unsafe interior mutability) separates these borrows at the type level.
Do NOT re-flag this as a borrow violation.

## Wave 7a: Anchor::reverse_all_by_source uses passed source, not hardcoded "anchor_piercing" — CONFIRMED CORRECT (2026-04-13)

`AnchorConfig::reverse_all_by_source` calls `stack.retain_by_source(source)` where `source`
is the parameter — NOT the hardcoded string "anchor_piercing" that `reverse()` uses.
This is the P2-5 fix: when a chip fires Anchor with a non-"anchor_piercing" source name,
all piercing entries from that source are correctly cleaned up. Test
`reverse_all_by_source_uses_passed_source_not_hardcoded` covers this explicitly.

## Wave 7a: reverse_all_by_source_dispatch — 16-variant match is exhaustive (2026-04-13)

`reverse_all_by_source_dispatch` in `reverse_dispatch.rs` has 16 arms matching all 16
variants of `ReversibleEffectType`. The enum and the dispatch match are in sync.
`fire_dispatch` over `EffectType` (which has more variants) is a different enum and is
not the comparison point. Do NOT flag 16 vs the full EffectType variant count as a bug.

## Waves C/D/E/G2: FIFO remove-then-stage in once.rs — CONFIRMED CORRECT (2026-04-14)

`evaluate_once` for gate-inner case queues `commands.remove_effect(entity, source)` FIRST,
then `commands.stage_effect(...)`. Load-bearing ordering: `RemoveEffectCommand` sweeps both
`BoundEffects` and `StagedEffects` by name. Queuing remove before stage means the outer
`Once` entry is cleaned up without touching the subsequently staged inner subtree. Do NOT
re-flag as wrong ordering.

## Waves C/D/E/G2: TrackArmedFireCommand despawn guard — CONFIRMED CORRECT (2026-04-14)

`TrackArmedFireCommand::apply` guards with `if world.get_entity(self.owner).is_err() { return; }`
before inserting/mutating `ArmedFiredParticipants`. Correct and intentional.

## Waves C/D/E/G2: RemoveStagedEffectCommand isolation — CONFIRMED CORRECT (2026-04-14)

`RemoveStagedEffectCommand::apply` only touches `StagedEffects` — it does NOT sweep
`BoundEffects`. It uses `iter().position()` + `Vec::remove()` for first-match-by-identity
removal. Do NOT flag absence of BoundEffects sweep — this command is specifically for
staged-only teardown.

## Waves C/D/E/G2: source chip propagation in tick_entropy_engine — CONFIRMED CORRECT (2026-04-14)

`chip.and_then(|c| c.0.clone()).unwrap_or_default()` correctly handles all three cases:
- `None` component → `""`
- `Some(EffectSourceChip(None))` → `""`
- `Some(EffectSourceChip(Some("name")))` → `"name"`

## Waves C/D/E/G2: evaluate_on TrackArmedFireCommand queued AFTER evaluate_terminal — CONFIRMED CORRECT (2026-04-14)

`evaluate_on` in `walking/on.rs`: calls `evaluate_terminal(resolved, terminal, source, commands)`
first, then `commands.track_armed_fire(owner, source.to_owned(), resolved)` if `is_armed_source`.
Ordering doesn't matter for correctness (both are deferred); participant tracking on owner is
logically after the fire. Correct.

## Waves C/D/E/G2: 4 watcher systems registered in EffectV3Systems::Bridge — CONFIRMED CORRECT (2026-04-14)

`stamp_spawned_bolts`, `stamp_spawned_cells`, `stamp_spawned_walls`, `stamp_spawned_breakers`
are all imported from `storage` mod and registered in `FixedUpdate` / `EffectV3Systems::Bridge`
in `effect_v3/plugin.rs`. Do NOT flag as missing wiring.

## Wave 5 magnetic: `dist > field.radius` boundary check is intentional (2026-04-15)

`apply_magnetic_fields` uses `if dist > field.radius { continue; }`. This means a bolt at
exactly `field.radius` IS affected (the condition is false, so the bolt is NOT skipped).
Test `bolt_at_radius_boundary_still_affected` in `single_magnet.rs` explicitly locks this behavior.
Do NOT re-flag as an off-by-one error.

## Wave 5 magnetic: cap guard uses `>` not `>=` — intentional (2026-04-15)

The acceleration cap test `acceleration_exactly_at_cap_passes_through` confirms that the cap
uses strict `>` (not `>=`). A force magnitude exactly equal to `max_accel` passes through
without normalization+scaling. This is intentional — equal-to-cap is not over-cap.
Do NOT re-flag the `>` as wrong.

## Wave 5 magnetic: Dead cells excluded via Without<Dead> query filter — correct ECS pattern (2026-04-15)

`MagnetQuery` has `Without<Dead>` in its filter tuple. Dead cells never enter the inner loop at all.
No runtime `if dead { continue; }` check is needed or present. This is the correct ECS approach.
Do NOT flag absence of a runtime dead-check.

## Wave 5 magnetic: PhantomPhase::Ghost suppresses, Telegraph and Solid do not — confirmed (2026-04-15)

`if phantom.is_some_and(|p| *p == PhantomPhase::Ghost) { continue; }` correctly skips only Ghost.
Telegraph and Solid phases allow force through (cells behave as real magnets in those phases).
Tests in `dead_and_phantom.rs` cover all three variants and the no-component case.

## Wave 5 magnetic: Position2D (not GlobalPosition2D) is correct for both magnets and bolts (2026-04-15)

Both magnetic cells and bolts are root entities with no parent hierarchy. `Position2D` is their
world-space position. Using `GlobalPosition2D` would require the spatial hierarchy propagation
to have run — `Position2D` is directly available and equivalent for root entities.
Do NOT re-flag the use of `Position2D` instead of `GlobalPosition2D`.
