# ChainHit

Chains to additional cells on hit.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `count` | `u32` | Number of additional cells to chain to |

## Behavior

When the entity hits a cell, the hit chains to `count` additional nearby cells, damaging them as well.

## Reversal

Removes the chain hit component from the entity.
