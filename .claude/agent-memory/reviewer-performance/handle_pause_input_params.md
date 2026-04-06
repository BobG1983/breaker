---
name: handle_pause_input ResMut<NodeOutcome> + MessageWriter params
description: handle_pause_input uses ResMut<NodeOutcome> and MessageWriter<ChangeState<NodeState>> — both are gated by run_if, different schedule from lifecycle writers, no conflict
type: project
---

`handle_pause_input` system params: `ResMut<NodeOutcome>`, `MessageWriter<ChangeState<NodeState>>`,
`ResMut<PauseMenuSelection>`, `ResMut<Time<Virtual>>`.

`ResMut<NodeOutcome>` is also mutably accessed by `handle_node_cleared`, `handle_timer_expired`,
`handle_run_lost` — but those run in `FixedUpdate`. `handle_pause_input` runs in `Update`.
Different schedules = no Bevy system-parallelism conflict.

The system is fully gated: `is_time_paused.and(any_with_component::<PauseMenuScreen>).and(not_in_transition)`.
During normal gameplay (unpaused) it does not execute at all.

**Why:** `ResMut` in `Update` on a resource that `FixedUpdate` also writes is fine — Bevy schedules
run sequentially, not concurrently across Update/FixedUpdate within one app update.
**How to apply:** Do not flag cross-schedule ResMut access as a parallelism concern.
