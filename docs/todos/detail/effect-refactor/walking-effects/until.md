# Until

## Receives
`Until(Trigger, Box<ScopedTree>)` — an event-scoped node with a trigger and a scoped inner tree.

## Behavior

Until applies its scoped effects immediately and keeps them active until the trigger fires.

**On installation (when the Until is first encountered during walking):**

1. Apply the ScopedTree's immediate children, same as During's "condition becomes true" behavior:
   - `Fire(ReversibleEffectType)` — call `fire_effect`.
   - `Sequence([ReversibleEffectType, ...])` — call `fire_effect` for each effect in order.
   - `When(Trigger, Tree)` — install the When as a listener via `stage_effect`.
   - `On(ParticipantTarget, ScopedTerminal)` — resolve participant, evaluate the scoped terminal.
   - `During(Cond, inner)` — install via `DuringInstallCommand`, appending to `BoundEffects` under source key `{original_source}#installed[0]`. (Shape B) The installed During is then managed by `evaluate_conditions` on the next frame.

2. If the Until's trigger is `TimeExpires(duration)`, register a timer:
   - Push `(duration, duration)` onto the entity's `EffectTimers` component (insert the component if absent).
   - The `tick_effect_timers` system counts down the timer each frame.
   - When remaining reaches 0, `EffectTimerExpired` is sent, and `on_time_expires` dispatches `TimeExpires(duration)`.
   - That dispatch matches this Until's trigger and triggers reversal (step below).

**When the Until's trigger fires:**

1. Reverse the ScopedTree's immediate children, same as During's "condition becomes false" behavior:
   - `Fire(ReversibleEffectType)` — call `reverse_effect`.
   - `Sequence([ReversibleEffectType, ...])` — call `reverse_effect` in reverse order.
   - `When(Trigger, Tree)` — remove the listener via `remove_effect`.
   - `On(ParticipantTarget, ScopedTerminal)` — reverse on the participant.
   - `During(Cond, inner)` — remove the `{original_source}#installed[0]` entry from BoundEffects. If the During was active (`source` is in `DuringActive`), call `reverse_all_by_source_dispatch` to bulk-reverse any effects that fired while the condition was active. (Shape B teardown)

2. Queue `remove_effect(entity, Bound, source, tree)` to remove the Until entry from BoundEffects. It is one-shot — it does not re-arm.

**Multiple Until entries with the same TimeExpires duration:** If two different Until entries on the same entity both use `TimeExpires(5.0)`, both will match when the timer fires. Both reverse their scoped effects. This is correct behavior — independent effects with the same duration are independent.

## Constraints

- DO apply effects immediately on installation, not on a trigger match.
- DO register a timer on installation when the trigger is TimeExpires.
- DO install nested During entries via `DuringInstallCommand` (with idempotency guard).
- DO reverse and remove when the trigger fires. Until is one-shot, not cycling like During.
- DO NOT re-arm. Once reversed, the Until is gone.
- DO reverse in the opposite order of application for Sequence children.
- DO NOT reverse individual effects that fired from a When listener inside the scope.
- DO call `reverse_all_by_source_dispatch` when tearing down an active nested During (Shape B).
- DO use deferred commands (fire_effect, reverse_effect, remove_effect) for all mutations.
