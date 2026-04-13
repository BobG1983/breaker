---
name: death bridge double global_query walk for Specific + Any
description: on_destroyed_inner iterates global_query twice per death event (DeathOccurred(kind) + DeathOccurred(Any))
type: project
---

`on_destroyed_inner` in `triggers/death/bridges.rs` calls `global_query.iter()` twice per
destroyed entity message: once for `DeathOccurred(kind)` and once for `DeathOccurred(Any)`.
Each walk is O(entities_with_BoundEffects).

**At current scale:** BoundEffects entities = ~1 breaker + a few chip entities. Two walks over
~5 entities per cell death is negligible. Cell deaths are infrequent (not every frame).

**Phase 3 concern:** With 200 cells each triggering death once, and each death walking 50
BoundEffects entities twice, that's 200 * 2 * 50 = 20,000 walk calls over the node teardown
phase. Still probably fast since walk_effects itself is O(tree_depth), but worth revisiting if
teardown hitches appear.

**The Specific+Any design is intentional** and was added in this refactor to support chips that
react to "any death" vs. "cell death" separately — don't flag as a bug.
