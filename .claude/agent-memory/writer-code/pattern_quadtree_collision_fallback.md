---
name: Quadtree collision narrow-phase pattern
description: Quadtree query_aabb_filtered returns broad-phase candidates that may not actually overlap — always add narrow-phase AABB overlap check before emitting collision messages
type: feedback
---

Quadtree `query_aabb_filtered` and `query_circle_filtered` return broad-phase candidates that MAY overlap, not guaranteed overlaps. Every collision system must verify actual geometric overlap before sending impact messages.

**Pattern for AABB-vs-AABB systems** (breaker_cell_collision, breaker_wall_collision, cell_wall_collision):
1. Get candidates from quadtree broad-phase
2. Look up each candidate's `Position2D` + `Aabb2D` via a lookup query (e.g., `Query<(&Position2D, &Aabb2D), With<Wall>>`)
3. Verify overlap: `dx < half_w_a + half_w_b && dy < half_h_a + half_h_b`
4. Only emit message if overlap is confirmed

**Pattern for circle-vs-AABB systems** (bolt_wall_collision):
1. Get candidates from `query_circle_filtered`
2. Build expanded AABB (wall AABB expanded by bolt radius)
3. Check strict inequality: bolt center inside expanded AABB
4. Only emit message if inside

**Why:** Without narrow-phase verification, false positive collision messages fire for entities that are only near each other in the quadtree grid but not geometrically overlapping. This causes incorrect gameplay behavior.

**How to apply:** Any system using `query_aabb_filtered` or `query_circle_filtered` must add a narrow-phase geometric check. See `bolt_wall_collision.rs` for the circle-vs-AABB reference pattern, and `breaker_cell_collision.rs` for the AABB-vs-AABB reference pattern.
