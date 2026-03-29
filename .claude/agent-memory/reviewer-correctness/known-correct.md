---
name: Confirmed correct patterns — do not re-flag
description: Patterns that look suspicious but are intentionally correct in this codebase
type: project
---

## bolt_wall_collision: velocity snapshot before inner loop

`velocity` is captured as `let velocity = bolt_vel.0` before the candidates
loop. `break` after first wall resolve ensures only one wall is processed per
bolt per frame, so `reflect(velocity, normal)` uses the correct pre-reflect
snapshot. The velocity write `bolt_vel.0 = reflect(velocity, normal)` is correct.

## ccd_sweep_breaker: fallback Aabb2D uses breaker_pos as center

`Aabb2D::new(breaker_pos, expanded_half)` — `Aabb2D::new` stores its first arg
directly as `center`, and `ray_intersect` treats `self.center` as world-space.
Since `breaker_pos` is already world-space, this is correct. Confirmed by reading
`rantzsoft_physics2d/src/aabb.rs`.

## bolt_wall_collision: wall_center = wall_pos.0 + wall_aabb.center

Wall entities store `Aabb2D` with `center` = `Vec2::ZERO` (offset from entity
position) and `Position2D` as world position. So `wall_center = wall_pos.0 +
wall_aabb.center` correctly converts to world-space center. Confirmed: all walls
and cells always spawn with `Aabb2D::new(Vec2::ZERO, half_extents)`.

## bolt_wall_collision: query uses query_aabb_filtered (not query_circle_filtered)

The system uses `quadtree.query_aabb_filtered(&Aabb2D::new(position, Vec2::splat(r)), query_layers)`
for broad-phase. The old memory note saying "query_circle_filtered" was stale.
The AABB query is correct for finding wall candidates within bolt radius.

## RampingDamage: first fire() BUG IS NOW FIXED (Phase 1 cleanup)

The original bug (accumulated: 0.0 on first insert) is fixed. Current code inserts with
`accumulated: damage_per_trigger` and `trigger_count: 1`. Test `multi_call_accumulation_is_linear`
now asserts accumulated=2.0 for 4 calls at 0.5 each. Do NOT re-flag.

## breaker_cell_collision / breaker_wall_collision: quadtree with narrow-phase

Both systems now include narrow-phase AABB overlap checks (dx/dy comparison) after
the broad-phase quadtree query. Not placeholders anymore — confirmed correct.

## bolt_breaker_collision: side-hit reflects only bolt_velocity.x

When `normal.x.abs() > normal.y.abs()` (side hit), only x is negated. This is
correct because cast_circle/ray_vs_aabb produces only axis-aligned normals for
rectangular AABBs. Side-hit normals are always purely `Vec2::X` or `Vec2::NEG_X`.

## bolt_breaker_collision: overlap resolution uses bolt_pos.x for hit_fraction

In the overlap path, `surface.reflect_top_hit(bolt_pos.x, ...)` uses the old x
(before position update). Since only `bolt_position.0.y` was changed (push up),
x is still the correct pre-overlap x for angle calculation. Intentionally correct.

## cell_wall_collision narrow-phase ignores Aabb2D.center offset

The narrow-phase computes `dx = |cell_pos.0.x - wall_pos.0.x|` without adding
`cell_aabb.center` or `wall_aabb.center`. This is correct because all spawned
cells and walls always have `Aabb2D.center = Vec2::ZERO`.

## bolt_cell_collision: piercing only fires if cell would be destroyed

`can_pierce && would_destroy` means piercing only applies to cells the bolt
one-shots. Bolts with piercing remaining still reflect off cells that would
survive. This is the intended mechanic — not a bug.

## maintain_quadtree uses GlobalPosition2D as world AABB center

When inserting into quadtree: `Aabb2D::new(global_pos.0, aabb.half_extents)`.
The stored quadtree AABB uses GlobalPosition2D as center (not Position2D + center
offset). For static entities (walls, cells), GlobalPosition2D == Position2D,
so the narrow-phase `wall_pos.0 + wall_aabb.center` is consistent.

## cleanup_cell: writes CellDestroyedAt before despawn (correct two-phase)

`commands.entity(msg.cell).despawn()` is deferred. `CellDestroyedAt` is written
in the same iteration before despawn executes. Entity is still alive when message
is emitted — correct per two-phase destruction design.

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
