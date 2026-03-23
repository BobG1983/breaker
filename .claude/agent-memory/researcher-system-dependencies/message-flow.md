---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-19 post-spawn-coordinator additions)
type: reference
---

# Message Flow Map

Last updated: 2026-03-23 (Wave 4+ audit: HighlightTriggered message added — RunPlugin, emitted by 5 detection systems, consumed by spawn_highlight_text. Prior: 2026-03-22 Wave 3 audit — bridge names finalized; 2026-03-21 refactor/unify-behaviors — OverclockEffectFired→EffectFired, etc.)

## Registered Messages (by plugin)

| Message | Registered by |
|---------|--------------|
| KeyboardInput | Bevy built-in (InputPlugin reads it) |
| BumpPerformed | BreakerPlugin |
| BumpWhiffed | BreakerPlugin |
| BreakerSpawned | BreakerPlugin |
| BoltHitBreaker | PhysicsPlugin |
| BoltHitCell | PhysicsPlugin |
| BoltHitWall | PhysicsPlugin |
| BoltLost | PhysicsPlugin |
| DamageCell | CellsPlugin |
| CellDestroyed | CellsPlugin |
| NodeCleared | NodePlugin (RunPlugin sub-plugin) |
| TimerExpired | NodePlugin (RunPlugin sub-plugin) |
| ApplyTimePenalty | NodePlugin (RunPlugin sub-plugin) |
| CellsSpawned | NodePlugin (RunPlugin sub-plugin) |
| SpawnNodeComplete | NodePlugin (RunPlugin sub-plugin) |
| SpawnAdditionalBolt | BoltPlugin |
| BoltSpawned | BoltPlugin |
| WallsSpawned | WallPlugin |
| RunLost | RunPlugin |
| HighlightTriggered | RunPlugin |
| ChipSelected | UiPlugin |
| AppExit | Bevy built-in |

---

## Message Flow Detail

### Spawn Coordination Cluster (cross-plugin → NodePlugin coordinator)

**BoltSpawned** (BoltPlugin → NodePlugin)
- Sender: `spawn_bolt` (BoltPlugin, OnEnter(GameState::Playing))
- Receiver: `check_spawn_complete` (NodePlugin, FixedUpdate — coordinator)
- Sent even when bolt already exists (not possible for baseline bolt, always spawned fresh)

**BreakerSpawned** (BreakerPlugin → NodePlugin)
- Sender: `spawn_breaker` (BreakerPlugin, OnEnter(GameState::Playing))
- Receiver: `check_spawn_complete` (NodePlugin, FixedUpdate — coordinator)
- NOTE: Sent even when breaker already exists (cross-node persistence) — the `spawn_breaker` idempotency guard also sends BreakerSpawned on the no-op path

**CellsSpawned** (NodePlugin internal)
- Sender: `spawn_cells_from_layout` (NodePlugin, OnEnter(GameState::Playing))
- Receiver: `check_spawn_complete` (NodePlugin, FixedUpdate — coordinator)

**WallsSpawned** (WallPlugin → NodePlugin)
- Sender: `spawn_walls` (WallPlugin, OnEnter(GameState::Playing))
- Receiver: `check_spawn_complete` (NodePlugin, FixedUpdate — coordinator)

**SpawnNodeComplete** (NodePlugin → scenario runner)
- Sender: `check_spawn_complete` (NodePlugin, FixedUpdate — fires when all 4 signals received)
- Receiver: `check_no_entity_leaks` (scenario runner, FixedUpdate — uses as baseline trigger)
- NOTE: No gameplay receiver — purely for scenario runner baseline sampling.
- NOTE: Registered by both NodePlugin and ScenarioLifecycle (the latter registers it in the scenario runner to listen across the crate boundary).

