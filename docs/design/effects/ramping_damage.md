# RampingDamage

Stacking damage bonus on consecutive impacts. No maximum cap.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `damage_per_trigger` | `f32` | Damage bonus added per trigger activation |

## Behavior

Each impact (any `Impacted(*)` trigger — Cell, Wall, Breaker, Bolt) adds `damage_per_trigger` to the entity's accumulated damage bonus. The bonus resets on missed bump (`NoBump` trigger). No maximum cap — limited only by chip selection and gameplay. The accumulated bonus is added to the entity's damage on each hit.

## Reversal

Removes the `RampingDamageState` component and resets accumulated bonus.
