---
name: Global component change detection perf trap
description: Systems that write Global* components every frame make Changed<Global*> always true, defeating incremental updates in consumers like quadtree
type: feedback
---

When introducing Global* components (GlobalPosition2D, etc.) that are recomputed every frame by a propagation system (compute_globals), any downstream system using `Changed<GlobalFoo>` will fire every frame for every entity — because the propagation system always writes even when the value hasn't changed.

**Why:** Bevy's change detection triggers on write, not on value change. A system that writes `*global_pos = local_pos` every frame marks it as changed even if the value is identical.

**How to apply:** When specs propose migrating a `Changed<Position2D>` query to `Changed<GlobalPosition2D>`, flag that the incremental behavior is lost. Either: (a) keep reading the local component for change detection, (b) add a "skip if equal" guard before writing, or (c) accept the performance regression and note it explicitly.