**Timing note:** All 4 domain spawn signals come from OnEnter(GameState::Playing) systems using
Commands (deferred). The entities/resources are committed to the world at the end of the OnEnter
schedule. `check_spawn_complete` runs in FixedUpdate and must receive the messages; since
OnEnter is fully flushed before the first FixedUpdate tick of the Playing state, the messages
written in OnEnter are available in the first FixedUpdate tick.

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
  - `bridge_bump` (BehaviorsPlugin) — evaluates chains via TriggerKind::PerfectBump/EarlyBump/LateBump/BumpSuccess → fires EffectFired → effect observers (including handle_speed_boost for SpeedBoost leaf)
  - `track_bump_result` (DebugPlugin, dev only)
  - NOTE (2026-03-21): `apply_bump_velocity` (BoltPlugin) DELETED — velocity scaling now handled by TriggerChain::SpeedBoost leaf via EffectFired/handle_speed_boost

### BumpWhiffed (BreakerPlugin → cross-domain)
- Sender: `grade_bump` (BreakerPlugin)
- Receivers:
  - `spawn_whiff_text` (BreakerPlugin)
  - `bridge_bump_whiff` (BehaviorsPlugin) — evaluates chains via TriggerKind::BumpWhiff → fires EffectFired
  - `track_bump_result` (DebugPlugin, dev only)

### BoltHitBreaker (PhysicsPlugin → cross-domain)
- Sender: `bolt_breaker_collision` (PhysicsPlugin)
- Receivers:
  - `grade_bump` (BreakerPlugin)
  - `bridge_breaker_impact` (BehaviorsPlugin, FixedUpdate)

### BoltHitCell (PhysicsPlugin → cross-domain)
- Sender: `bolt_cell_collision` (PhysicsPlugin)
- Receivers:
  - `bridge_cell_impact` (BehaviorsPlugin, FixedUpdate) — evaluates overclock trigger chains
- NOTE: `BoltHitCell` carries `{ cell: Entity, bolt: Entity }`. `handle_cell_hit` now reads `DamageCell` (not BoltHitCell) — see DamageCell entry below.

### DamageCell (CellsPlugin-owned, sent by physics + shockwave)
- Senders:
  - `bolt_cell_collision` (PhysicsPlugin) — one per cell hit (alongside BoltHitCell)
  - `handle_shockwave` (BehaviorsPlugin) — one per in-range non-locked cell
- Receiver: `handle_cell_hit` (CellsPlugin, FixedUpdate)
- NOTE: Consumer-owns pattern — cells defines the damage API, physics and shockwave call it.

### BoltHitWall (PhysicsPlugin → cross-domain)
- Sender: `bolt_cell_collision` (PhysicsPlugin) — sent when bolt hits a wall entity
- Receiver: `bridge_wall_impact` (BehaviorsPlugin, FixedUpdate)

### BoltLost (PhysicsPlugin → cross-domain)
- Sender: `bolt_lost` (PhysicsPlugin, PhysicsSystems::BoltLost) — fires for baseline AND ExtraBolt
- Receivers:
  - `spawn_bolt_lost_text` (BoltPlugin)
  - `bridge_bolt_lost` (BehaviorsPlugin, FixedUpdate) — evaluates all chains via TriggerKind::BoltLost → fires EffectFired → effect observers (handles both old bridge_bolt_lost consequence and new overclock chains in one unified bridge)

### CellDestroyed (CellsPlugin → RunPlugin/NodePlugin)
- Sender: `handle_cell_hit` (CellsPlugin, FixedUpdate, no ordering vs receiver)
- Receivers:
  - `track_node_completion` (NodePlugin, FixedUpdate, NodeSystems::TrackCompletion)
  - `bridge_cell_destroyed` (BehaviorsPlugin, FixedUpdate)
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

### ApplyTimePenalty (BehaviorsPlugin → NodePlugin) — cross-plugin message
- Sender chain:
  - `bridge_bolt_lost` or `bridge_bump` (BehaviorsPlugin) → commands.trigger(EffectFired { effect: TriggerChain::TimePenalty { .. }, bolt })
    → `handle_time_penalty` observer (immediate) → writes ApplyTimePenalty
