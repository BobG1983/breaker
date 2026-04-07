# Cell Types (Node Sequencing Context)

Cell modifier design and implementation lives in the [cell builder pattern todo](../cell-builder-pattern/cell-modifiers.md). This file covers only how cell modifiers interact with the node sequencing / tier system.

Assumes hazards and protocols already exist as implemented systems.

## Modifier Introduction Timeline

| Tier | New Modifier | Why Here |
|------|-------------|----------|
| 1 | Volatile | Easiest special — explosions teach AoE early |
| 3 | Sequence | Requires reading the field, introduces target prioritization |
| 4 | Survival | Introduces dodge-while-clearing pressure |
| 5 | Armored | Introduces directional precision, interacts with Piercing |
| 6 | Phantom | Introduces timing layer |
| 7 | Magnetic | Most complex modifier, saved for when players are ready |

Portal modifier appears from tier 5 in volatile nodes only. See [cell-modifiers.md](../cell-builder-pattern/cell-modifiers.md) for portal constraints.

## Tier Pool Weights
Each tier has a weighted pool of available modifiers for skeleton resolution. Higher tiers weight harder modifiers more heavily. Exact weights TBD — needs data structure design.

## Tough Is Not a Modifier
"Tough" (`tough.cell.ron`) is just a standard cell with more HP. It should be removed; tier-based HP scaling handles difficulty. All cells are standard cells with HP, plus zero or more modifiers.

## Needs Detail
- Tier-to-modifier-pool mapping (weights per tier)
- How skeleton resolution picks modifiers from the pool
- Modifier combination rules (which stacks are valid?)
- How sequence groups scope across blocks in a frame
