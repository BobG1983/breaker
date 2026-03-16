---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-16 post-Phase-2e)
type: reference
---

# Message Flow Map

Last updated: 2026-03-16 (Phase 2e — ApplyTimePenalty, SpawnAdditionalBolt added)

## Registered Messages (by plugin)

| Message | Registered by |
|---------|--------------|
| KeyboardInput | Bevy built-in (InputPlugin reads it) |
| BumpPerformed | BreakerPlugin |
| BumpWhiffed | BreakerPlugin |
| BoltHitBreaker | PhysicsPlugin |
| BoltHitCell | PhysicsPlugin |
| BoltLost | PhysicsPlugin |
| CellDestroyed | CellsPlugin |
| NodeCleared | NodePlugin (RunPlugin sub-plugin) |
| TimerExpired | NodePlugin (RunPlugin sub-plugin) |
| ApplyTimePenalty | NodePlugin (RunPlugin sub-plugin) — NEW |
| SpawnAdditionalBolt | BoltPlugin — NEW |
| RunLost | RunPlugin |
| UpgradeSelected | UiPlugin |
| AppExit | Bevy built-in |

---

## Message Flow Detail

### KeyboardInput (Bevy built-in)
- Sender: Bevy input system
- Receiver: `read_input_actions` (InputPlugin, PreUpdate)

### BumpPerformed (BreakerPlugin → cross-domain)
- Senders:
  - `update_bump` (BreakerPlugin, retroactive path)
  - `grade_bump` (BreakerPlugin, forward/same-frame path)
- Receivers:
  - `perfect_bump_dash_cancel` (BreakerPlugin)
  - `spawn_bump_grade_text` (BreakerPlugin)
  - `apply_bump_velocity` (BoltPlugin)
  - `bridge_bump` (BehaviorPlugin, conditional) — may trigger SpawnBoltRequested, TimePenaltyRequested
  - `track_bump_result` (DebugPlugin, dev only)

### BumpWhiffed (BreakerPlugin → cross-domain)
- Sender: `grade_bump` (BreakerPlugin)
- Receivers:
  - `spawn_whiff_text` (BreakerPlugin)
  - `track_bump_result` (DebugPlugin, dev only)

### BoltHitBreaker (PhysicsPlugin → cross-domain)
- Sender: `bolt_breaker_collision` (PhysicsPlugin)
- Receivers:
  - `grade_bump` (BreakerPlugin)

### BoltHitCell (PhysicsPlugin → cross-domain)
- Sender: `bolt_cell_collision` (PhysicsPlugin)
- Receivers:
  - `handle_cell_hit` (CellsPlugin)

### BoltLost (PhysicsPlugin → cross-domain)
- Sender: `bolt_lost` (PhysicsPlugin, PhysicsSystems::BoltLost) — fires for baseline AND ExtraBolt
- Receivers:
  - `spawn_bolt_lost_text` (BoltPlugin)
  - `bridge_bolt_lost` (BehaviorPlugin, conditional) — may trigger LoseLifeRequested or TimePenaltyRequested

### CellDestroyed (CellsPlugin → RunPlugin/NodePlugin)
- Sender: `handle_cell_hit` (CellsPlugin, FixedUpdate, no ordering vs receiver)
- Receivers:
  - `track_node_completion` (NodePlugin, FixedUpdate, NodeSystems::TrackCompletion)
- One-tick delay is safe — messages persist across frames.

### NodeCleared (NodePlugin internal → RunPlugin)
- Sender: `track_node_completion` (NodePlugin, FixedUpdate, NodeSystems::TrackCompletion)
- Receiver: `handle_node_cleared` (RunPlugin, FixedUpdate, .after(NodeSystems::TrackCompletion))
- Same-tick propagation guaranteed by ordering.

### TimerExpired (NodePlugin internal → RunPlugin) — now has TWO senders
- Senders:
  - `tick_node_timer` (NodePlugin, FixedUpdate, NodeSystems::TickTimer) — normal timer countdown
  - `apply_time_penalty` (NodePlugin, FixedUpdate, .after(NodeSystems::TickTimer)) — penalty drives to zero
- Receiver: `handle_timer_expired` (RunPlugin, FixedUpdate, .after(NodeSystems::TickTimer), .after(handle_node_cleared))
- Same-tick propagation guaranteed for tick_node_timer path. apply_time_penalty also runs before
  handle_timer_expired because handle_timer_expired is .after(NodeSystems::TickTimer) AND
  apply_time_penalty is also .after(NodeSystems::TickTimer). No explicit ordering between
  apply_time_penalty and handle_timer_expired — potential 1-tick delay (see known-conflicts.md).

