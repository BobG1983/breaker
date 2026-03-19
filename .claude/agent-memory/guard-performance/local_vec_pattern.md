---
name: local_vec_pattern
description: Verified: Local<Vec> clear/extend is the correct hot-path allocation pattern for this codebase
type: project
---

Verified in review of bolt_lost.rs and handle_cell_hit.rs (2026-03-19).

**Pattern:** `Local<Vec<T>>` with `.clear()` then `.extend(iter)` or `.push()` within a FixedUpdate system.

**Why it works:**
- `Vec::clear()` drops contained values but does NOT deallocate the heap buffer — capacity is preserved
- `Vec::extend()` only reallocates if capacity is exceeded; after warmup the Vec is at steady-state capacity and extend is zero-alloc
- `Local<T>` in Bevy is a per-system `SystemLocal` resource; its lifetime matches the system's lifetime (entire app run), so the Vec persists across frames

**Bounded sizes confirmed in this codebase:**
- bolt_lost.rs: max ~8-16 entries (bolt count ceiling)
- handle_cell_hit.rs: max 4 entries (MAX_BOUNCES physical ceiling)

**Archetype impact:** None — Local<T> is not a component; it does not participate in archetypes at all.

**How to apply:** This is the accepted pattern for any FixedUpdate system that needs a scratch buffer. Do not flag it as a concern; do flag any new per-frame `Vec::new()` or `vec![]` allocations in hot paths as regressions.
