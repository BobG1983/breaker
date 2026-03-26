# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Ownership Convention

Messages are defined in the domain that **conceptually owns the event**. Usually the sender, but "command" messages (telling a domain what to do) are defined by the receiving domain. Any domain may import and write another domain's message type — this is normal cross-domain communication, not a violation. See [plugins.md](plugins.md) "Cross-Domain Read Access" for the full rule.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker { bolt }` | bolt (bolt_breaker_collision) | breaker (grade_bump), effect (bridge_breaker_impact) |
| `BoltHitCell { cell, bolt }` | bolt (bolt_cell_collision) | effect (bridge_cell_impact) |
| `BoltHitWall { bolt }` | bolt (bolt_cell_collision) | effect (bridge_wall_impact) |
| `BoltLost` | bolt (bolt_lost) | bolt (spawn_bolt_lost_text), effect (bridge_bolt_lost) |
| `DamageCell { cell, damage, source_bolt }` | bolt (bolt_cell_collision), effect/effects (shockwave) | cells (handle_cell_hit) |
| `SpawnChainBolt { anchor, tether_distance }` | effect/effects (handle_chain_bolt) | bolt (spawn_chain_bolt) |
| `BumpPerformed { grade, bolt }` | breaker | breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), effect (bridge_bump) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text), effect (bridge_bump_whiff) |
| `BreakerSpawned` | breaker (spawn_breaker) | run/node (check_spawn_complete) |
| `CellDestroyed { was_required_to_clear }` | cells | run (track_node_completion), effect (bridge_cell_death evaluates trigger on remaining alive cell via RequestCellDestroyed) |
| `CellsSpawned` | run/node (spawn_cells_from_layout) | run/node (check_spawn_complete) |
| `BoltSpawned` | bolt (spawn_bolt) | run/node (check_spawn_complete) |
| `WallsSpawned` | wall (spawn_walls) | run/node (check_spawn_complete) |
| `SpawnNodeComplete` | run/node (check_spawn_complete) | scenario runner (baseline entity count sampling) |
| `NodeCleared` | run/node (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run/node (tick_node_timer) | run (handle_timer_expired) |
| `RunLost` | effect/effects/life_lost (handle_life_lost) | run (handle_run_lost) |
| `ApplyTimePenalty { seconds }` | effect/effects/time_penalty (handle_time_penalty) | run/node (apply_time_penalty) |
| `SpawnAdditionalBolt` | effect/effects/spawn_bolt (handle_spawn_bolt) | bolt (spawn_additional_bolt) |
| `ChipSelected { name }` | UI (handle_chip_input) | chips (dispatch_chip_effects) |
| `HighlightTriggered { kind }` | run (detect_mass_destruction, detect_close_save, detect_combo_king, detect_pinball_wizard, detect_nail_biter, detect_first_evolution, track_node_cleared_stats) | run (spawn_highlight_text) |

## Observer Events (trigger via commands.trigger())

These are Bevy observer events (`#[derive(Event)]` + `commands.trigger()`), not `Message` types. They flow within a single domain and are consumed synchronously by registered observers.

| Event | Sent By | Observed By |
|-------|---------|-------------|
| Per-effect typed events (e.g., `ShockwaveFired`, `SpeedBoostFired`, `LoseLifeFired`, `ChainBoltFired`, etc.) | effect/triggers/* (bridge systems via `fire_typed_event`) | effect/effects/* (per-effect observers, e.g., `handle_shockwave`, `handle_speed_boost`, `handle_life_lost`, `handle_chain_bolt`) |
| Per-effect passive events (e.g., `PiercingApplied`, `DamageBoostApplied`, `SpeedBoostApplied`, etc.) | chips (dispatch_chip_effects via `fire_passive_event`) | effect/effects/* (per-passive observers, e.g., `handle_piercing`, `handle_damage_boost`) |

## Registered Messages (no consumers yet)

None — all registered messages now have active consumers.
