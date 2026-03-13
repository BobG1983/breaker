# Message Inventory

| Message | Defined In | Registered By | Written By | Consumed By (actual) |
|---------|-----------|---------------|------------|---------------------|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_breaker_collision | breaker/grade_bump, (future: audio, upgrades, UI) |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_cell_collision | cells/handle_cell_hit, (future: upgrades, audio) |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_lost | bolt/spawn_bolt_lost_text, (future: breaker penalty) |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | cells/handle_cell_hit | run/track_node_completion, (future: upgrades, audio) |
| `NodeCleared` | `run/messages.rs` | `RunPlugin` | run/node/track_node_completion | run/handle_node_cleared |
| `TimerExpired` | `run/messages.rs` | `RunPlugin` | run/node/tick_node_timer | run/handle_timer_expired |
| `BumpPerformed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/update_bump, breaker/grade_bump | bolt/apply_bump_velocity, breaker/perfect_bump_dash_cancel, breaker/spawn_bump_grade_text |
| `BumpWhiffed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/grade_bump | breaker/spawn_whiff_text |
| `UpgradeSelected` | `ui/messages.rs` | `UiPlugin` | (not yet) | (future: upgrades) |
