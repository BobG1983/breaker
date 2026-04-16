# Tree Node Types

The `Tree` enum is the recursive node type stored in `BoundEffects` and `StagedEffects`. This file describes each variant, what it does at walk time, and the structural rules that distinguish `Tree` from `ScopedTree`.

```rust
pub enum Tree {
    Fire(EffectType),
    When(Trigger, Box<Self>),
    Once(Trigger, Box<Self>),
    During(Condition, Box<ScopedTree>),
    Until(Trigger, Box<ScopedTree>),
    Sequence(Vec<Terminal>),
    On(ParticipantTarget, Terminal),
}
```

The walker entry point is `evaluate_tree` in `walking/walk_effects/system.rs`. It dispatches each variant to a per-node evaluator in the corresponding `walking/<variant>/system.rs` file.

## Fire

```rust
Fire(EffectType)
```

Terminal. Fires the effect on the entity that owns this tree (the **Owner** — the entity whose `BoundEffects`/`StagedEffects` is being walked).

`evaluate_fire` queues a `FireEffectCommand`. At command flush time, the command calls `fire_dispatch(effect, entity, source, &mut world)`, which matches on the `EffectType` variant and calls `config.fire(entity, source, world)` on the relevant config struct.

```ron
Fire(SpeedBoost(multiplier: 1.5))
Fire(Shockwave(base_range: 24.0, range_per_level: 6.0, stacks: 1, speed: 400.0))
```

## When

```rust
When(Trigger, Box<Tree>)
```

Repeating gate. If the active trigger equals the gate trigger, evaluate the inner tree. Otherwise no-op. `When` entries in `BoundEffects` re-evaluate every tick — they are not consumed.

`evaluate_when` has special handling for the case where the inner tree is itself a gate (`When`, `Once`, or `Until`):

- **Inner is a gate** → stage the inner tree under the same source via `commands.stage_effect`. The staged entry will be evaluated against the *next* matching trigger, not the one that armed it. This is what gives nested `When(A, When(B, Fire(X)))` chains their "two events required" semantics.
- **Inner is anything else** (`Fire`, `Sequence`, `On`, `During`) → recurse via `evaluate_tree`.

```ron
When(PerfectBumped, Fire(SpeedBoost(multiplier: 1.5)))
When(PerfectBumped, When(Impacted(Cell), Fire(Shockwave(...))))
```

## Once

```rust
Once(Trigger, Box<Tree>)
```

Same as `When`, but self-removes from `BoundEffects` after the first match. Implementation is `evaluate_once`:

- If the inner is a nested gate, the order is **remove first, then stage**: `RemoveEffectCommand` sweeps both `BoundEffects` and `StagedEffects` by name, so queuing the remove first clears the outer entry without wiping the freshly-staged inner.
- If the inner is anything else, walker recurses normally and then queues the remove.

```ron
Once(BoltLostOccurred, Fire(SecondWind(())))
```

## During

```rust
During(Condition, Box<ScopedTree>)
```

State-scoped. Applies the inner scoped tree's effects while the condition is true; reverses them when the condition becomes false. The condition cycles can fire/reverse repeatedly across the lifetime of the chip.

`During` is **not** processed by triggers. The walker's `evaluate_during` queues a `DuringInstallCommand` that idempotently inserts the `During` as a top-level entry in `BoundEffects`. The condition poller (`evaluate_conditions`, in `EffectV3Systems::Conditions`) iterates all installed Durings every tick and fires/reverses them on transitions. See `conditions.md` for the full state machine.

```ron
During(NodeActive, Fire(SpeedBoost(multiplier: 1.3)))
During(ShieldActive, Sequence([SpeedBoost(multiplier: 1.5), DamageBoost(multiplier: 2.0)]))
```

