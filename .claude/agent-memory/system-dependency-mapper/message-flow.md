---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-12)
type: reference
---

# Message Flow Map

## Messages Registered

| Message | Registered By | Senders | Active Receivers |
|---------|--------------|---------|-----------------|
| `KeyboardInput` (Bevy built-in) | InputPlugin | Bevy InputPlugin | `read_input_actions` |
| `BumpPerformed` | BreakerPlugin | `update_bump`, `grade_bump` | `perfect_bump_dash_cancel`, `apply_bump_velocity`, `spawn_bump_grade_text` |
| `BoltHitBreaker` | PhysicsPlugin | `bolt_breaker_collision` | `grade_bump` |
| `BoltHitCell` | PhysicsPlugin | `bolt_cell_collision` | `handle_cell_hit` |
| `BoltLost` | PhysicsPlugin | `bolt_lost` | `spawn_bolt_lost_text` |
| `CellDestroyed` | CellsPlugin | `handle_cell_hit` | **NO ACTIVE RECEIVERS** ⚠️ |
| `NodeCleared` | RunPlugin | **NO SENDERS** ⚠️ | **NO ACTIVE RECEIVERS** ⚠️ |
| `TimerExpired` | RunPlugin | **NO SENDERS** ⚠️ | **NO ACTIVE RECEIVERS** ⚠️ |
| `UpgradeSelected` | UiPlugin | **NO SENDERS** ⚠️ | **NO ACTIVE RECEIVERS** ⚠️ |
| `AppExit` (Bevy built-in) | ScreenPlugin | `handle_main_menu_input` | Bevy runtime |

## Detailed Flow

### BumpPerformed
```
update_bump ──(BumpGrade::Timeout)──────────────────────────┐
grade_bump  ──(BumpGrade::{Early,Perfect,Late,None})─────────┼──► perfect_bump_dash_cancel (breaker)
                                                             ├──► apply_bump_velocity       (bolt)
                                                             └──► spawn_bump_grade_text     (breaker)
```

### BoltHitBreaker
```
bolt_breaker_collision (physics) ──► grade_bump (breaker)
```
NOTE: BoltHitBreaker comment says "consumed by audio, upgrades, UI" — but audio/upgrades/UI are stubs with no receivers.

### BoltHitCell
```
bolt_cell_collision (physics) ──► handle_cell_hit (cells)
```
NOTE: BoltHitCell comment says "consumed by upgrades, cells, and audio" — only cells receives it currently.

### BoltLost
```
bolt_lost (physics) ──► spawn_bolt_lost_text (bolt)
```
NOTE: BoltLost comment says "consumed by breaker (applies penalty per breaker type)" — penalty not yet implemented.

### CellDestroyed
```
handle_cell_hit (cells) ──► [NO RECEIVERS]
```
Comment says "consumed by run (progress tracking), upgrades (overclock triggers), audio" — these are future Phase 2+ systems.

### NodeCleared / TimerExpired / UpgradeSelected
These are future Phase 2+ messages. Registered but have no senders or receivers yet.

## Cross-Plugin Message Boundaries

These are the intentional cross-domain message flows:

| From Domain | Message | To Domain |
|-------------|---------|-----------|
| physics | BoltHitBreaker | breaker |
| physics | BoltHitCell | cells |
| physics | BoltLost | bolt |
| breaker | BumpPerformed | bolt (apply_bump_velocity), breaker (perfect_bump_dash_cancel, spawn_bump_grade_text) |
| cells | CellDestroyed | (future: run, upgrades, audio) |
| screen | AppExit | Bevy runtime |
