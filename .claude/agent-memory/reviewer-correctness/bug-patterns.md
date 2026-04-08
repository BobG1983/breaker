---
name: bug-patterns — SPLIT
description: Split on 2026-04-08 (was 351 lines). See bug-patterns-open.md and bug-patterns-resolved.md.
type: project
---

This file has been split into two focused files:
- [bug-patterns-open.md](bug-patterns-open.md) — OPEN and LATENT bugs (actively check these)
- [bug-patterns-resolved.md](bug-patterns-resolved.md) — FIXED / CONFIRMED CORRECT patterns (do not re-flag)

## rantzsoft_stateflow transition effects: elapsed never incremented — FIXED (2026-04-06 verified)

All 12 built-in effects (fade, dissolve, pixelate, wipe, iris, slide) in `rantzsoft_stateflow`
now call `progress.elapsed += time.delta_secs()` in their run systems (verified in each
`*_run` system in `rantzsoft_stateflow/src/transition/effects/`). TransitionRunComplete
is sent when `elapsed >= duration`. This bug was OPEN as of 2026-04-03 but is FIXED on develop.

## rantzsoft_stateflow orchestration tests 8/9 vacuous assertion — OPEN (2026-04-03)

`out_transition_sends_state_changed_after_state_change` and
`out_transition_sends_transition_end_after_state_change` in
`rantzsoft_stateflow/src/transition/orchestration.rs` check
`iter_current_update_messages().next().is_some()` on the last of 10 updates.
Since messages are frame-scoped and the transition completes by update ~4-5,
there are no messages on update 10. These tests would assert false and FAIL
even for a correct implementation. Need a `MessageLog` capture-system approach.

## Insert-without-increment on first fire() — FIXED in Phase 1 cleanup

In effect `fire()` functions that accumulate state, the original bug inserted with
`accumulated: 0.0` on first call, missing the first trigger contribution. This was
confirmed in `ramping_damage.rs` Phase 1 original implementation.

**Current status**: FIXED. `ramping_damage::fire()` now inserts with
`accumulated: damage_per_trigger` on first call. `multi_call_accumulation_is_linear`
confirms: 4 calls at 0.5 yield 2.0 (not 1.5).

Watch for this pattern in other effect `fire()` functions that weren't reviewed.

## ccd_sweep_breaker fallback Aabb2D center semantics

`Aabb2D::new(center, half_extents)` stores `center` as-is in the `center` field.
`ray_intersect` uses `self.center` as world-space center. When constructing the
fallback expanded AABB in `ccd_sweep_breaker`, passing `breaker_pos` as center is
correct IF `breaker_pos` is already world-space. Confirmed correct — no bug.

## query_circle_filtered vs query_aabb_filtered in bolt_wall_collision

Old notes referred to `query_circle_filtered` — current code uses `query_aabb_filtered`.
The stale memory was corrected. Always verify which query function is actually used.

## gravity_well::fire() infinite loop when max == 0 — FIXED

`gravity_well::fire()` was previously missing the `max == 0` guard. Now has both
`if world.get_entity(entity).is_err() { return; }` AND `if max == 0 { return; }` guards
at lines 47-53. `spawn_phantom::fire()` also has the despawned-entity guard at line 46.

**Status**: FIXED — confirmed on feature/chip-evolution-ecosystem branch 2026-03-31.

## Phase 4 effect systems: shockwave/pulse migrated to Position2D (FIXED in full-verification-fixes)

`apply_shockwave_damage`, `apply_pulse_damage`, and `process_explode_requests` previously read
`transform.translation.truncate()` as the center. These were migrated to query `Position2D`
directly. Spawned shockwave/explode/ring entities now have `Position2D(position)` set at
fire-time. They still do NOT have `GlobalPosition2D` or `Spatial2D` — they are ephemeral
effect entities that do not move after spawn.

## Phase 5 effect fire(): Transform vs Position2D for bolt position

`chain_lightning::fire()` — FIXED in rework: now uses `Position2D` directly.

`piercing_beam::fire()` — FIXED in full-verification-fixes branch: now uses `entity_position()`
helper which returns `Position2D -> Vec2::ZERO` (no Transform fallback). Confirmed by tests in
`fire_tests.rs` (`fire_without_position2d_falls_back_to_zero_not_transform`, etc.).

## dispatch_chip_effects: effects dispatched even on max-stack add_chip failure — FIXED

`dispatch_chip_effects` now has `continue;` after the `add_chip` max-stacks warning
(system.rs line 57-59). The `for root_effect in &effects` loop is skipped when max stacks hit.