- Receiver: `apply_time_penalty` (NodePlugin, FixedUpdate, .after(NodeSystems::TickTimer))
- Cross-plugin boundary: BehaviorsPlugin (standalone) → NodePlugin

### SpawnAdditionalBolt (BehaviorsPlugin → BoltPlugin) — cross-plugin message
- Sender chain:
  - `bridge_bump` (BehaviorsPlugin) → commands.trigger(EffectFired { effect: TriggerChain::SpawnBolt, bolt })
    → `handle_spawn_bolt` observer (immediate) → writes SpawnAdditionalBolt
- Receiver: `spawn_additional_bolt` (BoltPlugin, FixedUpdate, .after(BehaviorSystems::Bridge))
- Cross-plugin boundary: BehaviorsPlugin (standalone) → BoltPlugin
- Ordering: spawn_additional_bolt runs AFTER BehaviorSystems::Bridge set — same-tick guarantee

### RunLost (BehaviorsPlugin → RunPlugin)
- Sender: `handle_life_lost` observer (immediate on EffectFired { effect: TriggerChain::LoseLife, .. })
- Receiver: `handle_run_lost` (RunPlugin, .after(handle_node_cleared), .after(handle_timer_expired))
- Cross-plugin boundary: BehaviorsPlugin (standalone) → RunPlugin

### ChipSelected (UiPlugin → ChipsPlugin)
- Sender: `handle_chip_input` (ChipSelectPlugin/ScreenPlugin, Update, run_if(GameState::ChipSelect)) — sent on confirm keypress with chip name only (`{ name: String }`)
- Receiver: `apply_chip_effect` (ChipsPlugin, Update, run_if(GameState::ChipSelect)) — reads ChipSelected, triggers ChipEffectApplied observer event
- NOTE: Previously called UpgradeSelected; renamed to ChipSelected to match game vocabulary.
- NOTE: handle_chip_input reads ButtonInput<KeyCode> directly (not InputActions). This is intentional — same pattern as main menu.

### HighlightTriggered (RunPlugin internal)
- Senders (all FixedUpdate PlayingState::Active or Update ChipSelect):
  - `detect_mass_destruction` (RunPlugin, FixedUpdate)
  - `detect_close_save` (RunPlugin, FixedUpdate, .after(BreakerCollision))
  - `detect_combo_and_pinball` (RunPlugin, FixedUpdate) — emits ComboKing and/or PinballWizard
  - `detect_nail_biter` (RunPlugin, FixedUpdate, .after(NodeSystems::TrackCompletion))
  - `detect_first_evolution` (RunPlugin, Update, ChipSelect state)
- Receiver: `spawn_highlight_text` (RunPlugin, Update, PlayingState::Active) — spawns Text2d popup per message
- All detection systems also record to RunStats.highlights (bounded by HighlightConfig.highlight_cap)
- HighlightTriggered is always emitted on detection regardless of cap (juice fires even when highlight list is full)

### AppExit (Bevy built-in)
- Sender: `handle_main_menu_input` (ScreenPlugin)
- Receiver: Bevy app exit handler

---

## Cross-Plugin Boundary Summary

