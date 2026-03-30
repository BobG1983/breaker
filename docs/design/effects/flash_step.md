# FlashStep

Breaker teleport on dash reversal during settling. When the breaker is settling after a dash and the player dashes in the opposite direction, the breaker teleports to the destination instead of sliding.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| (none currently) | | Teleport is binary — either active or not |

## Behavior

When active on a breaker:
1. The breaker dashes normally in a direction
2. During the **settling** phase (post-dash deceleration), if the player initiates a dash in the **opposite direction**:
   - Instead of the normal dash animation, the breaker **teleports** instantly to the dash endpoint
   - The settling phase is skipped entirely
3. Normal same-direction dashes behave as usual

The point is to skip the settling penalty for direction reversal — rewarding players who can read the bolt's trajectory and react with a reversal.

`fire()` inserts `FlashStepActive` on the breaker entity. The breaker's dash system reads this component to determine whether a reversal-during-settling should teleport.

## Reversal

Removes `FlashStepActive` from the breaker. Dashes return to normal behavior.

## Ingredients

Breaker Speed x2 + Reflex x1.

## VFX

- On teleport trigger: Breaker disintegrates into energy streak particles at departure point
- Light-streak connects departure and arrival positions (1-2 frames)
- Departure afterimage fades ~0.3s
- Arrival: radial distortion burst + rematerialization (particles converge to form breaker)
- Small screen shake on arrival
