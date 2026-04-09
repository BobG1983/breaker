# Adding a New Condition

Step-by-step reference for adding a condition to the new effect system. Conditions are state-based (not event-based) and power `During(condition, ...)` entries.

## 1. Add variant to Condition enum

In `effect/core/types/definitions/enums.rs`:

```rust
enum Condition {
    NodeActive,
    ShieldActive,
    ComboActive(u32),
    // NewCondition,
}
```

Parameterized conditions (like `ComboActive(u32)`) support per-threshold activation.

## 2. Write condition monitor system

One system per condition. Each watches for its specific state change and activates/deactivates matching During entries in `BoundEffects.conditions`.

```rust
fn monitor_new_condition(
    // State source (Res<State<T>>, Query<Added<T>>, RemovedComponents<T>, etc.)
    mut query: Query<(Entity, &mut BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    // Detect start/end transitions
    // For each entity with matching During(NewCondition, ...) entries:
    //   On start → activate_during(...)
    //   On end   → deactivate_during(...)
}
```

The monitor must track previous state to detect transitions (not just current state). Use `Local<T>`, `Res<State<T>>`, `Added<T>`, or `RemovedComponents<T>` as appropriate.

## 3. On condition start: activate During entries

For each `During(NewCondition, inner)` entry in `BoundEffects.conditions`:

| Inner tree shape | Activation action |
|---|---|
| `Fire(reversible_effect)` | `commands.fire_effect(entity, effect, source, context)` |
| `Sequence([Fire(a), Fire(b)])` | Fire all children in order |
| `When(trigger, inner)` | Register the When into `BoundEffects.triggers` with a scope source |
| `On(target, Fire(effect))` | Fire on resolved target entity |

**Scope source** for nested When registration: `format!("{source}:During({condition:?})")`. This allows targeted cleanup on deactivation.

## 4. On condition end: deactivate During entries

For each active `During(NewCondition, inner)` entry:

| Inner tree shape | Deactivation action |
|---|---|
| `Fire(reversible_effect)` | `commands.reverse_effect(entity, effect, source)` |
| `Sequence([Fire(a), Fire(b)])` | Reverse all children in reverse order |
| `When(trigger, inner)` | Unregister from `BoundEffects.triggers` by scope source + clean armed StagedEffects entries with same scope source |
| `On(target, Fire(effect))` | Reverse on resolved target entity |

Cleaning StagedEffects on deactivation prevents stale armed entries from firing after the condition ends.

## 5. Register monitor system

Register in the effect plugin:
- After the systems that produce the monitored state
- Condition monitors defer effect execution via `EffectCommandsExt` on `Commands` (queue fire/reverse, don't execute inline)
- Conditions can cycle — re-activate on next start, re-deactivate on next end

## 6. Update conditions table

Add the new condition to `docs/design/triggers/index.md` and `docs/architecture/effects/conditions.md` with its start/end definitions.

## During lifecycle notes

- During entries live in `BoundEffects.conditions` permanently — they are never consumed
- Condition cycling is handled by monitors calling activate/deactivate on each transition
- Each condition cycles independently of other conditions
- Direct `Fire` inside During requires the effect to be reversible (enforced by builder and RON loader)
- Nested `When` inside During relaxes to any effect (the listener registration is what gets reversed)
