---
name: scan_threshold_triggers per-frame scan pattern
description: HashSet alloc every FixedUpdate tick scanning all BoundEffects trees — acceptable at current scale
type: project
---

`scan_threshold_triggers` allocates a `HashSet<OrderedFloat<f32>>` on the stack (actually heap)
every FixedUpdate tick and recursively walks all `BoundEffects` trees looking for
`NodeTimerThresholdOccurred` triggers.

**Entity count context:** BoundEffects live on ~1 breaker + a handful of chip entities. At
current scale (Phase 1–2), walking a few trees with depth ≤ 3 and allocating a tiny HashSet
is negligible even at 64 Hz.

**The real question (deferred to Phase 3):** If cells eventually get BoundEffects installed
(50–200 entities), this scan grows O(entities * tree_depth) every FixedUpdate tick. At that
point, moving to a change-detection pattern (only rescan on BoundEffects Added/Removed) would
be worthwhile.

**Stored pattern:** The intermediate HashSet is used for deduplication and then `.extend()`d
into the registry Vec. The registry is a Vec, not a HashSet — so the intermediate HashSet is
necessary unless the registry type changes. Worth noting for a potential future refactor.
