# During

## Receives
`During(Condition, Box<ScopedTree>)` — a state-scoped node with a condition and a scoped inner tree.

## Behavior

During tracks whether its condition is currently true or false and responds to transitions.

**When the condition becomes true (was false, now true):**

1. Evaluate the ScopedTree's immediate children:
   - `Fire(ReversibleEffectType)` — call `fire_effect` with the reversible effect.
   - `Sequence([ReversibleEffectType, ...])` — call `fire_effect` for each effect in order, left to right.
   - `When(Trigger, Tree)` — install the When as a listener (it becomes active and can match triggers).
   - `On(ParticipantTarget, ScopedTerminal)` — resolve participant, evaluate the scoped terminal on that entity.

**When the condition becomes false (was true, now false):**

1. Reverse the ScopedTree's immediate children:
   - `Fire(ReversibleEffectType)` — call `reverse_effect` with the same effect and source.
   - `Sequence([ReversibleEffectType, ...])` — call `reverse_effect` for each effect in reverse order, right to left.
   - `When(Trigger, Tree)` — remove the When listener. Individual effects that already fired from past trigger matches are NOT reversed.
   - `On(ParticipantTarget, ScopedTerminal)` — reverse the scoped terminal on the same participant entity.

**When the condition cycles (becomes true again after being false):**

1. Re-apply the ScopedTree as if the condition just became true. During can cycle indefinitely.

## Constraints

- DO track the condition state (true/false) so transitions can be detected.
- DO reverse in the opposite order of application for Sequence children.
- DO NOT reverse individual effects that fired from a When listener inside the scope — only remove the listener itself.
- DO NOT remove the During from BoundEffects. It stays permanently and cycles with its condition.
