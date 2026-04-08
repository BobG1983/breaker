---
name: Open and latent bug patterns
description: OPEN and LATENT bugs confirmed in this codebase — check these actively during review; do NOT re-flag RESOLVED items (see bug-patterns-resolved.md)
type: project
---

## rantzsoft_stateflow orchestration tests 8/9 vacuous assertion — OPEN (2026-04-03)

`out_transition_sends_state_changed_after_state_change` and
`out_transition_sends_transition_end_after_state_change` in
`rantzsoft_stateflow/src/transition/orchestration/tests/out_transition_tests.rs` check
`iter_current_update_messages().next().is_some()` on the last of 10 updates.
Since messages are frame-scoped and the transition completes by update ~4-5,
there are no messages on update 10. These tests would assert false and FAIL
even for a correct implementation. Need a `MessageLog` capture-system approach.

## check_aabb_matches_entity_dimensions: false positive for breakers in non-1.0 EntityScale layouts — OPEN

`check_aabb_matches_entity_dimensions` computes `expected = width.half_width() * scale` for breakers
and `expected = Vec2::splat(BoltRadius.0)` for bolts (no scale applied to bolt check).

The stored `Aabb2D` on both bolt and breaker entities is NEVER updated when `EntityScale` changes.
`apply_entity_scale_to_bolt` and `apply_entity_scale_to_breaker` only insert `EntityScale` —
neither touches `Aabb2D`. Physics systems compute live AABB from `BoltRadius * scale` /
`BreakerWidth * scale` directly.

Result: for breakers in layouts with `entity_scale != 1.0` (e.g., `boss_arena.node.ron` has `entity_scale: 0.7`),
the checker fires false-positive `AabbMatchesEntityDimensions` violations.
Bolt invariant is also wrong for scaled layouts (uses scale-1.0 semantics).

**Status**: OPEN — confirmed on scenario-coverage branch review 2026-03-30.
Scenarios using Corridor (entity_scale=1.0) are not affected.

## ActiveQuickStops: fire() is no-op when component absent; no consumer reads multiplier — OPEN

`quick_stop::fire()` silently no-ops if `ActiveQuickStops` is absent (unlike all other stat boosts
which lazy-init). Neither `move_breaker` nor `dash::handle_braking` queries `ActiveQuickStops` to
scale their deceleration. `MovementQuery` and `DashQuery` do not include `ActiveQuickStops`.
The `QuickStop` effect fires but its multiplier is never applied to actual deceleration.

**Status**: OPEN — confirmed on cache-removal refactor review 2026-03-30.
Location: `effect/effects/quick_stop.rs` (fire fn), `breaker/queries.rs` (MovementQuery),
`breaker/systems/move_breaker.rs`, `breaker/systems/dash/system.rs`.

## circuit_breaker::fire() u32 underflow when bumps_required == 0 — OPEN latent bug

`circuit_breaker/effect.rs:73`: `let remaining = config.bumps_required - 1` where `bumps_required`
is `u32`. If `bumps_required == 0`, this panics in debug or wraps to `u32::MAX` in release.
Current production RON uses `bumps_required: 3` — no current trigger.

**Status**: OPEN latent. Not triggered by current data. Needs `bumps_required == 0` guard.
Location: `breaker-game/src/effect/effects/circuit_breaker/effect.rs:73`

## MirrorProtocol::fire() wastes RNG call with dead random velocity — latent design issue

`mirror_protocol/effect.rs:73-78`: generates `random_velocity` via `rng.random_range(0..TAU)` and
passes it to bolt builder, but line 85 immediately overwrites with `mirror_vel`. The random call
advances RNG state for no observable effect. Other systems sharing `GameRng` will see different
values if MirrorProtocol fires. Not a correctness bug for the mirror bolt itself, but affects
RNG determinism for other game systems.

**Status**: OPEN design issue — the extra RNG call has no gameplay effect on the mirror bolt.

## BoltBuilder::build() silently drops OptionalBoltData — latent API hazard, not active bug

The four `build()` terminal impls in `breaker-game/src/bolt/builder.rs` silently drop:
`spawned_by`, `lifespan`, `with_effects`, `inherited_effects`.
`bolt_params` IS captured via `build_core()`. But `BoltSpawnOffsetY` etc. are absent from
`build()` output even when `config()` was called.

