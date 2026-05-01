# Messages — Inter-Domain Communication

Domains communicate through Bevy 0.18 messages (`#[derive(Message)]`, `MessageReader<T>`, `MessageWriter<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

## Choosing Between Component, Resource, and Message

| Use a... | When the data is... | Lifetime | Mutation pattern |
|----------|---------------------|----------|------------------|
| **Component** | Per-entity state — what an entity *is* or *carries* (HP, velocity, active boosts) | Lives with the entity | Owning domain mutates; other domains read |
| **Resource** | Singleton state — global config, per-run state, registries | Inserted once, persists across ticks (or scope) | Owning domain mutates; other domains read |
| **Message** | A discrete event — something happened, multiple consumers may care | One frame (default), or until drained | Producer writes once; any consumer reads |

If you're tempted to mutate a remote domain's component or resource directly, the answer is almost always *write a message instead*. The exceptions are catalogued in `plugins.md` under "Cross-Domain Write Exception Rubric".

A handful of messages also act as commands ("apply this time penalty"); those are owned by the *receiving* domain (the one that knows what the verb means), not the sender. Most messages name an event in the past tense (`BoltLost`, `Destroyed<T>`, `BumpPerformed`) and are owned by the domain where the event originated.

To find the producer or consumer of a specific message, `rg "MessageWriter<X>"` and `rg "MessageReader<X>"` (or `Messages<X>` / `commands.write`) are authoritative. This doc deliberately does not enumerate them — the table goes stale within weeks of any refactor.

## Damage Pipeline Messages

The `rantzsoft_dmg` crate owns the damage/heal/kill message types: `DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity`. Game systems emit `DamageDealt<T>` (and `HealDealt<T>`); the crate's pipeline applies them, detects deaths, and despawns. See `ordering.md` for the chain order and the `DmgSystems::EmitDamage` requirement for game-side writers.

## Force Application Pattern

When multiple independent systems exert forces on a bolt in the same tick, the canonical approach is a force-accumulation pipeline:

1. **Emitters** send `ApplyBoltForce { bolt: Entity, force: Vec2 }` messages. `force` is an acceleration in world-units/s²; emitters must NOT pre-multiply by `delta_secs` — the consumer owns the `* dt` step.
2. **Consumer** (`apply_bolt_forces` in `BoltSystems::ApplyForces`) drains all `ApplyBoltForce` messages each tick, sums forces per bolt entity, then applies `sum * delta_secs` to each bolt's `Velocity2D` before `SpatialSystems::ApplyVelocity` integrates position.

This pattern keeps `Velocity2D` write-ownership in the bolt domain, lets any number of producers compose forces without coordination, and ensures a single `* dt` multiplication per bolt per tick regardless of how many emitters contributed.

## Effect Dispatch — Not a Message

Effect firing does not use `#[derive(Message)]` and does not use `commands.trigger()`. Each per-effect config struct implements `Fireable::fire(entity, source, world)` (and optionally `Reversible::reverse(...)`); `EffectCommandsExt` queues commands that call free dispatch functions in `effect_v3/dispatch/`.

| Method | Queued by | Applies via |
|--------|-----------|-------------|
| `commands.fire_effect(entity, effect, source)` | Trigger bridges, walker `evaluate_fire`, chip dispatch, sequence terminals | `FireEffectCommand::apply` → `fire_dispatch(...)` → `config.fire(...)` |
| `commands.reverse_effect(entity, effect, source)` | `evaluate_conditions` Shape D disarm | `ReverseEffectCommand::apply` → `reverse_dispatch(...)` → `config.reverse(...)` |
| `commands.route_effect(entity, name, tree, route_type)` | `evaluate_terminal` for `Terminal::Route` | Installs into `BoundEffects` (`RouteType::Bound`) or `StagedEffects` (`RouteType::Staged`) |
| `commands.stamp_effect(entity, name, tree)` | Chip dispatch (non-Fire roots), `evaluate_when`/`evaluate_once` arming, `SpawnStampRegistry` watchers, `evaluate_conditions` Shape A install | Sugar for `route_effect(_, _, _, RouteType::Bound)` |
| `commands.stage_effect(entity, name, tree)` | `evaluate_when`/`evaluate_once` arming nested gates | Sugar for `route_effect(_, _, _, RouteType::Staged)` |
| `commands.remove_effect(entity, name)` | Chip unequip, `evaluate_once` self-removal | Name-sweep across both vecs |
| `commands.remove_staged_effect(entity, name, tree)` | `walk_staged_effects` consume | Removes the first matching `(name, tree)` from `StagedEffects` only |
| `commands.track_armed_fire(owner, armed_source, participant)` | `evaluate_on` when source is an armed key | Appends participant to `ArmedFiredParticipants` |

Each effect module in `effect_v3/effects/<name>/` provides a config + `impl Fireable` (+ `impl Reversible` if reversible) + optional runtime systems registered via `Fireable::register(app)`. The enum-to-trait jump happens exactly once, in `fire_dispatch` and `reverse_dispatch`.

For the full effect-system architecture, see `architecture/effects/`.
