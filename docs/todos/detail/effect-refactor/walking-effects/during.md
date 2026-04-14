# During

## Receives
`During(Condition, Box<ScopedTree>)` — a state-scoped node with a condition and a scoped inner tree.

## Behavior

During tracks whether its condition is currently true or false and responds to transitions.

**When the condition becomes true (was false, now true):**

1. Evaluate the ScopedTree's immediate children:
   - `Fire(ReversibleEffectType)` — call `fire_effect` with the reversible effect.
   - `Sequence([ReversibleEffectType, ...])` — call `fire_effect` for each effect in order, left to right.
   - `When(Trigger, Tree)` — install the When as a listener via `install_armed_entry()`. The entry is appended to `BoundEffects` (or `StagedEffects`) under source key `{original_source}#armed[0]`. (Shape C)
   - `On(ParticipantTarget, ScopedTerminal)` — resolve participant, evaluate the scoped terminal on that entity. (Shape D)

**When the condition becomes false (was true, now false):**

1. Reverse the ScopedTree's immediate children:
   - `Fire(ReversibleEffectType)` — call `reverse_effect` with the same effect and source.
   - `Sequence([ReversibleEffectType, ...])` — call `reverse_effect` for each effect in reverse order, right to left.
   - `When(Trigger, Tree)` — remove the armed entry by source (`{original_source}#armed[0]`). Then call `reverse_all_by_source_dispatch` to bulk-reverse all effects that fired from that scope while the condition was active. (Shape C)
   - `On(ParticipantTarget, ScopedTerminal)` — reverse on the same participant entity, with `reverse_all_by_source_dispatch`. (Shape D)

**When the condition cycles (becomes true again after being false):**

1. Re-apply the ScopedTree as if the condition just became true. During can cycle indefinitely.

## Shape A Install — `When(X, During(Cond, inner))`

When walking encounters a `During` node that is itself the inner of a `When` node (Shape A), the walker calls `DuringInstallCommand`. This command:

1. Checks whether a `BoundEffects` entry with source `{original_source}#installed[0]` already exists (idempotency guard).
2. If absent, appends `(source: "{original_source}#installed[0]", tree: During(Cond, inner))` to the entity's `BoundEffects`.

The installed `During` is then managed by `evaluate_conditions` on the next frame — no immediate fire occurs at install time. This is the standard mechanism for Shape A and Shape B nested conditions.

## Constraints

- DO track the condition state in `DuringActive` (source present = true, absent = false).
- DO reverse in the opposite order of application for Sequence children.
- DO call `reverse_all_by_source_dispatch` (not just `reverse_effect`) when disarming a `When` or `On` child — all previously fired effects from that scope must be cleaned up.
- DO NOT remove the During from BoundEffects. It stays permanently and cycles with its condition.
- DO use `DuringInstallCommand` for Shape A/B installs to get idempotency guarantees.
