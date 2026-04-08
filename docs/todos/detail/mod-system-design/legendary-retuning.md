# Legendary Rarity Removal + Retuning

## Summary
Delete the `Legendary` rarity. Retune 11 legendary chips as Rare. Promote 2 legendaries (Deadline, Ricochet Protocol) to protocols. Remove Anchor evolution.

## Chips to retune as Rare (11)

Each needs: current legendary values → new Rare values (tuned down).

| Chip | Current Legendary Effects | New Rare Values |
|------|-------------------------|-----------------|
| Glass Cannon | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Desperation | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Whiplash | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Singularity | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Gauntlet | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Chain Reaction | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Feedback Loop | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Parry | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Powder Keg | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Death Lightning | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |
| Tempo | **[NEEDS DETAIL]** | **[NEEDS DETAIL]** |

## Chips promoted to Protocols (2)

| Chip | Action |
|------|--------|
| Deadline | Remove `legendary:` slot from `deadline.chip.ron`. Create `assets/protocols/deadline.protocol.ron` with effect tree. |
| Ricochet Protocol | Remove `legendary:` slot from `ricochet_protocol.chip.ron`. Create `assets/protocols/ricochet_protocol.protocol.ron` with effect tree. |

## Evolution removal (1)

| Evolution | Action |
|-----------|--------|
| Anchor | Delete `assets/chips/evolutions/anchor.evolution.ron`. Create `assets/protocols/anchor.protocol.ron` with effect tree. Remove Anchor recipe from `EvolutionTemplateRegistry`. |

## Code changes

1. Remove `Legendary` variant from `Rarity` enum (`chips/definition/types.rs`)
2. Remove `rarity_weight_legendary` from `ChipSelectConfig` + `defaults.chipselect.ron`
3. Remove `legendary:` slots from 13 `.chip.ron` files
4. Add `rare:` slots (with new tuned values) to the 11 chips that only had legendary
5. Remove Legendary color config entries
6. Update any tests that reference `Rarity::Legendary`

## What's needed

For each of the 11 chips:
1. Read the current `legendary:` slot effects from the `.chip.ron` file
2. Decide on Rare values (direction: ~30% weaker than legendary, but needs per-chip judgment)
3. Write the new `rare:` slot

This is a game design + tuning task, not a pure engineering task.

## Status
`[NEEDS DETAIL]` — need per-chip Rare tuning values for the 11 legendary-to-rare conversions
