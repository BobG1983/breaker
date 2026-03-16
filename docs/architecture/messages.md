# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker` | physics | breaker (grade_bump) |
| `BoltHitCell` | physics | cells (handle_cell_hit) |
| `BoltLost` | physics | bolt (spawn_bolt_lost_text), breaker/behaviors (bridge_bolt_lost) |
| `BumpPerformed { grade, multiplier }` | breaker | bolt (apply_bump_velocity), breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), breaker/behaviors (bridge_bump) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text) |
| `CellDestroyed` | cells | run (track_node_completion) |
| `NodeCleared` | run/node (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run/node (tick_node_timer) | run (handle_timer_expired) |
| `RunLost` | breaker/behaviors (handle_life_lost) | run (handle_run_lost) |
| `ApplyTimePenalty { seconds }` | breaker/behaviors (handle_time_penalty) | run/node (apply_time_penalty) |
| `SpawnAdditionalBolt` | breaker/behaviors (handle_spawn_bolt_requested) | bolt (spawn_additional_bolt) |

## Registered Messages (no consumers yet)

| Message | Registered By | Planned Consumers |
|---------|---------------|-------------------|
| `UpgradeSelected` | UI | chips (apply effects) |
