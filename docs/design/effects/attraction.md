# Attraction

Attracts the entity toward the nearest entity of a specified type.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `type` | `AttractionType` | `Cell`, `Wall`, or `Breaker` |
| `force` | `f32` | Attraction strength |

## Behavior

Steers the entity toward the nearest entity of the specified type. If multiple attraction types are active, the closest target of any type determines the pull direction (nearest wins).

**Type deactivation**: attraction toward a type deactivates when the entity hits that type, reactivates when it bounces off a non-attracted type.

## Reversal

Removes the attraction entry from the entity's active attractions.
