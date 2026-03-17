# Message Inventory

| Message | Defined In | Registered By | Written By | Consumed By (actual) |
|---------|-----------|---------------|------------|---------------------|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_breaker_collision | breaker/grade_bump, (future: audio, upgrades, UI) |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_cell_collision | cells/handle_cell_hit, (future: upgrades, audio) |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_lost | bolt/spawn_bolt_lost_text, behaviors/bridge_bolt_lost |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | cells/handle_cell_hit | run/track_node_completion, (future: upgrades, audio) |
| `NodeCleared` | `run/node/messages.rs` | `NodePlugin` | run/node/track_node_completion | run/handle_node_cleared |
| `TimerExpired` | `run/node/messages.rs` | `NodePlugin` | run/node/tick_node_timer, run/node/apply_time_penalty | run/handle_timer_expired |
| `ApplyTimePenalty { seconds }` | `run/node/messages.rs` | `NodePlugin` | behaviors/time_penalty (observer) | run/node/apply_time_penalty |
| `SpawnAdditionalBolt` | `bolt/messages.rs` | `BoltPlugin` | behaviors/spawn_bolt (observer) | bolt/spawn_additional_bolt |
| `RunLost` | `run/messages.rs` | `RunPlugin` | behaviors/handle_life_lost | run/handle_run_lost |
| `BumpPerformed { grade, multiplier }` | `breaker/messages.rs` | `BreakerPlugin` | breaker/update_bump, breaker/grade_bump | bolt/apply_bump_velocity, breaker/perfect_bump_dash_cancel, breaker/spawn_bump_grade_text, behaviors/bridge_bump |
| `BumpWhiffed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/grade_bump | breaker/spawn_whiff_text |
| `ChipSelected` | `ui/messages.rs` | `UiPlugin` | screen/chip_select/handle_chip_input | (future: chips) |

## Ownership Note
`RunLost`, `ApplyTimePenalty`, and `SpawnAdditionalBolt` all deviate from the sender-owns convention: each is defined in the consuming domain but sent by `behaviors` consequence observers. Accepted because the message semantically belongs to the consumer's vocabulary (run-lost is a run concept, apply-time-penalty is a node concept, spawn-additional-bolt is a bolt concept). The breaker behavior system is merely the trigger source. This is now a consistent pattern for all consequence-to-target messages.
