---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-13 full re-scan)
type: reference
---

# Message Flow Map

Last updated: 2026-03-13 (full re-scan, Bevy 0.18.1)

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
- Senders: `update_bump` (retroactive path), `grade_bump` (forward/forward+hit path)
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
- Sender: `handle_cell_hit` (CellsPlugin)
- Receivers: NONE currently active
- Missing consumers (future phases): RunPlugin (progress tracking, NodeCleared detection), UpgradesPlugin, AudioPlugin

### NodeCleared (RunPlugin — registered, not yet used)
- Sender: NONE
- Receivers: NONE

### TimerExpired (RunPlugin — registered, not yet used)
- Sender: NONE
- Receivers: NONE

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
| CellsPlugin | CellDestroyed | (no active consumer) |

---

## Notes
- All gameplay message flow is strictly one-way (no circular message chains)
- CellDestroyed is the only actively-sent message with no current consumer
- NodeCleared, TimerExpired, UpgradeSelected are registered but have no sender or receiver yet (future phases)
