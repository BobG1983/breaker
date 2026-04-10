# Fire

## Receives
`Fire(EffectType)` — a leaf node with an effect to execute.

## Behavior

1. Call `fire_effect(entity, effect, source)` where entity is the current Owner being walked and source is the source string from the storage entry.
2. Done. Fire is a leaf — there is nothing to recurse into.

## Constraints

- DO execute the effect on the Owner, not on any other entity. To target a different entity, the tree must use On before reaching Fire.
- DO NOT store anything in BoundEffects or StagedEffects. Fire is immediate and stateless.
