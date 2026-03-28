# Piercing

Bolt passes through cells it destroys instead of bouncing off them.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `count` | `u32` | Number of cells to pass through |

## Behavior

Pushes the count to the entity's `ActivePiercings` vec. Total pierce count = `sum(all_entries)`. When the bolt hits a cell and destroys it, piercing is decremented by 1 and the bolt continues in the same direction (no reflection). When piercing reaches 0, or the bolt hits a cell it fails to destroy, the bolt bounces as normal.

## Stacking

Additive. Multiple applications add together.

## Reversal

Removes the matching count entry from `ActivePiercings`.
