# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Active Messages (Phase 1 — consumed in code)

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker` | physics | breaker (grade_bump) |
| `BoltHitCell` | physics | cells (handle_cell_hit) |
| `BoltLost` | physics | bolt (spawn_bolt_lost_text) |
| `BumpPerformed { grade }` | breaker | bolt (apply_bump_velocity), breaker (bump_feedback, perfect_bump_dash_cancel) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text) |

## Registered Messages (Phase 2+ — no consumers yet)

These messages are pre-registered (`add_message()`) for future phases. They have defined types but no `MessageReader` consumers in the codebase yet.

| Message | Registered By | Planned Consumers |
|---------|---------------|-------------------|
| `CellDestroyed` | cells | run (progress tracking), upgrades (overclock triggers), audio |
| `NodeCleared` | run | state machine, UI |
| `UpgradeSelected` | UI | upgrades (apply effects) |
| `TimerExpired` | run | state machine |
