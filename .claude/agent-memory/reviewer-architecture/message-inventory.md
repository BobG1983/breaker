# Message Inventory

| Message | Defined In | Registered By | Written By | Consumed By (actual) |
|---------|-----------|---------------|------------|---------------------|
| `BoltHitBreaker` | `bolt/messages.rs` | `BoltPlugin` | bolt/bolt_breaker_collision | breaker/grade_bump, effect/bridge_breaker_impact, run/detect_combo_and_pinball |
| `BoltHitCell` | `bolt/messages.rs` | `BoltPlugin` | bolt/bolt_cell_collision | effect/bridge_cell_impact, run/detect_combo_and_pinball |
| `BoltHitWall` | `bolt/messages.rs` | `BoltPlugin` | bolt/bolt_cell_collision | effect/bridge_wall_impact |
| `BoltLost` | `bolt/messages.rs` | `BoltPlugin` | bolt/bolt_lost | bolt/spawn_bolt_lost_text, effect/bridge_bolt_lost, run/track_bolts_lost |
| `DamageCell { cell, damage, source_bolt }` | `cells/messages.rs` | `CellsPlugin` | bolt/bolt_cell_collision, effect/effects/shockwave | cells/handle_cell_hit |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | cells/handle_cell_hit | run/track_node_completion, effect/bridge_cell_destroyed, run/track_cells_destroyed, run/detect_mass_destruction, run/detect_combo_and_pinball |
| `NodeCleared` | `run/node/messages.rs` | `NodePlugin` | run/node/track_node_completion | run/handle_node_cleared, run/track_node_cleared_stats, run/detect_nail_biter |
| `TimerExpired` | `run/node/messages.rs` | `NodePlugin` | run/node/tick_node_timer, run/node/apply_time_penalty | run/handle_timer_expired |
| `ApplyTimePenalty { seconds }` | `run/node/messages.rs` | `NodePlugin` | effect/time_penalty (observer) | run/node/apply_time_penalty |
| `SpawnAdditionalBolt` | `bolt/messages.rs` | `BoltPlugin` | effect/spawn_bolt (observer) | bolt/spawn_additional_bolt |
| `RunLost` | `run/messages.rs` | `RunPlugin` | effect/handle_life_lost | run/handle_run_lost |
| `BumpPerformed { grade, bolt }` | `breaker/messages.rs` | `BreakerPlugin` | breaker/update_bump, breaker/grade_bump | breaker/perfect_bump_dash_cancel, breaker/spawn_bump_grade_text, effect/bridge_bump, run/track_bumps, run/detect_close_save |
| `BumpWhiffed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/grade_bump | breaker/spawn_whiff_text, effect/bridge_bump_whiff |
| `ChipSelected { name }` | `ui/messages.rs` | `UiPlugin` | screen/chip_select/handle_chip_input | chips/apply_chip_effect, run/track_chips_collected, run/detect_first_evolution |
| `HighlightTriggered { kind }` | `run/messages.rs` | `RunPlugin` | run/detect_mass_destruction, run/detect_close_save, run/detect_combo_and_pinball, run/detect_nail_biter, run/detect_first_evolution, run/track_node_cleared_stats (declared but unused) | run/spawn_highlight_text (imported but NOT registered — BLOCKING) |
| `BoltSpawned` | `bolt/messages.rs` | `BoltPlugin` | bolt/spawn_bolt | run/node/check_spawn_complete |
| `BreakerSpawned` | `breaker/messages.rs` | `BreakerPlugin` | breaker/spawn_breaker | run/node/check_spawn_complete |
| `WallsSpawned` | `wall/messages.rs` | `WallPlugin` | wall/spawn_walls | run/node/check_spawn_complete |
| `CellsSpawned` | `run/node/messages.rs` | `NodePlugin` | run/node/spawn_cells_from_layout | run/node/check_spawn_complete |
| `SpawnNodeComplete` | `run/node/messages.rs` | `NodePlugin` | run/node/check_spawn_complete | scenario-runner/check_no_entity_leaks |

## Ownership Note
`RunLost`, `ApplyTimePenalty`, `SpawnAdditionalBolt`, and `DamageCell` all deviate from the sender-owns convention: each is defined in the consuming domain but sent by other domains. Accepted because the message semantically belongs to the consumer's vocabulary. `DamageCell` is a "command" message — the cells domain defines the damage API, bolt/collision systems and effect/effects/shockwave both call it. This is now a consistent pattern for all command-to-domain messages.

## Observer Events (intra-domain, not Messages)
| Event | Domain | Triggered By | Observed By |
|-------|--------|-------------|-------------|
| Per-effect typed events (ShockwaveFired, LoseLifeFired, TimePenaltyFired, SpawnBoltsFired, SpeedBoostFired, ChainBoltFired, MultiBoltFired, ShieldFired, etc.) | effect | bridge_* systems via fire_typed_event() in effect/triggers/ | effect/effects/* handlers (one handler per event type) |
| `ChipEffectApplied { effect, max_stacks }` | chips | apply_chip_effect | chips/effects/* passive handlers |

NOTE (2026-03-21): ConsequenceFired REMOVED. OverclockEffectFired unified into EffectFired. bolt/behaviors/ DELETED. behaviors/consequences/ DELETED.
NOTE (2026-03-24): physics/ game domain DELETED. BoltHitBreaker/BoltHitCell/BoltHitWall/BoltLost moved from physics/messages.rs to bolt/messages.rs. PhysicsPlugin → BoltPlugin as registering plugin.
NOTE (2026-03-25, C7-R): EffectFired DELETED. behaviors/ renamed to effect/. Per-effect typed events replace EffectFired. fire_typed_event() is the new dispatch function in effect/typed_events.rs.

## Spawn Signal Pattern (added 2026-03-18)
Four domain-owned spawn signals (`BoltSpawned`, `BreakerSpawned`, `WallsSpawned`, `CellsSpawned`) converge in `run/node/check_spawn_complete`, which aggregates them via a `Local<SpawnChecklist>` bitfield and fires `SpawnNodeComplete` when all four arrive. Currently consumed only by the scenario runner for entity-leak baseline sampling. Known issue: `ScenarioLifecycle` redundantly calls `add_message::<SpawnNodeComplete>()` despite `NodePlugin` already registering it.
