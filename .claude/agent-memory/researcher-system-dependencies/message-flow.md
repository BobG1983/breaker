---
name: message-flow
description: Complete message flow map — who sends what, who receives what, cross-plugin boundaries, and messages with no consumers (as of 2026-03-19 post-spawn-coordinator additions)
type: reference
---

# Message Flow Map

Last updated: 2026-03-19 (new spawn coordination messages: BoltSpawned, BreakerSpawned, CellsSpawned, WallsSpawned, SpawnNodeComplete; check_spawn_complete coordinator in NodePlugin; check_no_entity_leaks now gated on SpawnNodeComplete)

## Registered Messages (by plugin)

| Message | Registered by |
|---------|--------------|
| KeyboardInput | Bevy built-in (InputPlugin reads it) |
| BumpPerformed | BreakerPlugin |
| BumpWhiffed | BreakerPlugin |
| BreakerSpawned | BreakerPlugin |
| BoltHitBreaker | PhysicsPlugin |
| BoltHitCell | PhysicsPlugin |
| BoltLost | PhysicsPlugin |
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
  - `apply_bump_velocity` (BoltPlugin)
  - `bridge_bump` (BehaviorsPlugin, conditional) — fires ConsequenceFired trigger → observers handle_time_penalty / handle_spawn_bolt
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
- NOTE: `BoltHitCell` carries `{ cell: Entity, bolt: Entity }` — the bolt field was re-added in feature/phase4b2-effect-consumption for pierce lookahead. Both fields are present and used.

### BoltLost (PhysicsPlugin → cross-domain)
- Sender: `bolt_lost` (PhysicsPlugin, PhysicsSystems::BoltLost) — fires for baseline AND ExtraBolt
- Receivers:
  - `spawn_bolt_lost_text` (BoltPlugin)
  - `bridge_bolt_lost` (BehaviorsPlugin, conditional) — fires ConsequenceFired trigger → observers handle_life_lost / handle_time_penalty

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

### ApplyTimePenalty (BehaviorsPlugin → NodePlugin) — cross-plugin message
- Sender chain:
  - `bridge_bolt_lost` or `bridge_bump` (BehaviorsPlugin) → commands.trigger(ConsequenceFired(TimePenalty))
    → `handle_time_penalty` observer (immediate) → writes ApplyTimePenalty
- Receiver: `apply_time_penalty` (NodePlugin, FixedUpdate, .after(NodeSystems::TickTimer))
- Cross-plugin boundary: BehaviorsPlugin (standalone) → NodePlugin

### SpawnAdditionalBolt (BehaviorsPlugin → BoltPlugin) — cross-plugin message
- Sender chain:
  - `bridge_bump` (BehaviorsPlugin) → commands.trigger(ConsequenceFired(SpawnBolt))
    → `handle_spawn_bolt` observer (immediate) → writes SpawnAdditionalBolt
- Receiver: `spawn_additional_bolt` (BoltPlugin, FixedUpdate, .after(BehaviorSystems::Bridge))
- Cross-plugin boundary: BehaviorsPlugin (standalone) → BoltPlugin
- Ordering: spawn_additional_bolt runs AFTER BehaviorSystems::Bridge set — same-tick guarantee

### RunLost (BehaviorsPlugin → RunPlugin)
- Sender: `handle_life_lost` observer (immediate on ConsequenceFired(LoseLife))
- Receiver: `handle_run_lost` (RunPlugin, .after(handle_node_cleared), .after(handle_timer_expired))
- Cross-plugin boundary: BehaviorsPlugin (standalone) → RunPlugin

### ChipSelected (UiPlugin → ChipsPlugin)
- Sender: `handle_chip_input` (ChipSelectPlugin/ScreenPlugin, Update, run_if(GameState::ChipSelect)) — sent on confirm keypress with chip name + kind
- Receiver: `apply_chip_effect` (ChipsPlugin, Update, run_if(GameState::ChipSelect)) — reads ChipSelected, triggers ChipEffectApplied observer event
- NOTE: Previously called UpgradeSelected; renamed to ChipSelected to match game vocabulary.
- NOTE: handle_chip_input reads ButtonInput<KeyCode> directly (not InputActions). This is intentional — same pattern as main menu.