**Status**: FIXED — confirmed in code. Test: `chip_at_max_stacks_does_not_dispatch_effects`.

## apply_pending_bolt_effects: silently drops effects if bolt lacks BoundEffects — FIXED

`apply_pending_bolt_effects` (scenario-runner lifecycle/systems/pending_effects.rs) now uses
`commands.entity(entity).insert_if_new((BoundEffects::default(), StagedEffects::default()))`
before extending — matching the cell/wall variants.

**Status**: FIXED — confirmed in code.

## bypass_menu_to_playing: Target::Breaker initial_effects always dropped — FIXED

`PendingBreakerEffects` resource introduced in scenario-runner `lifecycle/systems/types.rs`.
`apply_pending_breaker_effects` registered in FixedUpdate after `tag_game_entities`.
`bypass_menu_to_playing` uses `commands.insert_resource(PendingBreakerEffects(...))`.

**Status**: FIXED — confirmed in code. `menu_bypass.rs` handles all 4 target types via Pending*Effects.

## TransferCommand silently drops non-Do children when BoundEffects/StagedEffects absent

`TransferCommand::apply` (effect/commands.rs) uses `entity_ref.get_mut::<BoundEffects>()` guarded
by `if let Some(...)`. If the entity lacks `BoundEffects` and `permanent: true`, non-`Do` children
(When/Once/Until nodes) are silently dropped. Same for `StagedEffects` when `permanent: false`.

**Status**: FIXED in full-verification-fixes branch. `ensure_effect_components()` helper now inserts
both `BoundEffects` and `StagedEffects` as defaults before the `get_mut` calls. Tests in
`commands.rs` section II cover all absent-component combinations.

## check_aabb_matches_entity_dimensions: false positive for breakers in non-1.0 EntityScale layouts

`check_aabb_matches_entity_dimensions` computes `expected = width.half_width() * scale` for breakers
and `expected = Vec2::splat(BoltRadius.0)` for bolts (no scale applied to bolt check).

The stored `Aabb2D` on both bolt and breaker entities is NEVER updated when `EntityScale` changes.
`apply_entity_scale_to_bolt` and `apply_entity_scale_to_breaker` only insert `EntityScale` on the
entity — neither touches `Aabb2D`. Physics systems compute live AABB from `BoltRadius * scale` /
`BreakerWidth * scale` directly; they do not use the stored component.

Result: for breakers in layouts with `entity_scale != 1.0` (e.g., `boss_arena.node.ron` has `entity_scale: 0.7`),
the checker fires false-positive `AabbMatchesEntityDimensions` violations because
`stored_half_extents (60.0, 10.0) != expected (42.0, 7.0)`.

For bolts, the checker uses `Vec2::splat(radius.0)` without multiplying by scale — so bolt checks
are always scale-1.0 semantics. This means bolt invariant is wrong for scaled layouts too.

**Status**: OPEN — confirmed on scenario-coverage branch review 2026-03-30.
Scenarios using Corridor (entity_scale=1.0) are not affected. Affects any scenario that adds
`AabbMatchesEntityDimensions` invariant to a non-1.0-scale layout.

## TetherChainActive resource lifecycle — RESOLVED as of 2026-03-31

Previously flagged as OPEN: resource could leak across node boundaries.

Analysis on feature/chip-evolution-ecosystem branch: `reverse(chain=true)` correctly calls
`world.remove_resource::<TetherChainActive>()` when the chip effect expires. `cleanup_tether_chain_resource`
runs `OnExit(GameState::Playing)` as a safety net. `TetherChainBeam` entities have `CleanupOnNodeExit`.

Within a run, the resource intentionally persists across nodes so `maintain_tether_chain` can
rebuild beams each time bolt count changes. The primary bolt's `BoundEffects` is `CleanupOnRunEnd`
so chain mode re-fires on each node start, which is correct behavior.

**Status**: RESOLVED — no leak. The open concern was based on incomplete analysis of the `reverse()` path.

## Missing cross-domain ordering: EffectSystems::Recalculate — RESOLVED by cache removal

The Phase 3 issue about `Effective*` components needing `.after(EffectSystems::Recalculate)` is
no longer applicable. The cache-removal refactor (scenario-coverage branch) eliminated all
`Effective*` components and the `recalculate_*` systems entirely. Consumer systems now read
`Active*` directly each frame — no stale-cache risk exists. **Do NOT re-flag this ordering issue.**

## ActiveQuickStops: fire() is no-op when component absent; no consumer reads multiplier — OPEN

