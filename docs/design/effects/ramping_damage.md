# RampingDamage

Stacking damage bonus on consecutive impacts. No maximum cap.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `bonus_per_hit` | `f32` | Damage bonus added per impact |

## Behavior

Each impact (any `Impacted(*)` trigger — Cell, Wall, Breaker, Bolt) adds `bonus_per_hit` to the entity's accumulated damage bonus. The bonus resets on missed bump (`NoBump` trigger). No maximum cap — limited only by chip selection and gameplay. The accumulated bonus is added to the entity's damage on each hit.

## Reversal

Removes the `RampingDamageState` component and resets accumulated bonus.
