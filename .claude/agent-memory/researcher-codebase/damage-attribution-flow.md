---
name: damage-attribution-flow
description: End-to-end source_chip threading from ActiveChains through EffectFired to DamageCell, bolt spawn messages, and the incomplete evolution_damage pipeline
type: reference
---

# Damage Attribution Flow Map

## ActiveChains tuple structure
`ActiveChains(Vec<(Option<String>, TriggerChain)>)` -- chip_name is `None` for archetype chains, `Some(name)` for chip/evolution chains.

## Two populations
1. **Archetype init** (`init_archetype`): pushes with `(None, chain)` -- no chip attribution
2. **Chip overclock** (`handle_overclock`): pushes with `(Some(event.chip_name.clone()), chain)` -- has chip name

## Bridge systems thread chip_name into EffectFired.source_chip
All bridge systems (bridge_bump, bridge_cell_impact, bridge_breaker_impact, bridge_wall_impact, bridge_bolt_lost, bridge_bump_whiff, bridge_cell_destroyed) iterate `active.0` and pass `chip_name.clone()` as `source_chip` on every `EffectFired` they trigger.

## Effect handlers pass source_chip downstream
- `handle_shockwave` -> stores `trigger.event().source_chip` into `ShockwaveDamage.source_chip`
- `handle_spawn_bolt` -> writes `SpawnAdditionalBolt { source_chip: trigger.event().source_chip.clone() }`
- `handle_chain_bolt` -> writes `SpawnChainBolt { source_chip: event.source_chip.clone() }`

## Two DamageCell producers
1. **bolt_cell_collision** -- always writes `source_chip: None` (direct bolt-cell CCD hits)
2. **shockwave_collision** -- writes `source_chip: dmg.source_chip.clone()` (from ShockwaveDamage)

## DamageCell.source_chip is NOT consumed
`handle_cell_hit` reads DamageCell messages but ignores `source_chip`. It writes `CellDestroyed { was_required_to_clear }` with no chip attribution.

## Incomplete pipeline pieces
- `SpawnedByEvolution(String)` component: defined but never attached to any entity
- `HighlightTracker.evolution_damage: HashMap<String, f32>`: defined, default-initialized, never written to
- `MostPowerfulEvolution` highlight kind: exists as enum variant, has display text, scoring, and run-end text, but no detection system
- `SpawnAdditionalBolt.source_chip` and `SpawnChainBolt.source_chip`: carried in messages but `spawn_additional_bolt` and `spawn_chain_bolt` systems do NOT insert `SpawnedByEvolution` on the spawned bolt entity
