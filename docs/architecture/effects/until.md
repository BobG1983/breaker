# Until State Machine

`Until(Trigger, ScopedTree)` is event-scoped: apply effects immediately, reverse them when the gate trigger fires. It is the most complex tree node, and it is implemented as a deferred state machine — there is no desugaring into other node types.

This file documents the actual implementation: `evaluate_until` queues `UntilEvaluateCommand`, the command runs the state machine inside `Command::apply`, and `UntilApplied` tracks per-entity which sources have already fired.

## The implementation

All Until logic lives in `effect_v3/walking/until/system.rs`.

```rust
#[derive(Component, Default, Debug)]
pub struct UntilApplied(pub HashSet<String>);

pub fn evaluate_until(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &ScopedTree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    let _ = context;
    commands.queue(UntilEvaluateCommand {
        entity,
        gate_trigger:   gate_trigger.clone(),
        active_trigger: active_trigger.clone(),
        inner:          inner.clone(),
        source:         source.to_owned(),
    });
}
```

`evaluate_until` does almost nothing — it just queues the command. All the work happens when the command flushes with `&mut World` access.

## UntilApplied: what it tracks

`UntilApplied(HashSet<String>)` is a per-entity component holding source names whose Until has already fired. It is the only piece of state Until needs:

- `applied.contains(source)` → the inner effects are currently active for this Until entry.
- `!applied.contains(source)` → the inner effects have not been applied (or have already been reversed).

The state transitions look like:

```
NotApplied  -- gate doesn't match --> NotApplied  (no-op)
NotApplied  -- gate doesn't match, first walk --> Applied (fire inner)
NotApplied  -- gate matches, first walk --> NotApplied (fire then immediately reverse)
Applied     -- gate doesn't match --> Applied (no-op)
Applied     -- gate matches --> NotApplied (reverse inner, drop entry)
```

The "first walk" is the first time the walker encounters this Until entry on this entity. After the first walk, the entry remains in `BoundEffects` until the gate trigger fires — every subsequent tick's walk re-queues an `UntilEvaluateCommand`, which sees that the source is already in `UntilApplied` and no-ops on the apply path, only checking for gate trigger match.

## The four shapes

`UntilEvaluateCommand::apply` branches on the inner `ScopedTree` variant:

### Shape 1 — `Until(gate, ScopedTree::Fire(reversible))`

The simplest case. Direct fire of a single reversible effect.

```ron
Until(TimeExpires(2.0), Fire(SpeedBoost(multiplier: 1.3)))
```

State machine:

1. **First walk, gate doesn't match yet**: `fire_reversible_dispatch(reversible, entity, source, world)` → bolt speeds up. Insert `source` into `UntilApplied`.
2. **Subsequent walks, gate doesn't match**: source is in `UntilApplied`, no-op.
3. **Gate matches** (e.g. `TimeExpires` fires after 2s): `reverse_dispatch(reversible, entity, source, world)` → speed boost removed. Remove `source` from `UntilApplied`. Remove the Until entry from `BoundEffects`.

The first walk has a special case: if the gate trigger and active trigger happen to be equal on the very first walk (rare — would require an Until immediately matched by the trigger that armed it), the command fires and immediately reverses in the same tick, leaving `UntilApplied` and `BoundEffects` clean.

### Shape 2 — `Until(gate, ScopedTree::Sequence(reversibles))`

Multiple reversible effects, fired in order. Reversal walks the sequence in the same order (not reversed) — each reversible's `reverse()` is independent.

State machine is identical to Shape 1 except that `fire_scoped_tree` and `reverse_scoped_tree` iterate the vec.

### Shape 3 — `Until(gate, ScopedTree::When(_, _) | ScopedTree::On(_, _) | ScopedTree::During(_, _))`

These cases are handled by `fire_scoped_tree` / `reverse_scoped_tree`, but the fire-side branches for `When`/`On`/`During` are deliberately empty:

```rust
ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {
    // Nested When/On/During inside Until: conditional/redirected behavior
    // that fires during future walks, not during initial application.
}
```

The reason: a `ScopedTree::When` inside an `Until` is a deferred listener that fires during *future* trigger evaluation, not on the Until's first walk. Adding it to `BoundEffects` would cause it to fire — but Until cannot do that during apply, because then it would have to track the inner When for reversal too. Instead, the inner is left structurally inert during the Until's apply phase.

