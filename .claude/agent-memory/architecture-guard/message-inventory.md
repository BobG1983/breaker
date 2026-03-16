# Message Inventory

| Message | Defined In | Registered By | Written By | Consumed By (actual) |
|---------|-----------|---------------|------------|---------------------|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_breaker_collision | breaker/grade_bump, (future: audio, upgrades, UI) |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_cell_collision | cells/handle_cell_hit, (future: upgrades, audio) |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_lost | bolt/spawn_bolt_lost_text, breaker/behaviors/bridge_bolt_lost |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | cells/handle_cell_hit | run/track_node_completion, (future: upgrades, audio) |
| `NodeCleared` | `run/node/messages.rs` | `NodePlugin` | run/node/track_node_completion | run/handle_node_cleared |
| `TimerExpired` | `run/node/messages.rs` | `NodePlugin` | run/node/tick_node_timer | run/handle_timer_expired |
| `RunLost` | `run/messages.rs` | `RunPlugin` | breaker/behaviors/handle_life_lost | run/handle_run_lost |
| `BumpPerformed { grade, multiplier }` | `breaker/messages.rs` | `BreakerPlugin` | breaker/update_bump, breaker/grade_bump | bolt/apply_bump_velocity, breaker/perfect_bump_dash_cancel, breaker/spawn_bump_grade_text, breaker/behaviors/bridge_bump |
| `BumpWhiffed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/grade_bump | breaker/spawn_whiff_text |
| `ChipSelected` | `ui/messages.rs` | `UiPlugin` | screen/chip_select/handle_chip_input | (future: chips) |

## Ownership Note
`RunLost` deviates from the sender-owns convention: defined in `run/messages.rs` (consumer) but sent by `breaker/behaviors`. Accepted because "run lost" is semantically a run-domain concept, and the breaker domain is merely the trigger source.
