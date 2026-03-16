---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-16 full re-scan)
type: reference
---

# Message Flow Map

Last updated: 2026-03-16 (full re-scan, Bevy 0.18.1)

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
| NodeCleared | RunPlugin |
| TimerExpired | RunPlugin |
| UpgradeSelected | UiPlugin |
| AppExit | Bevy built-in |

---

## Message Flow Detail

### KeyboardInput (Bevy built-in)
- Sender: Bevy input system
- Receiver: `read_input_actions` (InputPlugin, PreUpdate)

### BumpPerformed (BreakerPlugin → cross-domain)
- Senders: `update_bump` (retroactive path), `grade_bump` (hit-grading path)
- Receivers:
  - `perfect_bump_dash_cancel` (BreakerPlugin) — cancels dash on Perfect grade
  - `spawn_bump_grade_text` (BreakerPlugin) — spawns grade feedback text
  - `apply_bump_velocity` (BoltPlugin) — amplifies bolt speed
  - `track_bump_result` (DebugPlugin, dev only) — updates debug display
- Missing consumers (future phases): AudioPlugin, UpgradesPlugin, UiPlugin (HUD)

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
- Sender: `bolt_lost` (PhysicsPlugin)
- Receivers:
  - `spawn_bolt_lost_text` (BoltPlugin) — spawns BOLT LOST feedback text
- Missing consumers (future phases): BreakerPlugin (penalty by archetype), AudioPlugin, UiPlugin

### CellDestroyed (CellsPlugin → cross-domain)
- Sender: `handle_cell_hit` (CellsPlugin, FixedUpdate, no ordering)
- Receivers:
  - `track_node_completion` (RunPlugin, FixedUpdate, no ordering) — decrements ClearRemainingCount, sends NodeCleared
- Ordering concern: NO explicit ordering between sender and receiver in FixedUpdate.
  Messages persist across frames so a one-tick delay is safe, but Bevy may run
  track_node_completion before handle_cell_hit on the same tick, deferring detection by one tick.

### NodeCleared (CellsPlugin→RunPlugin→RunPlugin internal)
- Sender: `track_node_completion` (RunPlugin, FixedUpdate)
- Receiver: `handle_node_cleared` (RunPlugin, FixedUpdate)
- Ordering concern: Both in the same unordered group in RunPlugin::build. Same-tick message
  from track_node_completion may not be read by handle_node_cleared until next tick.
  This is a 1-tick delay on node transition, imperceptible in practice.

### TimerExpired (RunPlugin internal)
- Sender: `tick_node_timer` (RunPlugin, FixedUpdate)
- Receiver: `handle_timer_expired` (RunPlugin, FixedUpdate)
- Ordering concern: Same as NodeCleared — both in the same unordered group.
  If tick_node_timer fires TimerExpired on tick N, handle_timer_expired may not
  read it until tick N+1. One tick delay on timer loss, imperceptible in practice.

### UpgradeSelected (UiPlugin — registered, not yet used)
- Sender: NONE
- Receivers: NONE

### AppExit (Bevy built-in)
- Sender: `handle_main_menu_input` (ScreenPlugin) — on Quit selection
- Receiver: Bevy app exit handler

---

## Cross-Plugin Boundary Summary

| From Plugin | Message | To Plugin |
|-------------|---------|-----------|
| Bevy input | KeyboardInput | InputPlugin |
| BreakerPlugin | BumpPerformed | BoltPlugin, DebugPlugin |
| BreakerPlugin | BumpWhiffed | DebugPlugin |
| PhysicsPlugin | BoltHitBreaker | BreakerPlugin |
| PhysicsPlugin | BoltHitCell | CellsPlugin |
| PhysicsPlugin | BoltLost | BoltPlugin |
| CellsPlugin | CellDestroyed | RunPlugin |
| RunPlugin | NodeCleared | RunPlugin (internal) |
| RunPlugin | TimerExpired | RunPlugin (internal) |

---

## Notes
- All gameplay message flow is strictly one-way (no circular message chains)
- CellDestroyed is now consumed by RunPlugin (was previously an orphan)
- NodeCleared and TimerExpired are now both sent and received (wired in Phase 2)
- UpgradeSelected remains registered but unused (future phases)