**Status**: Latent hazard — safe to leave as-is unless `build()` gains production callers.
(`build()` has zero production callers — all production sites use `spawn()`.)

## BreakerBuilder: with_width() does not recompute min_w/max_w — LATENT BUG

`core_params_from` reads `min_w: dims.min_w` always from `HasDimensions`, not from the
overridden width. When `.with_width(200.0)` is called, the entity gets `BaseWidth=200.0`,
`MinWidth=60.0`, `MaxWidth=600.0`. The min/max are stale relative to the new base width.

**Status**: LATENT — no production callers of `.with_width()` in current RON-driven flow.
**Location**: `breaker-game/src/breaker/builder/core.rs` (core_params_from)

## BreakerBuilder: with_lives()/with_color() before definition() are silently overwritten — LATENT BUG

`.definition()` unconditionally overwrites `optional.lives` and `optional.color_rgb`.
So `.with_lives(Some(5)).definition(...)` silently ignores the `with_lives` call.
Same for `.with_color([...]).definition(...)`.
Valid call order is `.definition()` FIRST, then `.with_*()`. Production follows this order.

**Location**: `breaker-game/src/breaker/builder/core.rs` (definition method)

## BreakerBuilder: rendered() before definition() uses wrong color — LATENT BUG

`.rendered(meshes, materials)` reads `self.optional.color_rgb` at call time.
If called before `.definition()`, `color_rgb` is `None` and falls back to default.
Then `.definition()` sets `optional.color_rgb` — but the material was already created with the wrong color.

**Status**: LATENT — production call order is `.definition(def).rendered(...).primary()`, which is correct.
**Location**: `breaker-game/src/breaker/builder/core.rs` (rendered method)

## WallBuilder: Lifetime field is never consumed in build()/spawn() — CONFIRMED BUG

`Lifetime` is set by `.timed(duration)` and `.one_shot()` on `WallBuilder<Floor, V>` but is
never read in `build()` or `spawn()` in `terminal.rs`. Calling `.one_shot()` or `.timed(5.0)`
produces an identical entity to omitting those calls.

**Status**: CONFIRMED BUG — no production callers of `.timed()` or `.one_shot()` yet
(only test callers). `second_wind/system.rs` manually spawns its floor wall without the builder.
**Location**: `breaker-game/src/walls/builder/core/terminal.rs` (both `build()` impls).

## WallBuilder dispatch_effects: strips RootEffect.target, pushes all children to wall entity — LATENT HAZARD

`dispatch_effects` in `terminal.rs` uses `let RootEffect::On { then, .. } = root;`, discarding
`target`. All `then` children are pushed to the wall entity itself, regardless of whether
`target` was `Bolt`, `Cell`, or `Wall`.

Current RON (`wall.wall.ron`) has no effects. If a future wall RON adds `On(target: Bolt, ...)`,
those nodes land on the wall entity's `BoundEffects` — never fired by the bolt.

**Status**: LATENT — safe with current data, unsafe if multi-target wall definitions appear.
**Location**: `breaker-game/src/walls/builder/core/terminal.rs:38-40`.

## should_fail_fast suppresses fast-exit for ALL violations when any allowed_failures exists — OPEN

`should_fail_fast` (runner/app.rs) returns false when `allowed_failures` is `Some([...])` with any
entries — even if the actual violation is from a DIFFERENT, disallowed invariant kind.

**Status**: OPEN — no test covers the mixed-allowed/disallowed failure case.
**Location**: `breaker-scenario-runner/src/runner/app.rs:410-421`

## breaker_count_reasonable: invariant_checks increment AFTER early return (differs from all other checkers) — OPEN INCONSISTENCY

`check_breaker_count_reasonable` increments `stats.invariant_checks` AFTER the `!entered_playing`
early return (line 45), unlike all 21 other checkers which increment BEFORE any early return.

The test `does_not_increment_invariant_checks_when_entered_playing_false` explicitly validates
this behavior. In production the `playing_gate` run_if condition prevents the system from running
at all during `!entered_playing`, so the inconsistency has no runtime impact.

**Status**: OPEN inconsistency — intentional per test, but differs from all peers.
