# RampingDamage

Stacking damage bonus on consecutive cell hits. No maximum cap.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `bonus_per_hit` | `f32` | Damage bonus added per consecutive cell hit |

## Behavior

Each cell hit adds `bonus_per_hit` to the entity's accumulated damage bonus. The bonus resets on missed bump (NoBump trigger). No maximum cap — limited only by chip selection and gameplay.

## Reversal

Removes the ramping damage component and resets accumulated bonus.
