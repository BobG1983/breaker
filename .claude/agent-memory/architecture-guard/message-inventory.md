# Message Inventory

| Message | Defined In | Registered By | Written By | Consumed By (actual) |
|---------|-----------|---------------|------------|---------------------|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_breaker_collision | breaker/grade_bump, (future: audio, upgrades, UI) |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_cell_collision | cells/handle_cell_hit, (future: upgrades, audio) |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_lost | bolt/spawn_bolt_lost_text, (future: breaker penalty) |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | cells/handle_cell_hit | (future: run, upgrades, audio) |
| `NodeCleared` | `run/messages.rs` | `RunPlugin` | (not yet) | (future: state machine, UI) |
| `TimerExpired` | `run/messages.rs` | `RunPlugin` | (not yet) | (future: state machine) |
| `BumpPerformed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/update_bump, breaker/grade_bump | bolt/apply_bump_velocity, breaker/perfect_bump_dash_cancel, breaker/spawn_bump_grade_text |
| `BumpWhiffed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/grade_bump | breaker/spawn_whiff_text |
| `UpgradeSelected` | `ui/messages.rs` | `UiPlugin` | (not yet) | (future: upgrades) |
