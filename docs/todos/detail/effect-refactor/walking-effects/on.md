# On

## Receives
`On(ParticipantTarget, Terminal)` — a redirect to a different entity, plus a terminal to evaluate on that entity.

## Behavior

1. Resolve the ParticipantTarget to a specific entity using the trigger context. See target-resolution/on-targets.md for the resolution table.
2. If the participant entity does not exist (despawned, or Killer absent for environmental death), stop. Do nothing.
3. Evaluate the terminal on the resolved entity (not the Owner):
   - `Fire(EffectType)` — call `fire_effect` with the participant entity.
   - `Route(RouteType, Tree)` — call `route_effect` with the participant entity.

## Constraints

- DO switch the target entity to the resolved participant for the terminal evaluation.
- DO NOT evaluate the terminal on the Owner. The whole point of On is to redirect.
- DO NOT recurse into the participant's own BoundEffects/StagedEffects. On evaluates a single terminal on the participant, it does not trigger a full walk of the participant's trees.
- DO pass the source string through so route_effect entries on the participant can be traced back to the original source.
