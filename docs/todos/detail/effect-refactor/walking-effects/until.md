# Until

## Receives
`Until(Trigger, Box<ScopedTree>)` — an event-scoped node with a trigger and a scoped inner tree.

## Behavior

Until applies its scoped effects immediately and keeps them active until the trigger fires.

**On installation (when the Until is first encountered during walking):**

1. Apply the ScopedTree's immediate children, same as During's "condition becomes true" behavior:
   - `Fire(ReversibleEffectType)` — call `fire_effect`.
   - `Sequence([ReversibleEffectType, ...])` — call `fire_effect` for each effect in order.
   - `When(Trigger, Tree)` — install the When as a listener.
   - `On(ParticipantTarget, ScopedTerminal)` — resolve participant, evaluate the scoped terminal.

**When the Until's trigger fires:**

1. Reverse the ScopedTree's immediate children, same as During's "condition becomes false" behavior:
   - `Fire(ReversibleEffectType)` — call `reverse_effect`.
   - `Sequence([ReversibleEffectType, ...])` — call `reverse_effect` in reverse order.
   - `When(Trigger, Tree)` — remove the listener.
   - `On(ParticipantTarget, ScopedTerminal)` — reverse on the participant.
2. Remove the Until entry from BoundEffects. It is one-shot — it does not re-arm.

## Constraints

- DO apply effects immediately on installation, not on a trigger match.
- DO reverse and remove when the trigger fires. Until is one-shot, not cycling like During.
- DO NOT re-arm. Once reversed, the Until is gone.
- DO reverse in the opposite order of application for Sequence children.
- DO NOT reverse individual effects that fired from a When listener inside the scope.