If you find yourself authoring an `Until(gate, When(...))` and expecting the inner When to start listening immediately, that is not what happens — Shape 3 does nothing on the fire side. Use a top-level `When` followed by a Stage/Bound install if you need that.

### Shape 4 — `Until(gate, ScopedTree::During(condition, inner_scoped))`

The most involved case. Until wraps a During — apply the During (so the condition poller picks it up) and tear it down on gate match.

```ron
Until(BoltLostOccurred,
    During(NodeActive,
        Fire(SpeedBoost(multiplier: 1.5))))
```

This means: while the node is playing, the bolt has +50% speed; the entire arrangement ends the moment a bolt is lost. The `During` should activate normally as long as the Until is still arming the install; once `BoltLost` fires, both the During install AND any active scoped effects must be torn down.

State machine:

1. **First walk, gate doesn't match**: install the inner During into `BoundEffects` under `format!("{source}#installed[0]")`. Insert `source` into `UntilApplied`. The condition poller will pick up the installed During next tick and activate it normally.
2. **Subsequent walks, gate doesn't match**: source is in `UntilApplied`, no-op.
3. **Gate matches** (`BoltLost`): call `teardown_installed_during`:
   - Remove the `#installed[0]` entry from `BoundEffects`.
   - If the During was active (source is in `DuringActive`), call `reverse_scoped_tree_by_source` on the inner — this uses `reverse_all_by_source_dispatch` to remove every effect instance fired from that install key.
   - Remove the install key from `DuringActive`.
4. **After teardown**: remove `source` from `UntilApplied`, remove the Until entry from `BoundEffects`.

The `teardown_installed_during` helper is what makes Shape 4 work. It bypasses the normal condition-poller deactivation path (which would only run on the next tick when the condition becomes false) and applies the reversal immediately.

## Why no desugaring

The previous design split Until into a desugaring system that rewrote each Until into `When + Reverse` nodes. That approach had two problems:

- It required a `Reverse` node type in the `Tree` enum that only existed transiently — never seen by RON, never produced outside the desugaring pass. The grammar leaked a node that was an implementation detail.
- It required two passes per tick: the desugaring system first, then trigger walking. Effects fired during desugaring couldn't depend on trigger context (because there was none yet), so the desugaring couldn't fully resolve participant-redirecting Untils.

The current design folds everything into a single command that runs at flush time with `&mut World`. The `Reverse` node is gone. There is no desugaring pass. The state machine is local to one component (`UntilApplied`) plus the existing `BoundEffects`/`StagedEffects`.

## Common authoring patterns

### Timed buff

```ron
Until(TimeExpires(2.0), Fire(SpeedBoost(multiplier: 1.3)))
```

Fires the speed boost. Two seconds later, `TimeExpires(2.0)` fires (the time category bridges its tick down to a Trigger), which matches the gate, reverses the boost, and removes the entry.

### Buff until perfect bump

```ron
On(Bump(Bolt),
    Until(PerfectBumpOccurred,
        Sequence([
            Fire(DamageBoost(multiplier: 2.0)),
            Fire(SizeBoost(multiplier: 1.5)),
        ])))
```

Wrapped in a top-level `On` so the chip dispatches the effect via a participant redirect. The Until activates on the first bump (any kind), fires both buffs, and stays armed until a `PerfectBumpOccurred` ends the buff.

### Persistent buff while alive (via Shape 4)

```ron
Until(BoltLostOccurred,
    During(NodeActive,
        Fire(SpeedBoost(multiplier: 1.5))))
```

Outer Until installs the inner During on first walk; the condition poller activates the speed boost while the node is playing; the moment `BoltLost` fires, the teardown helper unwinds the During and reverses the boost in the same tick, then removes the Until entry.

## Interaction with chip dispatch

Chip dispatch can freely stamp Untils into entities — the walker handles them. There's no special path for "this chip needs an Until to be applied at dispatch." A chip whose effects list contains a top-level Until will simply have it walked on the next tick after dispatch, and the state machine takes over from there.

The same applies to `Once(_, Until(...))` and `When(_, Until(...))` — when the outer gate matches and `evaluate_when` / `evaluate_once` arms the Until via a stage, the next walk against the freshly-staged Until queues an `UntilEvaluateCommand` and the state machine begins.
