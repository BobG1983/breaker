# Conditions and During

Conditions are state predicates evaluated each frame. Unlike triggers (which fire on edge events), a condition has a true/false value at any point in time and the `During` node fires when the condition becomes true and reverses when it becomes false. Conditions can cycle.

## Condition enum

```rust
pub enum Condition {
    NodeActive,
    ShieldActive,
    ComboActive(u32),
}
```

Each variant has a corresponding predicate function in `effect_v3/conditions/`:

- `NodeActive` → `is_node_active(world)` in `node_active.rs` — true while the node state machine is in `Playing`.
- `ShieldActive` → `is_shield_active(world)` in `shield_active.rs` — true while at least one `ShieldWall` entity exists. Implemented by scanning archetypes for the `ShieldWall` component without needing `&mut World`.
- `ComboActive(threshold)` → `is_combo_active(world, threshold)` in `combo_active.rs` — true while the consecutive perfect-bump streak is at or above `threshold`.

A fourth helper, `is_armed_source(source)` in `armed_source.rs`, is not a condition but a string predicate used by `evaluate_on` and `evaluate_conditions` to detect armed-key naming patterns (`#armed[`).

## Condition evaluation: evaluate_conditions

The condition poller is a single exclusive system that runs in `EffectV3Systems::Conditions`:

```rust
pub fn evaluate_conditions(world: &mut World) {
    // Phase 1: collect During entries from BoundEffects
    let mut during_entries: Vec<(Entity, String, Condition, ScopedTree)> = Vec::new();
    let mut query = world.query::<(Entity, &BoundEffects)>();
    for (entity, bound) in query.iter(world) {
        for (source, tree) in &bound.0 {
            if let Tree::During(condition, inner) = tree {
                during_entries.push((entity, source.clone(), condition.clone(), (**inner).clone()));
            }
        }
    }

    // Phase 2: evaluate transitions and fire/reverse
    for (entity, source, condition, inner) in during_entries {
        if world.get_entity(entity).is_err() { continue; }
        let is_true = evaluate_condition(&condition, world);
        // ... insert/remove DuringActive entry, call fire/reverse on transitions
    }
}
```

Two phases are required because the iteration phase needs an immutable borrow of the world, and the fire/reverse phase needs a mutable borrow. Cloning the entries between phases is cheap — entry counts per entity are small.

`evaluate_condition` is the dispatch helper:

```rust
pub fn evaluate_condition(condition: &Condition, world: &World) -> bool {
    match condition {
        Condition::NodeActive       => is_node_active(world),
        Condition::ShieldActive     => is_shield_active(world),
        Condition::ComboActive(n)   => is_combo_active(world, *n),
    }
}
```

It is `pub` so other systems (notably `UntilEvaluateCommand` Shape 4) can poll a single condition outside the polling system.

## DuringActive: per-entity state

```rust
#[derive(Component, Default, Debug)]
pub struct DuringActive(pub HashSet<String>);
```

Tracks which During sources currently have their effects applied on this entity. Inserted lazily by `evaluate_conditions` on first encounter.

The state machine is the standard one: rising edge → fire; falling edge → reverse.

```
not_active && condition_true   --> fire_scoped_tree    (insert source into DuringActive)
active && condition_false      --> reverse_scoped_tree (remove source from DuringActive)
not_active && condition_false  --> no-op
active && condition_true       --> no-op
```

## fire_scoped_tree and reverse_scoped_tree

The poller's two helpers walk the `ScopedTree` and apply effects. Mirrored on the fire and reverse sides:

```rust
fn fire_scoped_tree(inner: &ScopedTree, entity: Entity, source: &str, world: &mut World) {
    match inner {
        ScopedTree::Fire(reversible)  => fire_reversible_dispatch(reversible, entity, source, world),
        ScopedTree::Sequence(effects) => effects.iter().for_each(|r| fire_reversible_dispatch(r, entity, source, world)),
        ScopedTree::When(trigger, inner_tree) => {
            // Shape A: install an armed When entry into BoundEffects under #armed[0]
            install_armed_entry(entity, format!("{source}#armed[0]"), Tree::When(trigger.clone(), inner_tree.clone()), world);
        }
        ScopedTree::On(participant, scoped_terminal) => {
            // Shape D: install an armed On entry under #armed[0]
            install_armed_entry(entity, format!("{source}#armed[0]"), Tree::On(*participant, Terminal::from(scoped_terminal.clone())), world);
        }
        ScopedTree::During(..) => { /* nested During: handled by Shape A install pattern */ }
    }
}
```

Reversal mirrors:

