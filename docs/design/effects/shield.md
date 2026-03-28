# Shield

Temporary protection — dual behavior depending on entity type.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_duration` | `f32` | Base duration in seconds |
| `duration_per_level` | `f32` | Extra duration per stack |
| `stacks` | `u32` | Stack count |

Effective duration = `base_duration + (stacks - 1) * duration_per_level`.

## Behavior

Inserts `ShieldActive` component on the entity. Behavior depends on entity type:

- **On Breaker**: immune to bolt loss for the duration (bolts bounce off bottom instead of being lost)
- **On any entity with a health pool**: immune to damage for the duration

Timer decrements each tick. When it expires, `ShieldActive` is removed. Multiple fires extend the remaining duration (additive).

## Reversal

Removes the `ShieldActive` component from the entity.
