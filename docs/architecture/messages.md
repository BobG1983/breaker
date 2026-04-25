# Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Ownership Convention

Messages are defined in the domain that **conceptually owns the event**. Usually the sender, but "command" messages (telling a domain what to do) are defined by the receiving domain. Any domain may import and write another domain's message type — this is normal cross-domain communication, not a violation. See [plugins.md](plugins.md) "Cross-Domain Read Access" for the full rule.

## Active Messages

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltImpactBreaker { bolt, breaker }` | bolt (bolt_breaker_collision) | breaker (grade_bump), effect (bridge_breaker_impact) |
| `BoltImpactCell { cell, bolt, impact_normal, piercing_remaining }` | bolt (bolt_cell_collision) | effect (bridge_cell_impact), cells (check_armor_direction — reads impact_normal + piercing_remaining), run/highlights (detect_pinball_wizard — count only) |
| `BoltImpactWall { bolt, wall }` | bolt (bolt_wall_collision) | effect (bridge_wall_impact) |
| `BreakerImpactCell { breaker, cell }` | breaker (breaker_cell_collision) | effect (bridge_cell_impact, bridge_breaker_impacted) |
| `BreakerImpactWall { breaker, wall }` | breaker (breaker_wall_collision) | effect (bridge_wall_impact, bridge_breaker_impacted) |
| `CellImpactWall { cell, wall }` | cells (cell_wall_collision) | effect (bridge_wall_impact, bridge_cell_impacted) |
| `SalvoImpactBreaker { salvo, breaker }` | cells (salvo_breaker_collision) | effect bridges (Wave 6C — consumed by impact trigger systems) |
| `PortalEntered { portal }` | cells (check_portal_entry — reads BoltImpactCell, filters for PortalCell) | cells (handle_portal_entered — mock: immediately emits PortalCompleted) |
| `PortalCompleted { portal }` | cells (handle_portal_entered) | cells (handle_portal_completed — writes KillYourself<Cell> for the portal entity) |
| `BoltLost` | bolt (bolt_lost), protocol/protocols/reckless_dash (reckless_dash_double_penalty — duplicate write when Dashing) | bolt (spawn_bolt_lost_text), effect (bridge_bolt_lost), protocol/protocols/reckless_dash (reckless_dash_double_penalty — read to decide duplication) |
| `DamageDealt<Cell> { dealer, attributed_to, target, amount, source, _marker }` | bolt (bolt_cell_collision), cells (tick_survival_timer — self-destruct), cells (salvo_cell_collision), effect/effects (shockwave, pulse, chain_lightning, tether_beam; W7 explode and piercing_beam emit via the apply_*_damage consumer systems below, NOT inline from `Fireable::fire`), protocol/protocols/debt_collector (debt_collector_on_impact), protocol/protocols/iron_curtain (iron_curtain_on_bolt_lost), protocol/protocols/echo_strike (echo_strike_on_impact), protocol/protocols/reckless_dash (reckless_dash_amplify_damage), protocol/protocols/burnout (burnout_amplify_damage — mega-bump amplified hit, tagged `source: Some(SourceId::protocol(ProtocolKind::Burnout).build())` — see `docs/architecture/source_id.md`) | cells (check_armor_direction — mutating interceptor: drains, filters blocked hits, re-extends before apply_damage sees the queue), rantzsoft_dmg crate-internal (`apply_damage::<Cell>` in `DmgSystems::ApplyDamage`; invulnerability is enforced upstream by the crate's `invulnerable_filter::<Cell>` MessageMutator — emitters never pre-filter `Without<Invulnerable>`). Diffusion BFS redistribution runs alongside as separate systems (`diffusion_reduce_primary` in `MutateDamage`, `diffusion_emit_rings` in `PostApplyDamage`) |
| `ExplodeEmissionRequested { center, radius, base_damage, dealer, source }` (W7) | effect/effects/explode (`ExplodeConfig::fire` writes the request synchronously) | effect/effects/explode (`apply_explode_damage` in `DmgSystems::EmitDamage`; reads request, queries `CollisionQuadtree`, emits `DamageDealt<Cell>` per hit cell with raw `base_damage`) — crate-internal `pub(in crate::effect_v3)` |
| `PiercingBeamEmissionRequested { origin, direction, half_width, base_damage, dealer, source }` (W7) | effect/effects/piercing_beam (`PiercingBeamConfig::fire` writes the request synchronously, baking `bolt_base_damage * damage_mult` into `base_damage`) | effect/effects/piercing_beam (`apply_piercing_beam_damage` in `DmgSystems::EmitDamage`; reads request, iterates live cells, projects each onto beam axis, emits `DamageDealt<Cell>` per hit cell with raw `base_damage`) — crate-internal `pub(in crate::effect_v3)` |
| `DamageDealt<Bolt> { dealer, attributed_to, target, amount, source, _marker }` | effect/effects (as applicable) | rantzsoft_dmg crate-internal (apply_damage::<Bolt>) |
| `DamageDealt<Wall> { dealer, attributed_to, target, amount, source, _marker }` | effect/effects (as applicable) | rantzsoft_dmg crate-internal (apply_damage::<Wall>) |
| `DamageDealt<Breaker> { dealer, attributed_to, target, amount, source, _marker }` | effect/effects (as applicable) | rantzsoft_dmg crate-internal (apply_damage::<Breaker>) |
| `DamageDealt<Salvo> { dealer, attributed_to, target, amount, source, _marker }` | (no current production sender — pipeline registered for completeness) | rantzsoft_dmg crate-internal (apply_damage::<Salvo>) |
| `HealDealt<Cell> { healer, attributed_to, target, amount, source, cap, _marker }` | hazard (`volatility_grow_cells`, `cascade_heal_on_death`, `renewal_regrow`, `momentum_heal_on_nonlethal`, `sympathy_heal_adjacent`) | rantzsoft_dmg crate-internal (apply_heal::<Cell>) |
| `HealDealt<Bolt> { healer, attributed_to, target, amount, source, cap, _marker }` | effect/effects (as applicable) | rantzsoft_dmg crate-internal (apply_heal::<Bolt>) |
| `HealDealt<Wall> { healer, attributed_to, target, amount, source, cap, _marker }` | effect/effects (as applicable) | rantzsoft_dmg crate-internal (apply_heal::<Wall>) |
| `HealDealt<Breaker> { healer, attributed_to, target, amount, source, cap, _marker }` | effect/effects (as applicable) | rantzsoft_dmg crate-internal (apply_heal::<Breaker>) |
| `HealDealt<Salvo> { healer, attributed_to, target, amount, source, cap, _marker }` | (no current production sender — pipeline registered for completeness) | rantzsoft_dmg crate-internal (apply_heal::<Salvo>) |
| `KillYourself<T> { victim, killer, _marker }` | rantzsoft_dmg crate-internal (detect_deaths::<T>), bolt (bolt_lost for ExtraBolts, tick_bolt_lifespan on timer expiry), cells (handle_portal_completed for T=Cell) | rantzsoft_dmg crate-internal (handle_kill::<T>), run (handle_breaker_death for T=Breaker) |
| `Destroyed<Cell> { victim, killer, victim_pos, killer_pos, _marker }` | rantzsoft_dmg crate-internal (handle_kill::<Cell>) | run/node (track_node_completion), effect (on_cell_destroyed), protocol/protocols/fission (fission_on_cell_destroyed — increments FissionCounter, spawns bolt clone on every Nth kill), protocol/protocols/echo_strike (echo_strike_cleanup_destroyed_echoes — removes dead cells from EchoNetwork deques) |
| `Destroyed<Bolt> { victim, killer, victim_pos, killer_pos, _marker }` | rantzsoft_dmg crate-internal (handle_kill::<Bolt>) | effect (on_bolt_destroyed) |
| `Destroyed<Wall> { victim, killer, victim_pos, killer_pos, _marker }` | rantzsoft_dmg crate-internal (handle_kill::<Wall>) | effect (on_wall_destroyed) |
| `Destroyed<Breaker> { victim, killer, victim_pos, killer_pos, _marker }` | rantzsoft_dmg crate-internal (handle_kill::<Breaker>) | effect (on_breaker_destroyed) |
| `Destroyed<Salvo> { victim, killer, victim_pos, killer_pos, _marker }` | rantzsoft_dmg crate-internal (handle_kill::<Salvo>) | (no current consumers — emitted for future effect bridge use) |
| `DespawnEntity { entity }` | rantzsoft_dmg crate-internal (handle_kill::<T> for Cell/Bolt/Wall/Breaker/Salvo) | rantzsoft_dmg crate-internal (process_despawn_requests in FixedPostUpdate) |
| `BumpPerformed { grade, bolt, breaker }` | breaker, protocol/protocols/afterimage::afterimage_check_phantom_bounce (synthetic phantom bump — msg.breaker carries PhantomBreaker, not Breaker) | breaker (spawn_bump_grade_text, perfect_bump_dash_cancel), effect (bridge_bump), protocol/protocols/reckless_dash (reckless_dash_on_bump), protocol/protocols/conductor (conductor_swap_on_perfect_bump — Perfect grade only, ExtraBolt target only), protocol/protocols/afterimage::afterimage_spawn_phantom_bolt (reads Perfect-grade phantom bumps to spawn phantom-bolt clones) |
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
| `ReverseTimePenalty { seconds }` | effect/effects/time_penalty (reverse), protocol/protocols/siphon (siphon_on_cell_destroyed) | run/node (reverse_time_penalty) |
| `ChipSelected { name }` | state/run/chip_select (handle_chip_input) | chips (dispatch_chip_effects) |
| `HighlightTriggered { kind }` | run (detect_mass_destruction, detect_close_save, detect_combo_king, detect_pinball_wizard, detect_nail_biter, detect_first_evolution, detect_most_powerful_evolution, track_node_cleared_stats) | run (spawn_highlight_text) |

### Contract notes

- `BumpPerformed.breaker: Entity` MAY carry `PhantomBreaker` instead of `Breaker` (afterimage protocol synthetic phantom bump). Consumers that query `With<Breaker>` on this entity will silently miss phantom bumps; consumers that deref by `Entity` get the phantom.

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
