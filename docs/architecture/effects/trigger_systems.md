# Trigger System Pattern

Each trigger category (`bump`, `impact`, `death`, `bolt_lost`, `node`, `time`) has its own subdirectory under `effect_v3/triggers/`. Every category follows the same shape: a `mod.rs`, a `register.rs` that wires bridge systems into `EffectV3Systems::Bridge`, and a `bridges/` subdirectory holding the actual bridge functions.

```
effect_v3/triggers/
  bump/
    mod.rs
    register.rs        — register(app) — adds bridges to EffectV3Systems::Bridge
    bridges/
      mod.rs
      system.rs        — on_bumped, on_perfect_bumped, on_bump_occurred, ...
      tests/           — per-bridge unit tests
```

Trigger bridges are normal Bevy systems — no exclusive world access, no parallelism blocking. They take a `MessageReader<T>` for their source message, a `Query<(Entity, &BoundEffects, &mut StagedEffects)>`, and `Commands`. They build a `TriggerContext`, then call `walk_staged_effects` followed by `walk_bound_effects` for each entity that should see the trigger.

## The bridge shape

```rust
pub fn on_my_event(
    mut events: MessageReader<MyEvent>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in events.read() {
        let context = TriggerContext::Bump {
            bolt:    Some(event.bolt),
            breaker: event.breaker,
        };
        let trigger = Trigger::Bumped;

        // Local: walk only participating entities
        for entity in [event.bolt, event.breaker] {
            if let Ok((entity, bound, mut staged)) = query.get_mut(entity) {
                walk_staged_effects(entity, &trigger, &context, &staged.0, &mut commands);
                walk_bound_effects(entity, &trigger, &context, &bound.0, &mut commands);
            }
        }
    }
}
```

Five things every bridge does:

1. **Iterate source messages** — `for event in events.read()`.
2. **Build a `TriggerContext`** with the participant entities from the message.
3. **Choose the `Trigger` variant** to dispatch — local, global, or both (some bridges fire multiple variants from the same message).
4. **Decide which entities to walk** — for local triggers, iterate the participating entities; for global triggers, iterate every entity in the query.
5. **Walk staged then bound** for each chosen entity. The order matters — see `evaluation.md`.

## Local vs global

**Local triggers** fire on specific participant entities. The bridge iterates `[event.entity_a, event.entity_b]` (or whatever the participants are) and only walks those entities' chains. Examples: `Bumped`, `Impacted(EntityKind)`, `Died`, `Killed(EntityKind)`.

**Global triggers** fire on every entity that has effects. The bridge iterates the entire query. Examples: `BumpOccurred`, `ImpactOccurred(EntityKind)`, `DeathOccurred(EntityKind)`, `BoltLostOccurred`, `NodeStartOccurred`, `NodeEndOccurred`, `BumpWhiffOccurred`, `NoBumpOccurred`.

Some game messages produce **both** local and global triggers — a single `BoltImpactCell` fires `Impacted(Cell)` on the bolt, `Impacted(Bolt)` on the cell, `ImpactOccurred(Cell)` globally, and `ImpactOccurred(Bolt)` globally. These are typically split across separate bridge functions in the same category for clarity.

## A real example: the bump category

`triggers/bump/register.rs`:

```rust
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_bumped,
            bridges::on_perfect_bumped,
            bridges::on_early_bumped,
            bridges::on_late_bumped,
            bridges::on_bump_occurred,
            bridges::on_perfect_bump_occurred,
            bridges::on_early_bump_occurred,
            bridges::on_late_bump_occurred,
            bridges::on_bump_whiff_occurred,
            bridges::on_no_bump_occurred,
        )
            .in_set(EffectV3Systems::Bridge),
    );
}
```

Ten bridge functions in one category — local and global variants for each bump grade, plus whiff and no-bump (which only have global variants because there's no participating bolt).

`Trigger::Bumped` and `Trigger::BumpOccurred` look the same in the chip data; the difference is which bridge dispatches them. The local one passes `TriggerContext::Bump { bolt: Some(...), breaker: ... }` and walks only `[bolt, breaker]`; the global one passes the same context and walks everything.

## Source message ordering

Bridges run **after** the game systems that produce their source messages. Each category's `register::register` adds the necessary `.after()` constraints. Examples:

- Impact bridges go `.after(BoltSystems::CellCollision)` and `.after(BoltSystems::BreakerCollision)`.
- Bolt-lost bridge goes `.after(BoltSystems::BoltLost)`.
- Death bridges go `.after(EntitySystems::Death)`.

Without these constraints, a bridge might run earlier in the same tick than the game system that produces its message, miss the message that frame, and process it on the next FixedUpdate — introducing a one-tick lag for the entire effect chain.

## Run conditions

Most bridges should only fire while the node is playing. Add `.run_if(in_state(NodeState::Playing))` to the registration:

```rust
.add_systems(
    FixedUpdate,
    bridges::on_my_event
        .run_if(in_state(NodeState::Playing))
        .after(SourceDomainSystems::Produce)
        .in_set(EffectV3Systems::Bridge),
)
```

Some bridges (notably the `node` category's `NodeStartOccurred` / `NodeEndOccurred` bridges) intentionally run outside `Playing` — they're reading transitions in and out of the playing state. Don't blanket-apply the run condition without thinking about which lifecycle stage the trigger belongs to.

## What bridges do NOT do

- **Bridges do not call `fire_dispatch` directly.** They queue commands via the walker; commands flush later.
- **Bridges do not poll conditions.** That's `evaluate_conditions` in `EffectV3Systems::Conditions`.
- **Bridges do not reverse effects.** Reversal is driven by the condition poller (during) and the Until command (until).
- **Bridges do not modify `BoundEffects` or `StagedEffects` directly.** They mutate `StagedEffects` only through `walk_staged_effects` (which queues `commands.remove_staged_effect`) and through commands.

## Querying both BoundEffects and StagedEffects

Bridges take `Query<(Entity, &BoundEffects, &mut StagedEffects)>`. The `&mut` on `StagedEffects` is historic — the walker no longer mutates the slice directly; it queues `commands.remove_staged_effect`. The `&mut` is kept so the query change-detects on the staged component when commands flush later. (If this constraint relaxes in the future, the query can drop to `&StagedEffects`.)

The query naturally filters to entities that have both components. Entities without effect storage are skipped — the dispatch system inserts the components lazily on first stamp, so any entity with effects is guaranteed to have both.

## The walker is the same for all bridges

Every bridge ultimately calls `walk_staged_effects` and `walk_bound_effects`. Both functions take the same arguments and behave the same regardless of which trigger category called them. This is the whole point of the trigger/walker split: the trigger system knows about the source message, the walker knows about the tree structure. Adding a new trigger category only requires writing the bridge — the walker doesn't change.