### AppExit (Bevy built-in)
- Sender: `handle_main_menu_input` (ScreenPlugin)
- Receiver: Bevy app exit handler

---

## Cross-Plugin Boundary Summary

| From Plugin | Message | To Plugin |
|-------------|---------|-----------|
| Bevy input | KeyboardInput | InputPlugin |
| BreakerPlugin | BumpPerformed | BoltPlugin, BehaviorsPlugin, DebugPlugin |
| BreakerPlugin | BumpWhiffed | DebugPlugin |
| BreakerPlugin | BreakerSpawned | NodePlugin (coordinator) |
| BoltPlugin | BoltSpawned | NodePlugin (coordinator) |
| WallPlugin | WallsSpawned | NodePlugin (coordinator) |
| NodePlugin | CellsSpawned | NodePlugin (coordinator — self-message) |
| NodePlugin | SpawnNodeComplete | ScenarioRunner |
| PhysicsPlugin | BoltHitBreaker | BreakerPlugin |
| PhysicsPlugin | BoltHitCell | CellsPlugin |
| PhysicsPlugin | BoltLost | BoltPlugin, BehaviorsPlugin |
| CellsPlugin | CellDestroyed | NodePlugin |
| NodePlugin | NodeCleared | RunPlugin |
| NodePlugin | TimerExpired | RunPlugin |
| BehaviorsPlugin | ApplyTimePenalty | NodePlugin |
| BehaviorsPlugin | SpawnAdditionalBolt | BoltPlugin |
| BehaviorsPlugin | RunLost | RunPlugin |
| UiPlugin/ScreenPlugin | ChipSelected | ChipsPlugin |

---

## Observer → Message Chains (all in BehaviorsPlugin)

```
BoltLost message
  → bridge_bolt_lost system (.after(BoltLost set), in_set(BehaviorSystems::Bridge), conditional)
    → commands.trigger(ConsequenceFired(LoseLife))
      → handle_life_lost observer (immediate, pattern-matches Consequence::LoseLife)
        → LivesCount decremented
        → RunLost message (when lives == 0)
    → commands.trigger(ConsequenceFired(TimePenalty(seconds)))
      → handle_time_penalty observer (immediate, pattern-matches Consequence::TimePenalty)
        → ApplyTimePenalty message

BumpPerformed message
  → bridge_bump system (.after(BreakerCollision), in_set(BehaviorSystems::Bridge), conditional)
    → commands.trigger(ConsequenceFired(SpawnBolt))
      → handle_spawn_bolt observer (immediate, pattern-matches Consequence::SpawnBolt)
        → SpawnAdditionalBolt message
    → commands.trigger(ConsequenceFired(TimePenalty(seconds)))
      → handle_time_penalty observer (immediate, pattern-matches Consequence::TimePenalty)
        → ApplyTimePenalty message
```

All three observers (handle_life_lost, handle_time_penalty, handle_spawn_bolt) run immediately
when triggered via commands.trigger(ConsequenceFired). Messages written by observers are
available to downstream systems that run .after(BehaviorSystems::Bridge).

Consequence::BoltSpeedBoost is NEVER triggered at runtime — it is an init-time-only consequence
handled by apply_bolt_speed_boosts() called from init_archetype. The bridge skips it explicitly.

---

## Notes
- All gameplay message flow is strictly one-way (no circular chains)
- TimerExpired now has two senders: tick_node_timer and apply_time_penalty
- handle_timer_expired reads both sources transparently (single MessageReader)
- ApplyTimePenalty and SpawnAdditionalBolt are new cross-plugin boundaries, both
  routed through observer → message pattern (same as RunLost)
- ChipSelected (renamed from UpgradeSelected) is now received by chips/apply_chip_effect — fully active
