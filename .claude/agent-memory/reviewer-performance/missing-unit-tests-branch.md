---
name: feature/missing-unit-tests branch review
description: 58 new unit tests reviewed; one production change (overlay_color pub(super)); findings for each test file
type: project
---

## Branch: feature/missing-unit-tests

58 new unit tests across ~10 test files. One production change.

**Why reviewed:** Standard Verification Tier — reviewer-performance is required before commit.

**How to apply:** Future reviews of test-only branches follow the same framework: check (1) production changes, (2) App vs World test harness choice, (3) entity spawning patterns.

## Production Change

`breaker-game/src/fx/transition/system.rs` line 185: `overlay_color` visibility changed from private to `pub(super)`. The function is `const fn` that takes only data arguments — zero performance impact.

## Test File Assessment

All test files reviewed. No critical or moderate issues found. One minor observation:

### `spawn_bolt/tests.rs`
- `test_app()` calls `init_resource::<Assets<Mesh>>()` and `init_resource::<Assets<ColorMaterial>>()`. These are heap-allocated asset maps. Justified because `spawn_bolt` needs them to avoid panics — the system accesses these resources. Not wasteful; cannot use bare World.
- Uses `App::new()` + `MinimalPlugins` — correct choice; system requires Time, FixedTime, and message infrastructure.

### `bolt_wall_collision/tests.rs`
- Uses `RantzPhysics2dPlugin` — required because the collision system depends on the quadtree being maintained. Correct; cannot use bare World.
- Spawns `GlobalPosition2D` + `Spatial2D` + `Aabb2D` + `CollisionLayers` per entity — correct, these are all required by the quadtree. Not fragmentation; each test spawns 1-4 entities.

### `clamp_bolt_to_playfield/tests.rs`
- Uses `MinimalPlugins` + `FixedUpdate`. Correct; `PlayfieldConfig` is a plain resource. No App overhead beyond minimum.
- `tick()` helper pattern using `accumulate_overstep` — correct Bevy 0.18 pattern for FixedUpdate tests.

### `prepare_bolt_velocity/tests.rs`
- One test (`no_breaker_leaves_velocity_unchanged`) creates its own `App::new()` without the shared `test_app()` helper to deliberately omit the breaker. This is intentional and correct.

### `reset_bolt/tests.rs`
- Spawns multiple component bundles per test entity but no overlap or fragmentation concern — tests spawn 1-3 entities per test.

### `spawn_bolt/tests.rs` entity patterns
- Entity spawning includes `Spatial2D`, `GameDrawLayer::Breaker`, `Position2D` — all required components. Not fragmentation.

### `bump_visual/tests.rs`
- Three separate `test_app()` functions (`trigger_test_app()`, `animate_test_app()`, plus direct `App::new()` in one test) — each adds only the one system under test. This is optimal decomposition; not wasteful.

### `dash/tests.rs`
- `settling_tilt_is_frame_rate_independent` creates TWO full `App` instances and runs multiple ticks each. This is the most expensive test in the set. Justified: frame-rate independence requires two separate simulations at different timesteps. Cannot be simplified to a pure unit test without losing coverage.

### `propagate_breaker_changes/tests.rs`
- Uses `AssetPlugin::default()` in addition to `MinimalPlugins`. Required because `BreakerDefaults` is an asset type and `init_asset::<BreakerDefaults>()` needs the asset plugin. Correct.

### `init_breaker/tests.rs`
- `apply_overrides_modifies_config` test creates its own `App::new()` + `AssetPlugin` + `BreakerDefaults` asset. Necessary: tests `apply_breaker_config_overrides` system directly, which reads assets.

### `run/definition/tests.rs` and `run/resources/tests.rs`
- Pure unit tests using `ron::de::from_str`. No App at all. Optimal.

### `fx/transition/tests.rs`
- Uses `App::new()` + `MinimalPlugins` + `StatesPlugin`. Required: `spawn_transition_out/in` read `Res<TransitionConfig>` and `ResMut<GameRng>`; `animate_transition` writes `NextState`. State infrastructure requires `StatesPlugin`. Correct.
- `overlay_color_*` tests call the function directly without App. Optimal.

## Summary

All 58 tests use the minimum harness for their requirements. No accidental full-App usage where bare World would suffice, no per-test Vec/HashSet allocations in hot paths, no archetype fragmentation patterns in test entity spawning.
