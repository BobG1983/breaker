# Evaluation Flow

Tree evaluation happens in two stages, both driven by trigger bridge systems:

1. **Walk** — a bridge system has just translated a game message into a `Trigger` and a `TriggerContext`. It calls `walk_staged_effects` followed by `walk_bound_effects` for each entity that should see this trigger.
2. **Evaluate** — the walker iterates the entity's `(name, tree)` entries and calls `evaluate_tree`. `evaluate_tree` pattern-matches on the `Tree` variant and calls the appropriate per-node evaluator (`evaluate_fire`, `evaluate_when`, `evaluate_once`, `evaluate_during`, `evaluate_until`, `evaluate_sequence`, `evaluate_on`).

All walker code lives under `effect_v3/walking/`. The walker queues commands via `EffectCommandsExt` — it never touches `&mut World` directly. Effect execution happens later, when the queued commands flush.

## walk_bound_effects vs walk_staged_effects

```rust
pub(in crate::effect_v3) fn walk_bound_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    trees: &[(String, Tree)],
    commands: &mut Commands,
);

pub(in crate::effect_v3) fn walk_staged_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    trees: &[(String, Tree)],
    commands: &mut Commands,
);
```

Both are scoped to `pub(in crate::effect_v3)` so external callers must go through `EffectCommandsExt` rather than the walker directly.

`walk_bound_effects` simply iterates entries and calls `evaluate_tree` on each. Bound entries persist after evaluation — only `commands.remove_effect` (or `Tree::Once` self-removal) ever clears them.

`walk_staged_effects` adds two behaviors on top of the iteration:

1. **Top-level gate filter** — before evaluating, it checks that the entry's root variant is `When`/`Once`/`Until` and that the gate trigger equals the active trigger. Non-gate roots and non-matching gates are skipped without consumption. This avoids burning a staged entry that would otherwise no-op.
2. **Entry-specific consumption** — after evaluating a matching entry, it queues `commands.remove_staged_effect(entity, source.clone(), tree.clone())`. The remove is keyed on the exact `(source, tree)` tuple, not on `source` alone. This preserves any fresh same-name stages queued during evaluation.

## Bridge call order

Bridge systems must call `walk_staged_effects` **before** `walk_bound_effects` for the same `(entity, trigger, context)` tuple. The reasoning is documented in the walker itself:

> *Bridge systems must call this BEFORE `walk_bound_effects` so that entries staged by a `When` / `Once` arming during the same trigger event do not erroneously match the trigger that staged them.*

A bound `When(A, When(B, Fire(X)))` walked first would match `A`, arm a fresh `When(B, ...)` in `StagedEffects`, and if `walk_staged_effects` then runs against the still-active trigger `A`, the freshly-armed inner is walked too — but `B != A`, so the staged entry is skipped (correct). If however a bound `When(A, When(A, Fire(X)))` was walked first, the freshly-armed inner has the same trigger as the active one, so a subsequent staged walk would consume it on the same event, breaking the documented "two events required" semantics. Walking staged first prevents this entire class of issue.

This ordering is enforced inside each bridge system body, not at the schedule level — schedulers don't model "this happens before this within a single system tick."

## evaluate_tree dispatch

```rust
pub(in crate::effect_v3) fn evaluate_tree(
    entity: Entity,
    tree: &Tree,
    trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    match tree {
        Tree::Fire(effect_type)        => evaluate_fire(entity, effect_type, source, context, commands),
        Tree::When(gate, inner)        => evaluate_when(entity, gate, inner, trigger, context, source, commands),
        Tree::Once(gate, inner)        => evaluate_once(entity, gate, inner, trigger, context, source, commands),
        Tree::During(condition, inner) => evaluate_during(entity, condition, inner, context, source, commands),
        Tree::Until(gate, inner)       => evaluate_until(entity, gate, inner, trigger, context, source, commands),
        Tree::Sequence(terminals)      => evaluate_sequence(entity, terminals, context, source, commands),
        Tree::On(target, terminal)     => evaluate_on(entity, *target, terminal, context, source, commands),
    }
}
```

## Per-node evaluators

### evaluate_fire (`walking/fire.rs`)

Queues a `FireEffectCommand`. No conditional logic — every walk of a `Fire` produces exactly one fire command.

### evaluate_when (`walking/when/system.rs`)

```rust
if gate_trigger != active_trigger { return; }
match inner {
    Tree::When(..) | Tree::Once(..) | Tree::Until(..) => {
        commands.stage_effect(entity, source.to_owned(), inner.clone());
    }
    _ => {
        evaluate_tree(entity, inner, active_trigger, context, source, commands);
    }
}
```

The "arm-on-nested-gate" branch is what makes nested gate trees ladder across trigger events instead of collapsing in a single tick. The staged entry is evaluated against the *next* matching trigger.

### evaluate_once (`walking/once/system.rs`)

Same gate check as `When`, plus self-removal:

