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
| `DamageDealt<Cell> { dealer, target, amount, source_chip }` | bolt (bolt_cell_collision), effect/effects (shockwave, explode, pulse, chain_lightning, piercing_beam, tether_beam) | shared/death_pipeline (apply_damage::<Cell>) |
| `DamageDealt<Bolt> { dealer, target, amount, source_chip }` | effect/effects (as applicable) | shared/death_pipeline (apply_damage::<Bolt>) |
| `DamageDealt<Wall> { dealer, target, amount, source_chip }` | effect/effects (as applicable) | shared/death_pipeline (apply_damage::<Wall>) |
| `DamageDealt<Breaker> { dealer, target, amount, source_chip }` | effect/effects (as applicable) | shared/death_pipeline (apply_damage::<Breaker>) |
| `KillYourself<T> { entity }` | shared/death_pipeline (detect_deaths::<T>), bolt (bolt_lost for ExtraBolts, tick_bolt_lifespan on timer expiry) | shared/death_pipeline (handle_kill::<T>), run (handle_breaker_death for T=Breaker) |
| `Destroyed<Cell> { position, was_required_to_clear }` | shared/death_pipeline (handle_kill::<Cell>) | run/node (track_node_completion), effect (on_cell_destroyed) |
| `Destroyed<Bolt> { position }` | shared/death_pipeline (handle_kill::<Bolt>) | effect (on_bolt_destroyed) |
| `Destroyed<Wall> { position }` | shared/death_pipeline (handle_kill::<Wall>) | effect (on_wall_destroyed) |
| `Destroyed<Breaker> { position }` | shared/death_pipeline (handle_kill::<Breaker>) | effect (on_breaker_destroyed) |
| `DespawnEntity { entity }` | shared/death_pipeline (handle_kill::<T> for Cell/Bolt/Wall/Breaker) | shared/death_pipeline (process_despawn_requests in FixedPostUpdate) |
| `BumpPerformed { grade, bolt }` | breaker | breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), effect (bridge_bump) |
| `BumpWhiffed` | breaker | breaker (spawn_whiff_text), effect (bridge_bump_whiff) |
| `BreakerSpawned` | breaker (spawn_or_reuse_breaker) | run/node (check_spawn_complete) |
| `CellsSpawned` | run/node (spawn_cells_from_layout) | run/node (check_spawn_complete) |
| `BoltSpawned` | bolt (spawn_bolt) | run/node (check_spawn_complete) |
| `WallsSpawned` | walls (state/run/node/systems/spawn_walls) | state/run/node (check_spawn_complete) |
| `SpawnNodeComplete` | run/node (check_spawn_complete) | scenario runner (baseline entity count sampling) |
| `NodeCleared` | run/node (track_node_completion) | run (handle_node_cleared) |
| `TimerExpired` | run/node (tick_node_timer) | run (handle_timer_expired) |
| `RunLost` | run (handle_breaker_death — reads KillYourself<Breaker>) | run (handle_run_lost) |
| `ApplyTimePenalty { seconds }` | effect/effects/time_penalty (fire) | run/node (apply_time_penalty) |
| `ReverseTimePenalty { seconds }` | effect/effects/time_penalty (reverse) | run/node (reverse_time_penalty) |
| `ChipSelected { name }` | state/run/chip_select (handle_chip_input) | chips (dispatch_chip_effects) |
| `HighlightTriggered { kind }` | run (detect_mass_destruction, detect_close_save, detect_combo_king, detect_pinball_wizard, detect_nail_biter, detect_first_evolution, detect_most_powerful_evolution, track_node_cleared_stats) | run (spawn_highlight_text) |

## Effect Dispatch (commands extension — not Message or observer)

Effect firing does not use `#[derive(Message)]` or `commands.trigger()`. Instead, each per-effect config struct implements `Fireable::fire(entity, source, world)` (and optionally `Reversible::reverse(entity, source, world)`); the `EffectCommandsExt` extension trait queues commands that call free functions in `effect_v3/dispatch/` to dispatch the enum to the right config:

| Method | Queued by | Applies via |
|--------|-----------|-------------|
| `commands.fire_effect(entity, effect, source)` | trigger bridge systems / walker `evaluate_fire` / chip dispatch / sequence terminals | `FireEffectCommand::apply` → `fire_dispatch(&effect, entity, &source, world)` → `config.fire(entity, source, world)` |
| `commands.reverse_effect(entity, effect, source)` | `evaluate_conditions` Shape D disarm | `ReverseEffectCommand::apply` → `reverse_dispatch(&effect, entity, &source, world)` → `config.reverse(entity, source, world)` |
| `commands.route_effect(entity, name, tree, route_type)` | `evaluate_terminal` for `Terminal::Route` | `RouteEffectCommand::apply` → installs into `BoundEffects` (for `RouteType::Bound`) or `StagedEffects` (for `RouteType::Staged`) |
| `commands.stamp_effect(entity, name, tree)` | chip dispatch (non-Fire roots), `evaluate_when`/`evaluate_once` arming, `SpawnStampRegistry` watchers, `evaluate_conditions` Shape A install | sugar for `route_effect(_, _, _, RouteType::Bound)` |
| `commands.stage_effect(entity, name, tree)` | `evaluate_when`/`evaluate_once` arming nested gates | sugar for `route_effect(_, _, _, RouteType::Staged)` |
| `commands.remove_effect(entity, name)` | chip unequip, `evaluate_once` self-removal | `RemoveEffectCommand::apply` → name-sweep across both `BoundEffects` and `StagedEffects` |
| `commands.remove_staged_effect(entity, name, tree)` | `walk_staged_effects` consume | `RemoveStagedEffectCommand::apply` → first matching `(name, tree)` tuple removed from `StagedEffects` only |
| `commands.track_armed_fire(owner, armed_source, participant)` | `evaluate_on` when source is an armed key | `TrackArmedFireCommand::apply` → appends participant to `ArmedFiredParticipants` |

Each effect module in `effect_v3/effects/<name>/` provides a config struct + `impl Fireable` (+ `impl Reversible` if reversible) + optional runtime systems registered via `Fireable::register(app)`. The enum-to-trait jump happens exactly once, in `fire_dispatch` and `reverse_dispatch`.

## Registered Messages (no active producer/consumer)

*None — all previously registered-but-unused messages have been removed.*
