---
name: World borrow conflict in fire() functions
description: Holding a mutable resource borrow while querying world in fire() causes compile errors — a recurring spec mistake
type: feedback
---

In Bevy 0.18, `world.get_resource_mut::<R>()` returns a `Mut<R>` that holds a live borrow on the World. If a spec says "get resource, then query world" without explicitly dropping the resource borrow first, the implementation will fail to compile.

**Why:** fire() functions operate on `&mut World` directly (not via system parameters). The borrow checker enforces that you cannot call `world.query()` while holding a mutable resource borrow from the same world.

**How to apply:** When reviewing any spec that describes "get resource from world, then query world components" in the same function, flag it as BLOCKING unless the spec explicitly states the resource borrow is dropped (via scope or explicit drop) before the query runs. The existing gravity_well/effect.rs uses a `{ }` scope block for its query — this is the established pattern.
