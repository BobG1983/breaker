# Message Inventory

| Message | Defined In | Registered By | Consumed By (per arch doc) |
|---------|-----------|---------------|---------------------------|
| `BoltHitBreaker` | `physics/messages.rs` | `PhysicsPlugin` | audio, upgrades, UI |
| `BoltHitCell` | `physics/messages.rs` | `PhysicsPlugin` | upgrades, cells, audio |
| `BoltLost` | `physics/messages.rs` | `PhysicsPlugin` | breaker |
| `CellDestroyed` | `cells/messages.rs` | `CellsPlugin` | run, upgrades, audio |
| `NodeCleared` | `run/messages.rs` | `RunPlugin` | state machine, UI |
| `TimerExpired` | `run/messages.rs` | `RunPlugin` | state machine |
| `BumpPerformed` | `breaker/messages.rs` | `BreakerPlugin` | audio, upgrades |
| `UpgradeSelected` | `ui/messages.rs` | `UiPlugin` | upgrades |
