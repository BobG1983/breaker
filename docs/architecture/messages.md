# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker { bolt }` | physics | breaker (grade_bump) |
| `BoltHitCell { cell, bolt }` | physics | cells (handle_cell_hit) |
| `BoltLost` | physics | bolt (spawn_bolt_lost_text), behaviors (bridge_bolt_lost) |
| `BumpPerformed { grade, multiplier }` | breaker | bolt (apply_bump_velocity), breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), behaviors (bridge_bump) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text), behaviors (bridge_bump_whiff) |
| `BreakerSpawned` | breaker (spawn_breaker) | run/node (check_spawn_complete) |
| `CellDestroyed { was_required_to_clear }` | cells | run (track_node_completion) |
| `CellsSpawned` | run/node (spawn_cells_from_layout) | run/node (check_spawn_complete) |
| `BoltSpawned` | bolt (spawn_bolt) | run/node (check_spawn_complete) |
| `WallsSpawned` | wall (spawn_walls) | run/node (check_spawn_complete) |
| `SpawnNodeComplete` | run/node (check_spawn_complete) | scenario runner (baseline entity count sampling) |
| `NodeCleared` | run/node (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run/node (tick_node_timer) | run (handle_timer_expired) |
| `RunLost` | behaviors/consequences/life_lost (handle_life_lost) | run (handle_run_lost) |
| `ApplyTimePenalty { seconds }` | behaviors/consequences/time_penalty (handle_time_penalty) | run/node (apply_time_penalty) |
| `SpawnAdditionalBolt` | behaviors/consequences/spawn_bolt (handle_spawn_bolt) | bolt (spawn_additional_bolt) |
| `ChipSelected { name, kind }` | UI (handle_chip_input) | chips (apply_chip_effect) |

## Observer Events (trigger via commands.trigger())

These are Bevy observer events (`#[derive(Event)]` + `commands.trigger()`), not `Message` types. They flow within a single domain and are consumed synchronously by registered observers.

| Event | Sent By | Observed By |
|-------|---------|-------------|
| `ConsequenceFired(Consequence)` | behaviors (bridge_bolt_lost, bridge_bump, bridge_bump_whiff) | behaviors/consequences/* (handle_life_lost, handle_time_penalty, handle_spawn_bolt) |
| `ChipEffectApplied { effect, max_stacks }` | chips (apply_chip_effect) | chips/effects/* (handle_piercing, handle_damage_boost, etc.) |

## Registered Messages (no consumers yet)

None — all registered messages now have active consumers.
