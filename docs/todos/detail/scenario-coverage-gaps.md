# Scenario Coverage Gaps

Identified during Full Verification Tier review of `feature/breaker-builder-pattern` (2026-04-02).

## HIGH Priority

### 1. `BreakerPositionClamped` invariant stale after size boost refactor

`check_breaker_position_clamped` computes bounds as `playfield.right() - BaseWidth.half_width()`. But `move_breaker` now uses `effective_half_w = base_width.half_width() * size_boost_multiplier`. The invariant doesn't account for `ActiveSizeBoosts`, so it would produce false positives when size boosts are active on the breaker.

**Fix:** Update `BreakerPositionClamped` checker to query `ActiveSizeBoosts` and use the effective half-width.

### 2. No scenario exercises non-trivial `entity_scale` (node scaling factor != 1.0)

Every existing scenario uses layouts with `entity_scale: 1.0`. The `sync_bolt_scale` and `sync_breaker_scale` systems are only exercised with trivial node scaling in live gameplay. A layout with `entity_scale: 0.5` could introduce a bolt small enough to tunnel through walls.

**Fix:** Create `node_scale_entity_chaos.scenario.ron` with a layout using `entity_scale: 0.5`, apply `SizeBoost(2.0)` on every Bumped, and enable `BoltInBounds`, `BoltSpeedAccurate`, `AabbMatchesEntityDimensions`, `BreakerInBounds`, `BreakerPositionClamped`, `NoNaN`.

## MEDIUM Priority

### 3. No multi-node scenario for `spawn_or_reuse_breaker` reuse path

Every scenario runs from node 1. The reuse path (returns early, sends `BreakerSpawned` without spawning) is only exercised in unit tests. No scenario exercises multi-node progression.

**Fix:** Add a `BreakerCountReasonable` invariant (exactly 1 `PrimaryBreaker` during play). Create a multi-node scenario using `allow_early_end: true`.

### 4. `entity_scale_collision_chaos` only exercises boost, not node scaling

The layout `"Dense"` uses `entity_scale: 1.0`, so `NodeScalingFactor` has no effect. No min/max radius constraint is exercised — `ClampRange` is always `NONE`.

**Fix:** Create `bolt_radius_clamping_chaos.scenario.ron` with explicit `MinRadius`/`MaxRadius` on the bolt and `SizeBoost` to drive radius past max.

### 5. `sync_breaker_scale` height boost has no scenario regression

The old `width_boost_visual` only applied boosts to width. `sync_breaker_scale` now applies to both width AND height. No scenario verifies that height is actually boosted at runtime.

**Fix:** Enable `BreakerPositionClamped` and `AabbMatchesEntityDimensions` in existing scenarios when size boost is active.

## New Invariants Needed

- **`BreakerCountReasonable`** — fires if `With<PrimaryBreaker>` count != 1 during `GameState::Playing`
- **`BoltScaleCoherent`** (optional) — checks `Scale2D.x == effective_radius(BaseRadius, boost, node_scale, range)`
- **Update `BreakerPositionClamped`** — account for `ActiveSizeBoosts` in half-width calculation
