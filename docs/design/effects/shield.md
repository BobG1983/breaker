# Shield

Temporary protection -- dual behavior depending on entity type.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `stacks` | `u32` | Number of shield charges |

## Behavior

Inserts `ShieldActive { charges: stacks }` component on the entity. Behavior depends on entity type:

- **On Breaker**: absorbs bolt losses. Each bolt saved by the shield costs one charge. Multiple bolts lost in the same frame each consume one charge independently. When charges are exhausted, remaining bolts fall through to normal handling. When charges reach zero, `ShieldActive` is removed.
- **On any entity with a health pool**: immune to damage while `ShieldActive` is present (unchanged).

Multiple fires add charges to any existing shield (additive stacking).

## Reversal

Removes the `ShieldActive` component from the entity.