### ApplyTimePenalty (BreakerPlugin/BehaviorPlugin → NodePlugin) — NEW cross-plugin message
- Sender chain:
  - `bridge_bolt_lost` or `bridge_bump` (BehaviorPlugin) → commands.trigger(TimePenaltyRequested)
    → `handle_time_penalty` observer (immediate) → writes ApplyTimePenalty
- Receiver: `apply_time_penalty` (NodePlugin, FixedUpdate, .after(NodeSystems::TickTimer))
- Cross-plugin boundary: BehaviorPlugin (inside BreakerPlugin) → NodePlugin

### SpawnAdditionalBolt (BreakerPlugin/BehaviorPlugin → BoltPlugin) — NEW cross-plugin message
- Sender chain:
  - `bridge_bump` (BehaviorPlugin) → commands.trigger(SpawnBoltRequested)
    → `handle_spawn_bolt_requested` observer (immediate) → writes SpawnAdditionalBolt
- Receiver: `spawn_additional_bolt` (BoltPlugin, FixedUpdate, .after(PhysicsSystems::BreakerCollision))
- Cross-plugin boundary: BehaviorPlugin (inside BreakerPlugin) → BoltPlugin

### RunLost (BehaviorPlugin → RunPlugin)
- Sender: `handle_life_lost` observer (immediate on LoseLifeRequested)
- Receiver: `handle_run_lost` (RunPlugin, .after(handle_node_cleared), .after(handle_timer_expired))
- PREVIOUSLY UNORDERED — now fixed in run/plugin.rs.

### UpgradeSelected (UiPlugin — registered, not yet used)
- Sender: NONE (future phases)
- Receivers: NONE

### AppExit (Bevy built-in)
- Sender: `handle_main_menu_input` (ScreenPlugin)
- Receiver: Bevy app exit handler

---

## Cross-Plugin Boundary Summary

| From Plugin | Message | To Plugin |
|-------------|---------|-----------|
| Bevy input | KeyboardInput | InputPlugin |
| BreakerPlugin | BumpPerformed | BoltPlugin, BehaviorPlugin, DebugPlugin |
| BreakerPlugin | BumpWhiffed | DebugPlugin |
| PhysicsPlugin | BoltHitBreaker | BreakerPlugin |
| PhysicsPlugin | BoltHitCell | CellsPlugin |
| PhysicsPlugin | BoltLost | BoltPlugin, BehaviorPlugin |
| CellsPlugin | CellDestroyed | NodePlugin |
| NodePlugin | NodeCleared | RunPlugin |
| NodePlugin | TimerExpired | RunPlugin |
| BehaviorPlugin | ApplyTimePenalty | NodePlugin (NEW) |
| BehaviorPlugin | SpawnAdditionalBolt | BoltPlugin (NEW) |
| BehaviorPlugin | RunLost | RunPlugin |

---

## Observer → Message Chains (all in BehaviorPlugin)

```
BoltLost message
  → bridge_bolt_lost system (.after(BoltLost set), conditional)
    → LoseLifeRequested observer trigger
      → handle_life_lost observer (immediate)
        → LivesCount decremented
        → RunLost message (when lives == 0)
    → TimePenaltyRequested observer trigger
      → handle_time_penalty observer (immediate)
        → ApplyTimePenalty message

BumpPerformed message
  → bridge_bump system (.after(BreakerCollision), conditional)
    → SpawnBoltRequested observer trigger
      → handle_spawn_bolt_requested observer (immediate)
        → SpawnAdditionalBolt message
    → TimePenaltyRequested observer trigger
      → handle_time_penalty observer (immediate)
        → ApplyTimePenalty message
```

All three observers (handle_life_lost, handle_time_penalty, handle_spawn_bolt_requested) run
immediately when triggered — their messages are available in the same FixedUpdate tick.

---

## Notes
- All gameplay message flow is strictly one-way (no circular chains)
- TimerExpired now has two senders: tick_node_timer and apply_time_penalty
- handle_timer_expired reads both sources transparently (single MessageReader)
- ApplyTimePenalty and SpawnAdditionalBolt are new cross-plugin boundaries, both
  routed through observer → message pattern (same as RunLost)
- UpgradeSelected remains registered but unused (future phases)