```rust
fn reverse_scoped_tree(inner: &ScopedTree, entity: Entity, source: &str, world: &mut World) {
    match inner {
        ScopedTree::Fire(reversible)  => reverse_dispatch(reversible, entity, source, world),
        ScopedTree::Sequence(effects) => effects.iter().for_each(|r| reverse_dispatch(r, entity, source, world)),
        ScopedTree::When(_, inner_tree) => {
            // Remove armed entry, then reverse any effects the armed When may have fired
            bound.retain(|(name, _)| name != &armed_key);
            reverse_armed_tree(inner_tree, entity, &armed_key, world);
        }
        ScopedTree::On(_, scoped_terminal) => {
            // Remove armed entry, drain ArmedFiredParticipants, reverse on each tracked participant
            bound.retain(|(name, _)| name != &armed_key);
            let tracked = world.get_mut::<ArmedFiredParticipants>(entity).map(|mut c| c.drain(&armed_key)).unwrap_or_default();
            if let ScopedTerminal::Fire(reversible) = scoped_terminal {
                for participant in tracked {
                    commands.reverse_effect(participant, reversible.clone(), armed_key.clone());
                }
            }
        }
        ScopedTree::During(..) => { /* nested During: managed by its own #installed entry */ }
    }
}
```

## The four shapes

The During state machine handles four authoring patterns. Internally they all go through `fire_scoped_tree` / `reverse_scoped_tree`, but understanding them as named shapes makes authoring chips easier.

### Shape A — During wrapping a When (deferred listener)

```ron
During(NodeActive,
    When(PerfectBumped,
        Fire(Shockwave(base_range: 24.0, ...))))
```

When the condition becomes true, install an armed `When(PerfectBumped, Fire(Shockwave(...)))` into `BoundEffects` under `format!("{source}#armed[0]")`. The trigger system walks this entry every tick; `PerfectBumped` matches normally, the inner Fire is queued. When the condition becomes false, the `#armed[0]` entry is removed from `BoundEffects` and `reverse_armed_tree` undoes any reversible effects the armed When fired (using `reverse_all_by_source_dispatch` keyed on the `#armed[0]` source).

This shape is what lets you say "while in this state, this trigger does X." It's also why `ScopedTree::When(Trigger, Box<Tree>)` re-opens to the full unrestricted `Tree` — the inner When is walked by the normal trigger pipeline, where any effect type is valid.

### Shape B — Until wrapping a During

```ron
Until(BoltLostOccurred,
    During(NodeActive,
        Fire(SpeedBoost(multiplier: 1.5))))
```

The Until's `UntilEvaluateCommand` takes special care of this shape — see `until.md`. Essentially: the inner During is installed under `#installed[0]` so the condition poller activates it normally, and on Until-gate match, the install is torn down via `teardown_installed_during`. The condition poller never sees the Until itself; the Until only manages the install lifecycle.

### Shape C — During with direct Fire / Sequence

```ron
During(ShieldActive,
    Sequence([
        Fire(SpeedBoost(multiplier: 1.5)),
        Fire(DamageBoost(multiplier: 2.0)),
    ]))
```

The simplest case. On rising edge, fire each effect via `fire_reversible_dispatch`. On falling edge, reverse each via `reverse_dispatch`. The condition poller manages the lifecycle; no armed entries are involved.

### Shape D — During wrapping an On

```ron
During(ComboActive(3),
    On(Bump(Bolt),
        Fire(DamageBoost(multiplier: 1.5))))
```

Install an armed `On(Bump(Bolt), Fire(DamageBoost(1.5)))` into `BoundEffects` under `#armed[0]`. The walker's `evaluate_on` recognizes the armed-source naming and queues a `TrackArmedFireCommand` for each participant the On fires on, recording it in the owner's `ArmedFiredParticipants`. On condition end, the poller drains `ArmedFiredParticipants` for the armed key and queues a `reverse_effect` for each tracked participant — so the disarm reverses effects on the *exact* participants they were fired on, not on the owner.

This is why `ArmedFiredParticipants` exists. Without it, the disarm path would have no way to know which entities had received an effect through an armed redirect.

## Why poll instead of edge-triggering

Triggers are edge-driven (a bridge system reads a message and dispatches). Conditions are level-driven (the poller checks each frame). Two reasons not to make conditions edge-driven too:

1. **Multiple conditions can change in the same tick from different sources.** A bump could end a combo (`ComboActive`) at the same tick a node ends (`NodeActive`). A polling system handles all transitions uniformly; an edge-driven system would need a separate signal per condition source.
2. **Some conditions don't have a clean source signal.** `is_shield_active` is "any `ShieldWall` entity exists" — there is no single message that fires on shield expiry, just despawns scattered across multiple systems. Polling handles this without coupling.

Per-tick polling cost is low: the entry list is built from a single `Query<(Entity, &BoundEffects)>` iteration, and filtering for `Tree::During` is a cheap pattern match.

## Adding a condition

See `adding_conditions.md` for the step-by-step. In short: add a variant to the `Condition` enum, write a `is_my_condition(world: &World) -> bool` predicate function in `conditions/`, add it to the `evaluate_condition` match, and re-export from `conditions/mod.rs`.

The condition machinery automatically picks up new variants — no per-condition wiring beyond the predicate function.
