---
name: bolt-lost mid-node respawn bypasses birthing
description: bolt_lost teleports the bolt in-place during Playing; begin_node_birthing only fires OnEnter(AnimateIn), so the mid-node respawn has never gone through birthing
type: project
---

`bolt_lost` (FixedUpdate, NodeState::Playing) repositions the primary bolt entity in-place by mutating `Position2D`, `Velocity2D`, and `PreviousPosition` directly. No entity is destroyed or spawned. Because the state never leaves `Playing`, `OnEnter(NodeState::AnimateIn)` never fires and `begin_node_birthing` never runs.

`tick_birthing` already runs during both `AnimateIn` and `Playing` — so inserting `Birthing` in `bolt_lost` (non-extra branch only) would be picked up on the next frame with no schedule changes.

**Why:** The birthing feature was built around the node-start lifecycle (`OnEnter(AnimateIn)`) and the `Bolt::builder().birthed().spawn()` pattern at initial spawn. The mid-node bolt-lost respawn is a mutation path that was never covered.

**How to apply:** When tracing any bolt lifecycle question, distinguish these three paths:
1. Initial spawn (builder with `.birthed()`) — has birthing
2. Node transition (`reset_bolt` OnEnter Loading → `begin_node_birthing` OnEnter AnimateIn) — has birthing
3. Mid-node bolt-lost respawn (`bolt_lost` FixedUpdate Playing) — currently NO birthing (the bug)
