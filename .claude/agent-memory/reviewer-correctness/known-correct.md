---
name: Confirmed correct patterns — do not re-flag (Phase 1 collision)
description: Phase 1 collision system patterns that look suspicious but are intentionally correct in this codebase
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

## handle_kill<Cell>: writes Destroyed<Cell> before despawn (correct two-phase)

`CellDestroyedAt` and `cleanup_cell` no longer exist — replaced by the unified death pipeline.
`handle_kill<Cell>` writes `Destroyed<Cell>` and enqueues `DespawnEntity` via deferred commands.
Entity is still alive when `Destroyed<Cell>` is emitted — `process_despawn_requests` runs after
in `FixedPostUpdate`. Consumers of `Destroyed<Cell>` (effect bridges, node tracking, lock release)
all read before despawn executes — correct per two-phase destruction design.
**Location**: `breaker-game/src/shared/death_pipeline/systems/system.rs`

## ClampRange::apply: min before max ordering is correct

`ClampRange::apply` applies min clamp first (floor), then max clamp (ceiling).
When `min > max`, the result is `max` (value is raised to min, then lowered to max).
No validator enforces `min <= max` but no production data has inverted ranges.

## dispatch_chip_effects: Target::Breaker direct dispatch uses With<Breaker> only (no PrimaryBreaker)

`dispatch_chip_effects` uses `targets.breakers: Query<Entity, With<Breaker>>` without
`With<PrimaryBreaker>`. All chip RON files use `On(target: Breaker, ...)` at the top level,
which goes through `dispatch_chip_effects`'s direct dispatch — never `ResolveOnCommand`.
This path is CORRECT and does NOT require `PrimaryBreaker`.

## check_breaker_position_clamped omits NodeScalingFactor — matches move_breaker exactly

`check_breaker_position_clamped` computes `effective_half_width = base_width.half_width() * size_boosts.multiplier`.
`move_breaker/system.rs` (line 66-67) uses the identical formula for position clamping, with no NodeScalingFactor.
`breaker_cell_collision` and `breaker_wall_collision` DO use NodeScalingFactor for their AABB — but that is collision
detection, not position clamping. The checker mirrors what move_breaker enforces. Do NOT re-flag the omission of NodeScalingFactor.

## check_breaker_count_reasonable: double-gate (playing_gate + internal gate) is intentional

In plugin.rs, `checkers_c` has `.run_if(playing_gate)` which only passes when `stats.entered_playing == true`.
`check_breaker_count_reasonable` ALSO has its own `entered_playing` early-return gate. The double-gate is
intentional: the internal gate allows isolated unit tests (without playing_gate in place) to still respect
the entered_playing flag. In the full scenario runner, `ScenarioStats` is always present and both gates agree.

## check_breaker_count_reasonable: fires when ScenarioStats absent (None) by design

When `stats` is `Option<ResMut<ScenarioStats>>` and is `None`, the early-return gate is skipped and the checker runs.
This only occurs in unit tests (full runner always has ScenarioStats via init_resource). Tested in
`fires_when_scenario_stats_absent_and_zero_breakers`. Do NOT flag as missing gate.

## breaker_cell_collision / breaker_wall_collision: single() is correct with ExtraBreaker undefined

Both systems call `.single()` on `Query<..., With<Breaker>>`. `ExtraBreaker` is defined but
never inserted on any entity — only one `Breaker` entity exists. `.single()` returns `Ok`.
If `ExtraBreaker` is later used (spawns a second `Breaker`-marked entity), `.single()` will
return `Err` and both systems silently skip. This is a future concern, not a current bug.
