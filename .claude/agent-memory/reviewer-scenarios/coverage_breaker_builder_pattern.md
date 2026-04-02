---
name: Breaker Builder Pattern Coverage Map
description: Scenario and invariant gaps from the breaker-builder-pattern branch — new spawn_or_reuse_breaker, BreakerBuilder typestate, BoltBuilder, sync_bolt_scale, sync_breaker_scale, effective_radius/effective_size with ClampRange
type: project
---

## What Changed (feature/breaker-builder-pattern branch)

- `BreakerConfig` eliminated — all breaker params now stamped by `BreakerBuilder` typestate pattern (7 dimensions: Dimensions, Movement, Dashing, Spread, Bump, Visual, Role)
- `spawn_or_reuse_breaker` replaces a 4-system pipeline — a single system spawns or reuses the breaker entity
- `BoltBuilder` typestate introduced (6 dimensions: Position, Speed, Angle, Motion, Role, Visual)
- `sync_bolt_scale` replaces `bolt_scale_visual` — uses `effective_radius(BaseRadius, boost, node_scale, ClampRange)` to set `Scale2D`
- `sync_breaker_scale` replaces `width_boost_visual` — uses `effective_size(BaseWidth, BaseHeight, boost, node_scale, width_range, height_range)` — BOTH axes now boosted (old only boosted width)
- `apply_node_scale_to_breaker` and `apply_node_scale_to_bolt` insert `NodeScalingFactor` on entity enter
- `ClampRange` + `effective_radius` + `effective_size` in `shared/size.rs` are new pure functions
- `MinWidth`, `MaxWidth`, `MinHeight`, `MaxHeight`, `MinRadius`, `MaxRadius` are new components inserted by builders

## Scenario Coverage Status

| Mechanic | Coverage | Quality |
|----------|----------|---------|
| spawn_or_reuse_breaker first-node | Implicit via ALL scenarios | Never probed directly as scenario — only unit tests |
| spawn_or_reuse_breaker subsequent-node (reuse path) | Zero scenario coverage | NOT COVERED by any scenario |
| BreakerBuilder dimension constraints (MinWidth/MaxWidth) | No scenario exercises constraint clamping | NOT COVERED |
| sync_breaker_scale: boost on both axes | Unit tested in sync_breaker_scale.rs | No scenario — AabbMatchesEntityDimensions checker does NOT verify Scale2D against effective size |
| sync_breaker_scale: node scale on both axes | Unit tested | No scenario |
| sync_bolt_scale: boost + node_scale + ClampRange | Unit tested | entity_scale_collision_chaos has SizeBoost but no node scale variation |
| effective_radius clamping in live gameplay | entity_scale_collision_chaos (boost only) | No scenario with non-trivial node scaling factor (layout entity_scale != 1.0) |
| PrimaryBreaker marker correctness | spawn_or_reuse.rs unit tests | No scenario invariant |
| BoltBuilder primary/extra lifecycle | NoEntityLeaks catches gross errors | No invariant checks PrimaryBolt marker presence per-frame |
| AabbMatchesEntityDimensions vs Scale2D coherence | aabb_matches_entity_dimensions self-test | Checker compares Aabb2D vs BaseWidth/BaseRadius — DOES NOT check Scale2D vs effective size |

## Critical Behavioral Change: move_breaker uses boost but NOT node_scale for clamping

`BreakerMovementData` does NOT include `NodeScalingFactor`. `move_breaker` computes:
  `effective_half_w = base_width.half_width() * size_boost_multiplier`
But `sync_breaker_scale` computes:
  `effective_size = base * boost * node_scale` (clamped)

If `node_scale < 1.0`, `sync_breaker_scale` shrinks the visual scale but `move_breaker` clamping doesn't shrink accordingly.
If `node_scale > 1.0`, `sync_breaker_scale` grows the visual scale but `move_breaker` doesn't account for the wider visual footprint.

The `BreakerPositionClamped` invariant uses `BaseWidth` only — does NOT account for boost or node_scale.

## Invariant Gaps Introduced by This Migration

1. **Scale2D coherence with effective size** — `sync_breaker_scale`/`sync_bolt_scale` update `Scale2D` but no invariant checks that `Scale2D` matches `effective_size(BaseWidth, BaseHeight, boost, node_scale, ...)`. The `AabbMatchesEntityDimensions` checker verifies `Aabb2D` vs `BaseWidth`/`BaseRadius` (unscaled) — correct design, but no checker validates the scaled visual.

2. **BreakerPositionClamped stale after refactor** — uses `BaseWidth.half_width()` only. After the refactor, `move_breaker` uses `base_width * size_boost` for position clamping. The invariant does not track boost. If a future change adds node_scale to move_breaker clamping, the invariant won't follow automatically.

3. **Node-scaling layout scenario gap** — no scenario uses a layout with `entity_scale != 1.0` AND a size boost effect simultaneously. The interaction path (boost + node_scale applied, then clamped by ClampRange) is only covered in unit tests, never in the scenario runner where collision detection is live.

4. **spawn_or_reuse reuse path** — no scenario forces a node transition where the breaker entity persists (second+ node). All scenario coverage implicitly tests first-node spawn only.

## How to apply

- Flag missing node-scale + size-boost combination scenario as HIGH — it's a new interaction path in sync_bolt_scale/sync_breaker_scale that no existing scenario stresses with live physics.
- Flag BreakerPositionClamped not tracking boost as MEDIUM — move_breaker is correct (uses boost), but invariant is checking stale geometry.
- Flag spawn_or_reuse reuse path missing as MEDIUM.
- Flag Scale2D coherence gap as LOW (visual only — doesn't affect gameplay correctness).
