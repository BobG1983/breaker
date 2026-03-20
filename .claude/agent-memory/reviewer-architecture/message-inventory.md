# Message Inventory

| Message | Defined In | Registered By | Written By | Consumed By (actual) |
|---------|-----------|---------------|------------|---------------------|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_breaker_collision | breaker/grade_bump, (future: audio, upgrades, UI) |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_cell_collision | cells/handle_cell_hit, bolt/behaviors/bridges/bridge_overclock_impact |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_lost | bolt/spawn_bolt_lost_text, behaviors/bridge_bolt_lost, bolt/behaviors/bridges/bridge_overclock_bolt_lost |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | cells/handle_cell_hit, **bolt/behaviors/effects/shockwave (VIOLATION)** | run/track_node_completion, bolt/behaviors/bridges/bridge_overclock_cell_destroyed |
| `NodeCleared` | `run/node/messages.rs` | `NodePlugin` | run/node/track_node_completion | run/handle_node_cleared |
| `TimerExpired` | `run/node/messages.rs` | `NodePlugin` | run/node/tick_node_timer, run/node/apply_time_penalty | run/handle_timer_expired |
| `ApplyTimePenalty { seconds }` | `run/node/messages.rs` | `NodePlugin` | behaviors/time_penalty (observer) | run/node/apply_time_penalty |
| `SpawnAdditionalBolt` | `bolt/messages.rs` | `BoltPlugin` | behaviors/spawn_bolt (observer) | bolt/spawn_additional_bolt |
| `RunLost` | `run/messages.rs` | `RunPlugin` | behaviors/handle_life_lost | run/handle_run_lost |
| `BumpPerformed { grade, multiplier, bolt }` | `breaker/messages.rs` | `BreakerPlugin` | breaker/update_bump, breaker/grade_bump | bolt/apply_bump_velocity, breaker/perfect_bump_dash_cancel, breaker/spawn_bump_grade_text, behaviors/bridge_bump, bolt/behaviors/bridges/bridge_overclock_bump |
| `BumpWhiffed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/grade_bump | breaker/spawn_whiff_text |
| `ChipSelected` | `ui/messages.rs` | `UiPlugin` | screen/chip_select/handle_chip_input | chips/apply_chip_effect |
| `BoltSpawned` | `bolt/messages.rs` | `BoltPlugin` | bolt/spawn_bolt | run/node/check_spawn_complete |
| `BreakerSpawned` | `breaker/messages.rs` | `BreakerPlugin` | breaker/spawn_breaker | run/node/check_spawn_complete |
| `WallsSpawned` | `wall/messages.rs` | `WallPlugin` | wall/spawn_walls | run/node/check_spawn_complete |
| `CellsSpawned` | `run/node/messages.rs` | `NodePlugin` | run/node/spawn_cells_from_layout | run/node/check_spawn_complete |
| `SpawnNodeComplete` | `run/node/messages.rs` | `NodePlugin` | run/node/check_spawn_complete | scenario-runner/check_no_entity_leaks |

## Ownership Note
`RunLost`, `ApplyTimePenalty`, and `SpawnAdditionalBolt` all deviate from the sender-owns convention: each is defined in the consuming domain but sent by `behaviors` consequence observers. Accepted because the message semantically belongs to the consumer's vocabulary (run-lost is a run concept, apply-time-penalty is a node concept, spawn-additional-bolt is a bolt concept). The breaker behavior system is merely the trigger source. This is now a consistent pattern for all consequence-to-target messages.

## Observer Events (intra-domain, not Messages)
| Event | Domain | Triggered By | Observed By |
|-------|--------|-------------|-------------|
| `ConsequenceFired(Consequence)` | behaviors | bridge_bolt_lost, bridge_bump, bridge_bump_whiff | consequences/* handlers |
| `ChipEffectApplied { effect, max_stacks }` | chips | apply_chip_effect | effects/* handlers |
| `OverclockEffectFired { effect, bolt }` | bolt/behaviors | bridges (bridge_overclock_*) | effects/shockwave (+ future effect handlers) |

## Spawn Signal Pattern (added 2026-03-18)
Four domain-owned spawn signals (`BoltSpawned`, `BreakerSpawned`, `WallsSpawned`, `CellsSpawned`) converge in `run/node/check_spawn_complete`, which aggregates them via a `Local<SpawnChecklist>` bitfield and fires `SpawnNodeComplete` when all four arrive. Currently consumed only by the scenario runner for entity-leak baseline sampling. Known issue: `ScenarioLifecycle` redundantly calls `add_message::<SpawnNodeComplete>()` despite `NodePlugin` already registering it.
