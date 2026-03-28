---
name: Intentional Patterns — Phase 1 Collision Refactor
description: Patterns that look like violations but are intentional in this codebase, established during Phase 1 collision refactor review
type: project
---

## `is_inside_aabb` inline AABB check in `bolt_wall_collision`

The overlap detection in `bolt_wall_collision` (four individual comparisons for left/right/top/bottom distances) is not a violation — this system needs both the overlap test AND the nearest-face direction in one pass, which `Aabb2D::is_inside` would not provide. The manual inline implementation is intentional.

**Why:** The nearest-face identification needs the individual per-axis distances regardless of whether `Aabb2D::is_inside()` exists. Splitting into two passes (is_inside + nearest_face) would redundantly recompute.

**How to apply:** Do not flag the four-comparison overlap block in `bolt_wall_collision` as a missed utility opportunity.

## `breaker_cell_collision` / `breaker_wall_collision` as near-duplicates

These two files are structurally identical (same query type alias, same scale computation, same quadtree call, different layer constants and message type). They are intentionally separate because they send different messages and will diverge when moving-cell mechanics are added. Do not flag as reuse target without understanding the future divergence plan.

## `detect_*` rename to `breaker_*` / `cell_*`

The old `detect_breaker_cell_collision`, `detect_breaker_wall_collision`, `detect_cell_wall_collision`, `cleanup_destroyed_cells` names were replaced with `breaker_cell_collision`, `breaker_wall_collision`, `cell_wall_collision`, `cleanup_cell`. This is a vocabulary / naming convention improvement, not a functional change.

## `hit_fraction` extracted function in `bolt_breaker_collision/system.rs`

`hit_fraction` is a small extracted function that computes normalized hit position on the breaker surface. It appears in two call sites within the same file (overlap path and CCD path). This is correct extraction, not duplication.

## `spawn_bolt` / `spawn_wall` / `spawn_breaker_at` test helpers are local to each test module

Each collision test module (bolt_wall_collision, bolt_breaker_collision/tests) defines its own spawn helpers locally rather than sharing a crate-wide test utility. This is the established pattern in this codebase — test helpers are co-located with their test module. Do not flag as duplication without confirmation that a shared test utility module exists or is planned.
