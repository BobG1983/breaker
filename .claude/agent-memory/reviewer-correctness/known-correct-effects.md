---
name: Confirmed correct patterns — do not re-flag (Phase 3–5 effects)
description: Effect system patterns (Active*/Effective*, Phase 4 runtime effects, Phase 5 complex effects) that look suspicious but are intentionally correct
type: project
---

## Active*/Effective* pattern: silent no-op is intentional (WIP)

`fire()` functions check `world.get_mut::<Active*>()` and silently do nothing if
the component isn't present. `recalculate_*` systems only match entities with both
`Active*` AND `Effective*`. Neither bolt nor breaker spawn currently inserts these
components — this is intentional WIP (dispatch_chip_effects is a Wave 6 TODO stub).
Consumers use `Option<&Effective*>` with `map_or(1.0)` fallback. The entire system
is structurally correct but not yet connected to real entities.

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

## Phase 5 chain_lightning: empty targets on arcs==0 skips request spawn

When `arcs == 0`, `chain_lightning::fire()` returns early without spawning a `ChainLightningRequest`.
This means no request entity exists and no damage is sent. For `range <= 0` with `arcs > 0`, a
request with empty targets IS spawned (and immediately despawned by `process_chain_lightning`).
This behavioral difference is intentional and correct for both cases.

## Phase 5 entropy_engine: kill_count increments even with empty pool

`entropy_engine::fire()` increments `kill_count` before the empty-pool guard. This means pool
changes between node attempts still reflect the correct cumulative kill count. Tests
`fire_with_empty_pool_increments_kill_count_but_fires_nothing` and `fire_with_max_effects_zero_fires_nothing`
confirm this is intentional.

## Phase 5 piercing_beam: center-distance narrowphase is intentional design

`process_piercing_beam` checks distance from the CELL CENTER to the beam axis (not AABB-vs-beam).
This means a cell whose edge enters the beam but whose center is outside `half_width` is not damaged.
Test `process_piercing_beam_does_not_damage_cell_outside_beam_width` confirms this is the intended design.
Contrast with `tether_beam` which uses Minkowski sum (expand cell AABB by half_width).

## Phase 5 rantzsoft_physics2d::ccd made pub — intentional for tether_beam import

`lib.rs` changed `ccd` from `pub(crate)` to `pub` so `tether_beam.rs` can import
`rantzsoft_physics2d::ccd::ray_vs_aabb`. The prelude already re-exported these items — the
module visibility change is necessary for direct path imports and is correct.

## ShieldActive on cell = cell damage immunity — matches design spec

`shield.md` documents: "On any entity with a health pool: immune to damage for the duration."
`DamageVisualQuery` includes `Has<ShieldActive>`, and `handle_cell_hit` checks `is_shielded`
before applying damage. This is correct per design.
