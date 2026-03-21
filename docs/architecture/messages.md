# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Ownership Convention

Messages are defined in the domain that **conceptually owns the event**. Usually the sender, but "command" messages (telling a domain what to do) are defined by the receiving domain. Any domain may import and write another domain's message type — this is normal cross-domain communication, not a violation. See [plugins.md](plugins.md) "Cross-Domain Read Access" for the full rule.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker { bolt }` | physics | breaker (grade_bump), behaviors (bridge_breaker_impact) |
| `BoltHitCell { cell, bolt }` | physics | behaviors (bridge_cell_impact) |
| `BoltHitWall { bolt }` | physics | behaviors (bridge_wall_impact) |
| `BoltLost` | physics | bolt (spawn_bolt_lost_text), behaviors (bridge_bolt_lost) |
| `DamageCell { cell, damage, source_bolt }` | physics (bolt_cell_collision), behaviors/effects (shockwave) | cells (handle_cell_hit) |
| `BumpPerformed { grade, multiplier, bolt }` | breaker | bolt (apply_bump_velocity), breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), behaviors (bridge_bump) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text), behaviors (bridge_bump_whiff) |
| `BreakerSpawned` | breaker (spawn_breaker) | run/node (check_spawn_complete) |
| `CellDestroyed { was_required_to_clear }` | cells | run (track_node_completion), behaviors (bridge_cell_destroyed) |
| `CellsSpawned` | run/node (spawn_cells_from_layout) | run/node (check_spawn_complete) |
| `BoltSpawned` | bolt (spawn_bolt) | run/node (check_spawn_complete) |
| `WallsSpawned` | wall (spawn_walls) | run/node (check_spawn_complete) |
| `SpawnNodeComplete` | run/node (check_spawn_complete) | scenario runner (baseline entity count sampling) |
| `NodeCleared` | run/node (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run/node (tick_node_timer) | run (handle_timer_expired) |
| `RunLost` | behaviors/effects/life_lost (handle_life_lost) | run (handle_run_lost) |
| `ApplyTimePenalty { seconds }` | behaviors/effects/time_penalty (handle_time_penalty) | run/node (apply_time_penalty) |
| `SpawnAdditionalBolt` | behaviors/effects/spawn_bolt (handle_spawn_bolt) | bolt (spawn_additional_bolt) |
| `ChipSelected { name }` | UI (handle_chip_input) | chips (apply_chip_effect) |

## Observer Events (trigger via commands.trigger())

These are Bevy observer events (`#[derive(Event)]` + `commands.trigger()`), not `Message` types. They flow within a single domain and are consumed synchronously by registered observers.

| Event | Sent By | Observed By |
|-------|---------|-------------|
| `EffectFired { effect: TriggerChain, bolt: Option<Entity> }` | behaviors/bridges/* (all bridge systems) | behaviors/effects/* (handle_life_lost, handle_time_penalty, handle_spawn_bolt, handle_shockwave) |
| `ChipEffectApplied { effect, max_stacks }` | chips (apply_chip_effect) | chips/effects/* (handle_piercing, handle_damage_boost, handle_overclock, etc.) |

## Registered Messages (no consumers yet)

None — all registered messages now have active consumers.
