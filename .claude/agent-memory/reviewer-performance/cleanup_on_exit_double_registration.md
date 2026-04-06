---
name: cleanup_on_exit double registration pattern
description: Safety-net: cleanup_on_exit<NodeState> registered on BOTH OnEnter(NodeState::Teardown) and OnEnter(RunState::Teardown) — intentional, not a fragmentation or correctness issue
type: project
---

`plugin.rs` `register_cleanup()` registers `cleanup_on_exit::<NodeState>` on two schedules:
1. `OnEnter(NodeState::Teardown)` — normal path
2. `OnEnter(RunState::Teardown)` — safety net for quit-from-pause where NodeState may not reach Teardown

Both registrations are `OnEnter` (one-shot) so they do not run every frame. No performance concern.

The `CleanupOnExit<S>` query uses `With<CleanupOnExit<NodeState>>`. If both fire sequentially
(normal path + safety net), the second cleanup runs against zero entities (already despawned).
Bevy 0.18 `commands.entity(e).despawn()` on an already-despawned entity logs a warning but does
not panic — the `OnEnter(NodeState::Teardown)` fires before `OnEnter(RunState::Teardown)` in the
normal quit path, so in practice the safety net always sees zero remaining entities.

**Why:** Safety net covers the case where the quit-from-pause bypasses `NodeState::Teardown`.
**How to apply:** Do not flag this double registration as fragmentation or over-querying — it is intentional design with zero per-frame cost.
