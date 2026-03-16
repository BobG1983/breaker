---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-16 post-cleanup re-scan)
type: reference
---

# Message Flow Map

Last updated: 2026-03-16 (post-architecture-cleanup re-scan, Bevy 0.18.1)

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
  - `update_bump` (BreakerPlugin, retroactive path — bolt hit before button press)
  - `grade_bump` (BreakerPlugin, forward/same-frame path — button pressed before/during hit)
- Receivers:
  - `perfect_bump_dash_cancel` (BreakerPlugin) — cancels dash on Perfect grade
  - `spawn_bump_grade_text` (BreakerPlugin) — spawns grade feedback text
  - `apply_bump_velocity` (BoltPlugin) — amplifies bolt speed using embedded multiplier
  - `bridge_bump` (BehaviorPlugin, conditional) — triggers archetype consequence events
  - `track_bump_result` (DebugPlugin, dev only) — updates debug display
- Key change: multiplier is now embedded in the message (set by update_bump/grade_bump using
  BumpTimingQuery/BumpGradingQuery). apply_bump_velocity no longer queries Breaker components.
- Missing consumers (future phases): AudioPlugin, UiPlugin (HUD)

### BumpWhiffed (BreakerPlugin → cross-domain)
- Sender: `grade_bump` (BreakerPlugin)
- Receivers:
  - `spawn_whiff_text` (BreakerPlugin) — spawns WHIFF text
  - `track_bump_result` (DebugPlugin, dev only)
- Missing consumers (future phases): AudioPlugin

### BoltHitBreaker (PhysicsPlugin → cross-domain)
- Sender: `bolt_breaker_collision` (PhysicsPlugin)
- Receivers:
  - `grade_bump` (BreakerPlugin) — resolves bump timing
- Missing consumers (future phases): AudioPlugin, UpgradesPlugin, UiPlugin

### BoltHitCell (PhysicsPlugin → cross-domain)
- Sender: `bolt_cell_collision` (PhysicsPlugin)
- Receivers:
  - `handle_cell_hit` (CellsPlugin) — deals damage, triggers despawn
- Missing consumers (future phases): UpgradesPlugin, AudioPlugin

### BoltLost (PhysicsPlugin → cross-domain)
- Sender: `bolt_lost` (PhysicsPlugin, PhysicsSystems::BoltLost)
- Receivers:
  - `spawn_bolt_lost_text` (BoltPlugin) — spawns BOLT LOST feedback text
  - `bridge_bolt_lost` (BehaviorPlugin, conditional) — triggers LoseLifeRequested observer
- Missing consumers (future phases): AudioPlugin, UiPlugin

### CellDestroyed (CellsPlugin → RunPlugin/NodePlugin)
- Sender: `handle_cell_hit` (CellsPlugin, FixedUpdate, no ordering vs receiver)
- Receivers:
  - `track_node_completion` (NodePlugin, FixedUpdate, NodeSystems::TrackCompletion)
- Ordering concern: NO explicit ordering between sender and receiver across plugin boundary.
  Messages persist across frames so a one-tick delay is safe.

### NodeCleared (NodePlugin internal → RunPlugin)
- Sender: `track_node_completion` (NodePlugin, FixedUpdate, NodeSystems::TrackCompletion)
- Receiver: `handle_node_cleared` (RunPlugin, FixedUpdate, .after(NodeSystems::TrackCompletion))
- Same-tick propagation guaranteed by ordering constraint.

### TimerExpired (NodePlugin internal → RunPlugin)
- Sender: `tick_node_timer` (NodePlugin, FixedUpdate, NodeSystems::TickTimer)
- Receiver: `handle_timer_expired` (RunPlugin, FixedUpdate, .after(NodeSystems::TickTimer))
- Same-tick propagation guaranteed by ordering constraint.

### RunLost (BehaviorPlugin → RunPlugin — NEW cross-plugin boundary)
- Sender: `handle_life_lost` (BehaviorPlugin observer, immediate on LoseLifeRequested)
  - Triggered by: `bridge_bolt_lost` (BehaviorPlugin, .after(PhysicsSystems::BoltLost))
- Receiver: `handle_run_lost` (RunPlugin, FixedUpdate, unordered vs handle_node_cleared)
- Note: `handle_life_lost` is an observer, not a scheduled system. It runs immediately when
  `LoseLifeRequested` event is triggered, before the next scheduled system. RunLost message
  is available in the same FixedUpdate tick.
- Ordering concern: `handle_run_lost` has no ordering vs `handle_node_cleared` or
  `handle_timer_expired`. See known-conflicts.md for analysis.

### UpgradeSelected (UiPlugin — registered, not yet used)
- Sender: NONE (future phases — upgrade select screen)
- Receivers: NONE

### AppExit (Bevy built-in)
- Sender: `handle_main_menu_input` (ScreenPlugin) — on Quit selection
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
| BehaviorPlugin | RunLost | RunPlugin |

---

## Observer → Message Chain (new pattern in BehaviorPlugin)

```
BoltLost message
  → bridge_bolt_lost system (.after(BoltLost set), conditional)
    → LoseLifeRequested event (Bevy observer trigger)
      → handle_life_lost observer (immediate)
        → LivesCount decremented
        → RunLost message sent (when lives == 0)
          → handle_run_lost system (next scheduled slot, unordered vs win conditions)
```

This is the only observer-mediated message chain in the codebase. All other message flows
are direct system-to-system via MessageReader/MessageWriter.

---

## Notes
- All gameplay message flow is strictly one-way (no circular message chains)
- BumpPerformed now carries multiplier in the message body — no cross-domain component access needed
- RunLost is a new cross-plugin message boundary: BehaviorPlugin → RunPlugin
- NodeCleared and TimerExpired now have same-tick propagation (ordering constraints added)
- UpgradeSelected remains registered but unused (future phases)