`quick_stop::fire()` silently no-ops if `ActiveQuickStops` is absent (unlike all other stat boosts
which lazy-init). Neither `move_breaker` nor `dash::handle_braking` queries `ActiveQuickStops` to
scale their deceleration. `MovementQuery` and `DashQuery` do not include `ActiveQuickStops`.
The `QuickStop` effect fires but its multiplier is never applied to actual deceleration.

**Status**: OPEN — confirmed in cache-removal refactor review 2026-03-30.
Location: `effect/effects/quick_stop.rs` (fire fn), `breaker/queries.rs` (MovementQuery),
`breaker/systems/move_breaker.rs`, `breaker/systems/dash/system.rs`.

## circuit_breaker::fire() u32 underflow when bumps_required == 0 — OPEN latent bug

`circuit_breaker/effect.rs:73`: `let remaining = config.bumps_required - 1` where `bumps_required`
is `u32`. If `bumps_required == 0`, this panics in debug or wraps to `u32::MAX` in release.
Current production RON (`circuit_breaker.evolution.ron`) uses `bumps_required: 3` so no current
trigger. No guard against 0 exists. No test covers this case.

**Status**: OPEN — latent. Not triggered by current data. Needs `bumps_required == 0` guard.
Location: `breaker-game/src/effect/effects/circuit_breaker/effect.rs:73`

## MirrorProtocol::fire() wastes RNG call with dead random velocity — latent design issue

`mirror_protocol/effect.rs:73-78`: generates `random_velocity` via `rng.random_range(0..TAU)` and
passes it to bolt builder, but line 85 immediately overwrites with `mirror_vel`. The random call
advances RNG state for no observable effect. Other systems sharing `GameRng` will see different
values if MirrorProtocol fires. Not a correctness bug for the mirror bolt itself, but affects
RNG determinism for other game systems.

**Status**: OPEN design issue — the extra RNG call has no gameplay effect on the mirror bolt.

## BoltBuilder::build() silently drops OptionalBoltData — latent API hazard, not active bug

The four `build()` terminal impls in `breaker-game/src/bolt/builder.rs` (lines 376, 408, 440, 472)
include role+cleanup+serving markers in the returned tuple but silently drop: `bolt_params`,
`spawned_by`, `lifespan`, `with_effects`, `inherited_effects`.

**This is NOT an active bug**: grep confirms `build()` has zero callers in production code.
All production sites use `spawn()` which processes `OptionalBoltData` via `spawn_inner()`.

