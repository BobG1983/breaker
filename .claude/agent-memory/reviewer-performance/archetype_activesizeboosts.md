---
name: ActiveSizeBoosts archetype fragmentation
description: Optional ActiveSizeBoosts on Breaker — fragmentation impact assessment
type: project
---

`ActiveSizeBoosts` is `Option<&ActiveSizeBoosts>` in both `SyncBreakerScaleQuery` and `BreakerCellCollisionQuery` (and `DashQuery`). The component is dynamically added/removed at runtime on the Breaker entity.

At 1 Breaker entity, archetype churn from add/remove is completely negligible — there's exactly one entity to re-bucket. Each add/remove creates at most 2 archetypes. This is acceptable and was confirmed as a non-issue.

**Why:** Breaker is a singleton. ECS archetype invalidation cost scales with entity count moving between archetypes, not number of archetypes.

**How to apply:** Do not flag Optional components on Breaker as archetype fragmentation — it only matters for entity types that exist in the hundreds.