```rust
if gate_trigger != active_trigger { return; }
match inner {
    Tree::When(..) | Tree::Once(..) | Tree::Until(..) => {
        commands.remove_effect(entity, source);     // remove FIRST
        commands.stage_effect(entity, source.to_owned(), inner.clone());
    }
    _ => {
        evaluate_tree(entity, inner, active_trigger, context, source, commands);
        commands.remove_effect(entity, source);
    }
}
```

The remove-first / stage-second ordering for the nested-gate case is load-bearing: `RemoveEffectCommand` sweeps both `BoundEffects` and `StagedEffects` by name, so queuing the remove first clears the outer entry without touching the freshly-staged inner.

### evaluate_during (`walking/during/system.rs`)

`During` is not a triggered node — it's a state-scoped install. `evaluate_during` queues a `DuringInstallCommand` that idempotently inserts the `During` as a top-level entry in `BoundEffects` under the key `format!("{source}#installed[0]")`. The condition poller (`evaluate_conditions`, in `EffectV3Systems::Conditions`) then takes over.

If the source string already contains `"#installed"`, the walker skips re-installation — the entry being walked is a poller-managed installed entry, not an authoring path.

See `conditions.md`.

### evaluate_until (`walking/until/system.rs`)

`Until` is also not a triggered node in the usual sense — `evaluate_until` queues an `UntilEvaluateCommand` that runs a state machine inside `Command::apply`. The state machine uses the per-entity `UntilApplied` component to track which sources have already fired their inner effects.

The state machine handles four shapes (Fire/Sequence direct, plus Until-wrapping-During). See `until.md`.

### evaluate_sequence and evaluate_terminal (`walking/sequence.rs`)

```rust
pub fn evaluate_sequence(entity: Entity, terminals: &[Terminal], _context: &TriggerContext, source: &str, commands: &mut Commands) {
    for terminal in terminals {
        evaluate_terminal(entity, terminal, source, commands);
    }
}

pub fn evaluate_terminal(entity: Entity, terminal: &Terminal, source: &str, commands: &mut Commands) {
    match terminal {
        Terminal::Fire(effect_type) => commands.queue(FireEffectCommand { entity, effect: effect_type.clone(), source: source.to_owned() }),
        Terminal::Route(route_type, tree) => commands.queue(RouteEffectCommand { entity, name: source.to_owned(), tree: (**tree).clone(), route_type: *route_type }),
    }
}
```

`Sequence` is unconditional ordered fire/route. There is no gate — the surrounding `When`/`Once` provides the gate.

### evaluate_on (`walking/on/system.rs`)

```rust
if let Some(resolved) = resolve_participant(target, context) {
    evaluate_terminal(resolved, terminal, source, commands);
    if is_armed_source(source) {
        commands.track_armed_fire(owner, source.to_owned(), resolved);
    }
}
```

Resolves the participant from the `TriggerContext`, calls `evaluate_terminal` on the resolved entity, and if the source is an armed key (Shape D), additionally tracks the participant on the owner's `ArmedFiredParticipants` so the disarm path can reverse effects on the right entity later.

`is_armed_source` (in `conditions/armed_source.rs`) tests for the `#armed[` substring used by `evaluate_conditions` when installing scoped `On` entries.

## Same-tick stage-then-consume

Multiple commands queued in the same tick all flush together, in queue order. `walk_staged_effects` exploits this carefully:

> *`evaluate_tree` may queue `Stage` commands for arm-push paths. Those MUST be enqueued BEFORE the consume below so the entry-specific remove hits the original staged entry (first match on `(source, tree)`) and not the freshly-armed inner.*

So when a staged `When(A, When(A, Fire(X)))` matches against `A`:

1. `evaluate_tree` → `evaluate_when` (matches) → inner is a gate → `commands.stage_effect(entity, source, When(A, Fire(X)))`
2. After `evaluate_tree` returns, `walk_staged_effects` queues `commands.remove_staged_effect(entity, source, When(A, When(A, Fire(X))))`
3. Both commands flush. The remove finds the original outer (exact tuple match), removes it. The freshly-armed inner survives (different `Tree` value).

Next tick walks the inner `When(A, Fire(X))` against the next matching `A`, which fires `X` and removes the inner. Depth-N chains take N events to fully drain.

## Trigger sources

The bridge systems are the only callers of `walk_staged_effects` and `walk_bound_effects`. Bridge systems live in `effect_v3/triggers/<category>/bridges/system.rs` and follow a uniform pattern — see `trigger_systems.md`. Categories are `bump`, `impact`, `death`, `bolt_lost`, `node`, `time`. All bridges are registered into `EffectV3Systems::Bridge` via the per-category `register::register(app)` function called by `EffectV3Plugin::build`.

## Where conditions and timers fit

The condition poller (`evaluate_conditions`) runs in `EffectV3Systems::Conditions` after all bridges. It does not call `walk_*_effects` — it uses its own iteration over entries whose top-level variant is `Tree::During` and applies/reverses scoped trees directly. See `conditions.md`.

`TimeExpires` is a regular `Trigger` variant. The `time` trigger category owns timer ticking systems plus a bridge that converts elapsed-timer messages into `Trigger::TimeExpires(seconds)` dispatches just like any other trigger.