| From Plugin | Message | To Plugin |
|-------------|---------|-----------|
| Bevy input | KeyboardInput | InputPlugin |
| BreakerPlugin | BumpPerformed | BoltPlugin, BehaviorsPlugin, DebugPlugin |
| BreakerPlugin | BumpWhiffed | BehaviorsPlugin, DebugPlugin |
| BreakerPlugin | BreakerSpawned | NodePlugin (coordinator) |
| BoltPlugin | BoltSpawned | NodePlugin (coordinator) |
| WallPlugin | WallsSpawned | NodePlugin (coordinator) |
| NodePlugin | CellsSpawned | NodePlugin (coordinator — self-message) |
| NodePlugin | SpawnNodeComplete | ScenarioRunner |
| PhysicsPlugin | BoltHitBreaker | BreakerPlugin, BehaviorsPlugin |
| PhysicsPlugin | BoltHitCell | BehaviorsPlugin |
| PhysicsPlugin | BoltHitWall | BehaviorsPlugin |
| PhysicsPlugin | BoltLost | BoltPlugin, BehaviorsPlugin |
| CellsPlugin | DamageCell | CellsPlugin (handle_cell_hit) |
| CellsPlugin | CellDestroyed | NodePlugin, BehaviorsPlugin |
| BehaviorsPlugin | DamageCell (via shockwave) | CellsPlugin |
| NodePlugin | NodeCleared | RunPlugin |
| NodePlugin | TimerExpired | RunPlugin |
| BehaviorsPlugin | ApplyTimePenalty | NodePlugin |
| BehaviorsPlugin | SpawnAdditionalBolt | BoltPlugin |
| BehaviorsPlugin | RunLost | RunPlugin |
| RunPlugin | HighlightTriggered | RunPlugin (spawn_highlight_text) |
| UiPlugin/ScreenPlugin | ChipSelected | ChipsPlugin |

---

## Observer → Message Chains (all in BehaviorsPlugin, unified 2026-03-21)

NOTE: ConsequenceFired is GONE. All leaf effects are now dispatched via EffectFired (was OverclockEffectFired).
Bridge systems call `commands.trigger(EffectFired { effect: TriggerChain::LeafVariant, bolt })`.
Effect observers pattern-match on EffectFired.effect to determine their leaf variant.

```
BoltLost message
  → bridge_bolt_lost (.after(BoltLost set), in_set(BehaviorSystems::Bridge))
    → evaluate(TriggerKind::BoltLost, chain) → Fire(leaf) or Arm
    → commands.trigger(EffectFired { effect: TriggerChain::LoseLife, bolt: None })
      → handle_life_lost observer → LivesCount decremented → RunLost message (when lives == 0)
    → commands.trigger(EffectFired { effect: TriggerChain::TimePenalty{seconds}, bolt: None })
      → handle_time_penalty observer → ApplyTimePenalty message

BumpPerformed message
  → bridge_bump (.after(BreakerSystems::GradeBump), in_set(BehaviorSystems::Bridge))
    → evaluate(TriggerKind::PerfectBump/EarlyBump/LateBump/BumpSuccess, chain) → Fire(leaf) or Arm
    → commands.trigger(EffectFired { effect: TriggerChain::SpawnBolt, bolt: Some(e) })
      → handle_spawn_bolt observer → SpawnAdditionalBolt message
    → commands.trigger(EffectFired { effect: TriggerChain::Shockwave{..}, bolt: Some(e) })
      → handle_shockwave observer → DamageCell messages (for cells in range)
```

All observers run immediately when triggered via commands.trigger(EffectFired).
Messages written by observers are available to downstream systems .after(BehaviorSystems::Bridge).

TriggerChain::SpeedBoost (was BoltSpeedBoost) is dispatched at runtime via EffectFired → handle_speed_boost in behaviors/effects/speed_boost.rs. It is NOT init-time-only. (Note: the old BoltSpeedBoost name was renamed to SpeedBoost { target, multiplier } in refactor/unify-behaviors.)

---

## Notes
- All gameplay message flow is strictly one-way (no circular chains)
- TimerExpired now has two senders: tick_node_timer and apply_time_penalty
- handle_timer_expired reads both sources transparently (single MessageReader)
- ApplyTimePenalty and SpawnAdditionalBolt are new cross-plugin boundaries, both
  routed through observer → message pattern (same as RunLost)
- ChipSelected (renamed from UpgradeSelected) is now received by chips/apply_chip_effect — fully active
