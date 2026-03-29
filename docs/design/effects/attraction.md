# Attraction

Attracts the entity toward the nearest entity of a specified type.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `attraction_type` | `AttractionType` | `Cell`, `Wall`, or `Breaker` |
| `force` | `f32` | Attraction strength |
| `max_force` | `Option<f32>` | Optional maximum force magnitude per tick; clamps velocity delta |

## Behavior

Steers the entity toward the nearest entity of the specified type. If multiple attraction types are active, the closest target of any type determines the pull direction (nearest wins).

When `max_force` is `Some(cap)`, the effective force applied each tick is `min(force, cap)`. This prevents excessive steering at close range. When `max_force` is `None`, force is applied without capping (preserves original behavior).

**Type deactivation**: attraction toward a type deactivates when the entity hits that type, reactivates when it bounces off a non-attracted type.

## Reversal

Removes the matching attraction entry from the entity's active attractions. Matching uses `attraction_type`, `force`, and `max_force` as the entry identity.