Tests in builder.rs section E test `build()` but only test paths where `optional.bolt_params`
is either None (no-config path tested in `build_without_from_config_has_no_bolt_params`) or set
via `config()` (which does populate `bolt_params`). No test calls `.with_lifespan(x).build()`
to verify lifespan is (or isn't) on the entity — this test gap is acceptable since `build()`
is unused in production.

**Status**: Latent hazard — safe to leave as-is unless `build()` gains production callers.

## .definition() builder omits BoltRespawnOffsetY and BoltRespawnAngleSpread — RESOLVED (Wave 6)

**RESOLVED** as of Wave 6 of feature/breaker-builder-pattern (2026-04-02).

`BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, and `BoltInitialAngle` were **deleted** from the
codebase in Wave 6. `LostBoltData` now queries `BoltSpawnOffsetY` and `BoltAngleSpread` (single
components, no respawn variants). The original latent bug was eliminated by removing the
component split. Do NOT re-flag.

## PrimaryBreaker marker never inserted on spawned breaker — RESOLVED (Breaker Builder Pattern)

**RESOLVED** as of feature/breaker-builder-pattern (2026-04-02).

`spawn_or_reuse_breaker` calls `Breaker::builder()...primary()...spawn(&mut commands)`.
The builder's `.primary()` method inserts `PrimaryBreaker` and `CleanupOnRunEnd` alongside `Breaker`.
`DispatchInitialEffects` and `resolve_default` queries for `(With<Breaker>, With<PrimaryBreaker>)`
will now find the breaker entity correctly. Do NOT re-flag.

## BreakerBuilder: with_width() does not recompute min_w/max_w — LATENT BUG

`core_params_from` reads `min_w: dims.min_w` always from `HasDimensions`, not from the
overridden width. When `.with_width(200.0)` is called on a builder whose `.definition()`
computed `min_w = 60.0` (= 120.0 * 0.5), the entity gets `BaseWidth=200.0`, `MinWidth=60.0`,
`MaxWidth=600.0`. The min/max are stale relative to the new base width.

No test checks `MinWidth`/`MaxWidth` after `.with_width()`. Same issue applies to `.with_height()`.

**Status**: LATENT — no production callers of `.with_width()` in current RON-driven flow.
**Location**: `breaker-game/src/breaker/builder/core.rs` (core_params_from)

## BreakerBuilder: with_lives()/with_color() before definition() are silently overwritten — LATENT BUG

`.definition()` unconditionally overwrites `optional.lives` and `optional.color_rgb`:
- `self.optional.lives = def.life_pool.map_or(...)` always executes
- `self.optional.color_rgb = Some(def.color_rgb)` always executes

So `.with_lives(Some(5)).definition(...)` silently ignores the `with_lives` call.
Same for `.with_color([...]).definition(...)`.

`with_effects` is partially guarded: `definition()` only overwrites if `def.effects` is non-empty.

**Status**: LATENT — valid call order is `.definition()` FIRST, then `.with_*()`.
Production spawns follow this order (`spawn_or_reuse_breaker` calls `.definition(def)` before `.rendered(...).primary()`).
**Location**: `breaker-game/src/breaker/builder/core.rs` (definition method)

## BreakerBuilder: rendered() before definition() uses wrong color — LATENT BUG

`.rendered(meshes, materials)` reads `self.optional.color_rgb` to create the material handle.
If called before `.definition()`, `color_rgb` is `None` at call time and falls back to
`BreakerDefinition::default().color_rgb`. Then `.definition()` sets `optional.color_rgb`
to the definition's color — but the material was already created with the wrong color.

**Status**: LATENT — production call order is `.definition(def).rendered(...).primary()`, which is correct.
**Location**: `breaker-game/src/breaker/builder/core.rs` (rendered method)

## Scale2D stores absolute pixel dimensions, not scale ratios — confirmed design pattern

`spawn_breaker` initializes `Scale2D { x: config.width, y: config.height }` (e.g., 120.0 × 20.0).
`sync_breaker_scale` writes `effective_size()` result (absolute dimensions) to `Scale2D`.
`compute_globals` copies `Scale2D → GlobalScale2D`, `derive_transform` puts these into
`Transform.scale`. Breaker sprite is `Rectangle::new(1.0, 1.0)` — scaled to pixel dimensions.
This is an absolute-dimension semantic for `Scale2D`, intentionally different from the usual
ratio-multiplier usage. Do NOT re-flag as "incorrect use of Scale2D".

## WallBuilder Lifetime field is never consumed in build()/spawn() — CONFIRMED BUG (Wave 2)

`Lifetime` is set by `.timed(duration)` and `.one_shot()` on `WallBuilder<Floor, V>` but is
never read in `build()` or `spawn()` in `terminal.rs`. Calling `.one_shot()` or `.timed(5.0)`
produces an identical entity to omitting those calls. No marker component or timer is inserted.

**Status**: CONFIRMED BUG — no production callers of `.timed()` or `.one_shot()` yet
(only test callers). `second_wind/system.rs` manually spawns its floor wall without the builder.
**Location**: `breaker-game/src/walls/builder/core/terminal.rs` (both `build()` impls).

## WallBuilder dispatch_effects: strips RootEffect.target, pushes all children to wall entity — LATENT HAZARD

`dispatch_effects` in `terminal.rs` uses `let RootEffect::On { then, .. } = root;`, discarding
`target`. All `then` children are pushed via `push_bound_effects` to the wall entity itself,
regardless of whether `target` was `Bolt`, `Cell`, or `Wall`.

Current RON (`wall.wall.ron`) has no effects. All test helpers use `target: Wall`.
If a future wall RON adds `On(target: Bolt, ...)`, those nodes land on the wall entity's
`BoundEffects` — never fired by the bolt. `dispatch_wall_effects` was deleted in the wall-builder-pattern feature; wall effects are now dispatched inline in Wall::builder().spawn(), but the target-stripping bug in dispatch_effects() still applies. The entire wall effects path remains scaffolding only.

**Status**: LATENT — safe with current data, unsafe if multi-target wall definitions appear.
**Location**: `breaker-game/src/walls/builder/core/terminal.rs:38-40`.

## WallBuilder Floor::compute_position does NOT add half_thickness — CONFIRMED CORRECT

`Floor::compute_position` returns `(0.0, playfield_bottom)` — no `+ ht` offset.
This matches `second_wind/system.rs` which spawns at `Position2D(Vec2::new(0.0, bottom_y))`.
The wall spans BELOW the playfield edge by `ht` (AABB half_extents), so the center is at the edge.
Do NOT re-flag the asymmetry vs Ceiling (which DOES add `ht`).

## should_fail_fast suppresses fast-exit for ALL violations when any allowed_failures exists — OPEN

`should_fail_fast` (runner/app.rs) returns false when `allowed_failures` is `Some([...])` with any
entries — even if the actual violation is from a DIFFERENT, disallowed invariant kind. A self-test
scenario allowing `BoltInBounds` that also triggers an unexpected `NoNaN` will NOT fail fast.
This is a logic bug: the check should compare the violation's kind against `allowed_failures`, not
just test whether the set is non-empty.

**Status**: OPEN — no test covers the mixed-allowed/disallowed failure case.
**Location**: `breaker-scenario-runner/src/runner/app.rs:410-421`

## breaker_count_reasonable: invariant_checks increment AFTER early return (differs from all other checkers)

`check_breaker_count_reasonable` increments `stats.invariant_checks` AFTER the `!entered_playing`
early return (line 45), unlike all 21 other checkers which increment BEFORE any early return.
When the checker is gated out by `entered_playing == false`, the counter is NOT incremented,
while other checkers always increment. The test `does_not_increment_invariant_checks_when_entered_playing_false`
explicitly validates this behavior. In production this doesn't matter because the `playing_gate`
run_if condition prevents the system from running at all during `!entered_playing`.
**Status**: OPEN inconsistency — intentional per test, but differs from all peers.

## sync_breaker_scale in Update vs collision systems in FixedUpdate — CONFIRMED CORRECT

`sync_breaker_scale` (Update) writes `Scale2D` (visual only). Collision systems (`breaker_cell_collision`,
`breaker_wall_collision`) in FixedUpdate read `BaseWidth`/`BaseHeight`/`ActiveSizeBoosts`/
`NodeScalingFactor` DIRECTLY — they do not read `Scale2D`. No ordering dependency. Intentional split.
Do NOT re-flag the Update/FixedUpdate mismatch.

## sync_bolt_scale Update vs tick_birthing FixedUpdate ordering — CONFIRMED CORRECT (feature/bolt-birthing-animation, 2026-04-08)

`sync_bolt_scale` (Update) has `Without<Birthing>` filter. `tick_birthing` (FixedUpdate) writes `Scale2D` for
birthing bolts. Commands flush between FixedUpdate and Update — so by the time sync_bolt_scale runs,
`Birthing` is already present and the bolt is excluded. No clobber. Intentional.

## bolt_lost deferred Birthing insert timing — CONFIRMED CORRECT (feature/bolt-birthing-animation, 2026-04-08)

`bolt_lost` does: (1) immediate mutable-query position/velocity write, (2) deferred commands insert
Scale2D{0,0}/PreviousScale{0,0}/CollisionLayers::default()/Birthing. The one-frame window where
the bolt has a new position but still has old scale/layers is safe because bolt_lost runs last
in FixedUpdate and no other system in the same frame can detect the bolt at its new position.
The deferred commands flush before Update, so sync_bolt_scale's Without<Birthing> filter correctly
excludes the bolt. Do NOT re-flag this one-frame window.

## all_animate_in_complete: query is NOT bolt-specific — CONFIRMED CORRECT (feature/bolt-birthing-animation, 2026-04-08)

`all_animate_in_complete` queries `Query<(), With<Birthing>>` — matches ANY entity with Birthing,
not just bolts. This is intentional: it waits for ALL birthing animations to complete regardless
of entity type. If any non-bolt entity gains Birthing in the future, this system will naturally
wait for them too. Do NOT re-flag as "should be bolt-specific".

## quit teardown chain: TransitionType::None via dynamic transition — CONFIRMED CORRECT (feature/bolt-birthing-animation, 2026-04-08)

`with_dynamic_transition` closure returning `TransitionType::None` correctly propagates through
`resolve_transition` as `Some(TransitionType::None)`, then `begin_transition` handles it with
an early-return instant state change. No `Time<Virtual>` pause, no overlay. The route is
`GameState::Menu → Teardown` (instant) and `MenuState::Main → Teardown` (instant), which chains
correctly to `AppState::Teardown` via the condition-triggered AppState route watching `GameState`.

## Birthing stashed_layers in bolt_lost — confirmed correct (2026-04-08)

`bolt_lost` stashes `entry.layers` which was copied from the bolt's current CollisionLayers
before any insert. The stashed value is the correct full-collision layer set since no system
has zeroed the layers yet at that point.
