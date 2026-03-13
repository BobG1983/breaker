# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker` | physics | breaker (grade_bump) |
| `BoltHitCell` | physics | cells (handle_cell_hit) |
| `BoltLost` | physics | bolt (spawn_bolt_lost_text) |
| `BumpPerformed { grade }` | breaker | bolt (apply_bump_velocity), breaker (bump_feedback, perfect_bump_dash_cancel) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text) |
| `CellDestroyed` | cells | run (track_node_completion) |
| `NodeCleared` | run (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run (tick_node_timer) | run (handle_timer_expired) |

## Registered Messages (no consumers yet)

| Message | Registered By | Planned Consumers |
|---------|---------------|-------------------|
| `UpgradeSelected` | UI | upgrades (apply effects) |
