---
name: SpatialData optional fields
description: Scale2D and PreviousScale added as optional fields to SpatialData QueryData
type: project
---

`SpatialData` in `rantzsoft_spatial2d/src/queries.rs` has two optional fields added:
- `pub scale: Option<&'static Scale2D>`
- `pub previous_scale: Option<&'static PreviousScale>`

These are used in `BoltCollisionData`, `ResetBoltData`, `LostBoltData`, and `apply_attraction`.

**Impact:** Optional fields in a `QueryData` derive struct cause the query to match entities regardless of whether those components are present. At bolt counts of 1–few entities, the per-frame cost of the None-branch check is unmeasurable. No archetype fragmentation concern because bolt archetypes are stable (Scale2D/PreviousScale are spawned at bolt creation, not added/removed dynamically).

**How to apply:** Do not flag optional fields on SpatialData as a performance concern unless bolt entity counts grow to hundreds.
