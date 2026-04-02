# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Ownership Convention

Messages are defined in the domain that **conceptually owns the event**. Usually the sender, but "command" messages (telling a domain what to do) are defined by the receiving domain. Any domain may import and write another domain's message type — this is normal cross-domain communication, not a violation. See [plugins.md](plugins.md) "Cross-Domain Read Access" for the full rule.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltImpactBreaker { bolt, breaker }` | bolt (bolt_breaker_collision) | breaker (grade_bump), effect (bridge_breaker_impact) |
| `BoltImpactCell { cell, bolt }` | bolt (bolt_cell_collision) | effect (bridge_cell_impact) |
| `BoltImpactWall { bolt, wall }` | bolt (bolt_wall_collision) | effect (bridge_wall_impact) |
| `BreakerImpactCell { breaker, cell }` | breaker (breaker_cell_collision) | effect (bridge_cell_impact, bridge_breaker_impacted) |
| `BreakerImpactWall { breaker, wall }` | breaker (breaker_wall_collision) | effect (bridge_wall_impact, bridge_breaker_impacted) |
| `CellImpactWall { cell, wall }` | cells (cell_wall_collision) | effect (bridge_wall_impact, bridge_cell_impacted) |
| `BoltLost` | bolt (bolt_lost) | bolt (spawn_bolt_lost_text), effect (bridge_bolt_lost) |
| `DamageCell { cell, damage, source_chip }` | bolt (bolt_cell_collision), effect/effects (shockwave, explode, pulse, chain_lightning, piercing_beam, tether_beam) | cells (handle_cell_hit) |
| `BumpPerformed { grade, bolt }` | breaker | breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), effect (bridge_bump) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text), effect (bridge_bump_whiff) |
| `BreakerSpawned` | breaker (spawn_or_reuse_breaker) | run/node (check_spawn_complete) |
| `CellDestroyedAt { position, was_required_to_clear }` | cells (cleanup_cell) | run (track_node_completion), effect (bridge_cell_death evaluates trigger on remaining alive cell via RequestCellDestroyed) |
| `CellsSpawned` | run/node (spawn_cells_from_layout) | run/node (check_spawn_complete) |
| `BoltSpawned` | bolt (spawn_bolt) | run/node (check_spawn_complete) |
| `WallsSpawned` | wall (spawn_walls) | run/node (check_spawn_complete) |
| `SpawnNodeComplete` | run/node (check_spawn_complete) | scenario runner (baseline entity count sampling) |
| `NodeCleared` | run/node (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run/node (tick_node_timer) | run (handle_timer_expired) |
| `RunLost` | effect/effects/life_lost (handle_life_lost) | run (handle_run_lost) |
| `ApplyTimePenalty { seconds }` | effect/effects/time_penalty (fire) | run/node (apply_time_penalty) |
| `ReverseTimePenalty { seconds }` | effect/effects/time_penalty (reverse) | run/node (reverse_time_penalty) |
| `RequestBoltDestroyed { bolt }` | bolt (bolt_lost) | effect (bridge_bolt_death), bolt (cleanup_destroyed_bolts) |
| `ChipSelected { name }` | UI (handle_chip_input) | chips (dispatch_chip_effects) |
| `HighlightTriggered { kind }` | run (detect_mass_destruction, detect_close_save, detect_combo_king, detect_pinball_wizard, detect_nail_biter, detect_first_evolution, detect_most_powerful_evolution, track_node_cleared_stats) | run (spawn_highlight_text) |

## Effect Dispatch (commands extension — not Message or observer)

Effect firing does not use `#[derive(Message)]` or `commands.trigger()`. Instead, `EffectKind` exposes `fire(entity, world)` and `reverse(entity, world)` free functions dispatched via `EffectCommandsExt`:

| Method | Queued by | Applies via |
|--------|-----------|-------------|
| `commands.fire_effect(entity, effect, source_chip)` | trigger bridge systems evaluating `Do(effect)` nodes | `FireEffectCommand::apply` → `effect.fire(entity, &source_chip, world)` |
| `commands.reverse_effect(entity, effect, source_chip)` | `Reverse` node unwinding | `ReverseEffectCommand::apply` → `effect.reverse(entity, &source_chip, world)` |
| `commands.transfer_effect(entity, name, children, permanent, context)` | `On` node redirect | `TransferCommand::apply` → pushes to `BoundEffects` or `StagedEffects`; `context` carries trigger entity references for targeted `On` resolution |
| `commands.push_bound_effects(entity, effects)` | `dispatch_cell_effects`, `dispatch_breaker_effects` dispatch systems | `PushBoundEffects::apply` → inserts `BoundEffects`/`StagedEffects` if absent, then appends entries |

Each effect module in `effect/effects/` provides `fire()`, `reverse()`, and `register()`. The enum match in `EffectKind` is mechanical dispatch only.

## Registered Messages (no active producer/consumer)

*None — all previously registered-but-unused messages have been removed.*