The inner is a `ScopedTree`, so direct `Fire` is reversibility-restricted (see [Scoped restrictions](#scoped-restrictions) below).

## Until

```rust
Until(Trigger, Box<ScopedTree>)
```

Event-scoped. Applies effects immediately, reverses them when the trigger fires. Different from `During` because it's edge-driven, not state-driven, and self-removes after the gate fires.

`evaluate_until` queues an `UntilEvaluateCommand`. The command runs a small state machine using the per-entity `UntilApplied` component to track which sources have already fired their inner effects. The state machine handles four shapes — see `until.md` for the full breakdown. Importantly: there is **no desugaring step**. Until is not rewritten into `When + Reverse` nodes. It is a direct state machine inside the command.

```ron
Until(TimeExpires(2.0), Fire(SpeedBoost(multiplier: 1.3)))
```

## Sequence

```rust
Sequence(Vec<Terminal>)
```

Ordered multi-execute. Walks `Terminal`s left to right. Each terminal is either a `Fire(EffectType)` (queues `FireEffectCommand`) or a `Route(RouteType, Tree)` (queues `RouteEffectCommand` with the terminal source). See `evaluate_sequence` and `evaluate_terminal` in `walking/sequence.rs`.

```ron
Sequence([
    Fire(SpeedBoost(multiplier: 1.2)),
    Fire(DamageBoost(multiplier: 1.5)),
    Fire(Piercing(charges: 3)),
])
```

## On

```rust
On(ParticipantTarget, Terminal)
```

Redirects a terminal to a participant of the active trigger event. `evaluate_on` resolves the participant by pattern-matching `(ParticipantTarget, TriggerContext)` — see `walking/on/system.rs`. Resolution returns `Option<Entity>` because some participants are optional (`Death.killer`, `Bump.bolt` for whiff/no-bump).

If the source string is an "armed" key (matches the `#armed[...]` naming convention used by `evaluate_conditions` for Shape D), `evaluate_on` also queues a `TrackArmedFireCommand` on the owner so the disarm path can later reverse effects on the exact participants they fired on.

```ron
On(Bump(Bolt), Fire(SpeedBoost(multiplier: 1.5)))
On(Death(Killer), Route(Staged, When(NodeEndOccurred, Fire(LoseLife(())))))
```

`Tree::On` only carries a single `Terminal`, not a tree. To redirect a multi-step subtree, wrap a `Route` terminal whose payload is the subtree.

## Scoped restrictions: Tree vs ScopedTree

Both `Tree::During` and `Tree::Until` take a `ScopedTree` rather than a `Tree`. The restriction is structural and enforced at the type level:

```rust
pub enum ScopedTree {
    Fire(ReversibleEffectType),       // reversible only — fire/reverse must be defined
    Sequence(Vec<ReversibleEffectType>), // every element must be reversible
    When(Trigger, Box<Tree>),          // nested When re-opens to full Tree
    On(ParticipantTarget, ScopedTerminal),
    During(Condition, Box<Self>),      // nested During stays scoped
}
```

The reversibility constraint is the whole point of the split: `During` and `Until` *will* call `reverse()` on their inner `Fire`s when the scope ends, so a non-reversible effect in direct `Fire` position would be invalid. Wrapping the same effect in a nested `When` re-opens the inner type to the full `Tree`, because the `When` listener (not the effect) is what gets removed at scope end.

| Position | Reversibility required |
|---|---|
| `ScopedTree::Fire(_)` | Yes — must be `ReversibleEffectType` |
| `ScopedTree::Sequence(_)` | Yes — every element must be reversible |
| `ScopedTree::When(_, Tree)` | No — inner `Tree` is unrestricted |
| `ScopedTree::On(_, ScopedTerminal)` | `ScopedTerminal::Fire` is reversible-only |
| `ScopedTree::During(_, Self)` | Self-recursive — same restrictions apply |
| `Tree::Fire(_)` (outside scope) | No — any `EffectType` allowed |

This is also what the `ReversibleEffectType` enum exists for. There's no validation step at load time — the loader can simply not produce a `ScopedTree::Fire(non_reversible)` because the type doesn't allow it.

## Terminal vs Tree

`Terminal` is the leaf type used inside `Sequence` and `On`. The split exists so that `Sequence` can avoid the `Box<Tree>` indirection on each child, and so `On` can clearly express "redirect this single thing" rather than "redirect a whole tree" (which would be ambiguous about which entity owns the redirected subtree).

```rust
pub enum Terminal {
    Fire(EffectType),              // fire on Owner (or redirected target via On)
    Route(RouteType, Box<Tree>),   // install on Owner with bound/staged permanence
}

pub enum ScopedTerminal {
    Fire(ReversibleEffectType),
    Route(RouteType, Box<Tree>),
}
```

`Terminal::Route(Bound, ...)` is equivalent to a `commands.stamp_effect`. `Terminal::Route(Staged, ...)` is equivalent to a `commands.stage_effect`. Both queue `RouteEffectCommand`, which then calls the appropriate stamp/stage logic.

## What about the old `Reverse` node?

There used to be a node-graph `Reverse` variant (and a corresponding `When + Reverse` desugaring step for `Until`). Neither exists in the current code. `Until` and `During` reversal now happens via direct calls to `reverse_dispatch` / `reverse_all_by_source_dispatch` from inside `UntilEvaluateCommand` and `evaluate_conditions`. The walker never sees a "reverse this" node — reversal is a function call, not a node type.
