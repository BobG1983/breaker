# Sequence

## Receives
`Sequence(Vec<Terminal>)` — an ordered list of terminals to evaluate.

## Behavior

1. Iterate the terminals left to right.
2. For each terminal, evaluate it:
   - `Fire(EffectType)` — call `fire_effect`.
   - `Route(RouteType, Tree)` — call `route_effect` on the target entity.
3. Done.

## Constraints

- DO evaluate in order, left to right. Order matters — earlier effects may affect later ones.
- DO NOT skip terminals. Every terminal in the sequence is evaluated.
- DO NOT reverse order during forward evaluation. Reverse order only applies when a scoped Sequence (inside During/Until) is being reversed.
