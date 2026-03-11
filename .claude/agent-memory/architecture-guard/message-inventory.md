# Message Inventory

| Message | Defined In | Registered By | Written By (actual) | Should Write | Consumed By (per arch doc) |
|---------|-----------|---------------|--------------------|--------------|----|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_breaker_collision | physics | audio, upgrades, UI |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_cell_collision | physics | upgrades, cells, audio |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | physics/bolt_lost | physics | breaker |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | **physics/bolt_cell_collision (VIOLATION)** | cells | run, upgrades, audio |
| `NodeCleared` | `run/messages.rs` | `RunPlugin` | (not yet) | run | state machine, UI |
| `TimerExpired` | `run/messages.rs` | `RunPlugin` | (not yet) | run | state machine |
| `BumpPerformed` | `breaker/messages.rs` | `BreakerPlugin` | breaker/bump.rs | breaker | audio, upgrades |
| `UpgradeSelected` | `ui/messages.rs` | `UiPlugin` | (not yet) | UI | upgrades |
