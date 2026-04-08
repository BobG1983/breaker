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
Each tier has a weighted pool of available modifiers for constraint resolution, defined as a weighted list per tier in a RON file (`tier_pools.ron`). Higher tiers weight harder modifiers more heavily. Exact weight values determined through playtesting.

## Constraint Resolution
When a block cell position has a constraint (`Any`, `MustInclude`, `MustNotInclude`), the generator resolves it against the tier's modifier pool:

1. Start with the tier's full modifier pool (weighted)
2. Apply `MustInclude` — ensure these behaviors are in the result
3. Apply `MustNotInclude` — remove these from the pool before random selection
4. Roll remaining behaviors from the pool based on weights
5. Return `Vec<CellBehavior>` for `Cell::builder().with_behaviors()`

## Tough Is Not a Modifier
"Tough" (`tough.cell.ron`) is just a standard cell with more HP. It should be removed; tier-based HP scaling handles difficulty. All cells are standard cells with HP, plus zero or more modifiers.

## Resolved
- **Tier-to-modifier-pool mapping**: weighted list per tier in RON (see main doc content budget section)
- **Modifier combination rules**: deny-list in code. All combos valid by default; list only conflicting/degenerate pairs.
- **Sequence scoping**: block-authored sequences stay block-local; generator can assign cross-block sequences to eligible cells during composition.
