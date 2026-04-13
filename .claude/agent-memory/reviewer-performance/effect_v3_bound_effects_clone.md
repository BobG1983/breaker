---
name: BoundEffects clone pattern in bridge systems
description: bound.0.clone() on every entity per collision message — cost and context
type: project
---

Every bridge system (impact, death, node) clones `bound.0` (a `Vec<(String, Tree)>`) before
calling `walk_effects`. Trees contain `Box<ScopedTree>` and `Vec<Terminal>` but are small
in practice (1–3 entries per chip). At current scale (~1 breaker, maybe 3–5 chips installed),
these clones are O(chips) and happen only on collision events, not every frame.

**Why it exists:** `walk_effects` takes ownership of `&[(String, Tree)]` passed by value, and
the borrow from the query can't be held across the mutable `commands` call.

**Scale concern:** If BoundEffects grows to 10+ trees per entity and there are 50+ BoundEffects
entities (e.g. all cells have effects installed), and multiple collisions per frame, this
could be 50 * N_collisions * O(tree_depth) clones per frame. Watch at Phase 3.
