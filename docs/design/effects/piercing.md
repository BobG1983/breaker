# Piercing

Bolt passes through cells instead of bouncing off them.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `count` | `u32` | Number of cells to pass through |

## Behavior

Pushes the count to the entity's ActivePiercings vec. Total pierce count = `sum(all_entries)`. The bolt passes through that many cells before bouncing.

## Stacking

Additive. Multiple applications add together.

## Reversal

Removes the matching count entry from ActivePiercings.
