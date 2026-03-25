---
name: damage-attribution-flow
description: End-to-end damage attribution pipeline from ActiveChains through EffectFired to DamageCell to MostPowerfulEvolution highlight -- fully wired as of b9a5fb4
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
- `handle_shockwave` -> stores into `ShockwaveDamage.source_chip`
- `handle_spawn_bolt` -> writes `SpawnAdditionalBolt { source_chip }`
- `handle_multi_bolt` -> writes N x `SpawnAdditionalBolt { source_chip }`
- `handle_chain_bolt` -> writes `SpawnChainBolt { source_chip }`

## Bolt spawn systems insert SpawnedByEvolution
- `spawn_additional_bolt`: if `msg.source_chip` is `Some`, inserts `SpawnedByEvolution(name)` on the new bolt entity
- `spawn_chain_bolt`: if `msg.source_chip` is `Some`, inserts `SpawnedByEvolution(name)` on the new bolt entity

## Two DamageCell producers
1. **bolt_cell_collision** -- reads `Option<&SpawnedByEvolution>` from `CollisionQueryBolt`, writes `source_chip: spawned_by_evo.map(|s| s.0.clone())`
2. **shockwave_collision** -- writes `source_chip: dmg.source_chip.clone()` (from `ShockwaveDamage`)

## Two DamageCell consumers
1. **handle_cell_hit** -- applies damage, may despawn cell + write `CellDestroyed`. Ignores `source_chip` (not needed here).
2. **track_evolution_damage** -- accumulates `damage` into `HighlightTracker.evolution_damage[source_chip]` when `source_chip` is `Some`.

## Run-end detection
- `detect_most_powerful_evolution` runs on `OnEnter(GameState::RunEnd)`, finds max-damage chip in `evolution_damage`, pushes `RunHighlight { kind: MostPowerfulEvolution, value, detail: Some(name) }` to `RunStats.highlights`.

## Cross-node persistence
- `evolution_damage` is a cross-node field in `HighlightTracker` -- NOT cleared by `reset_highlight_tracker`.

## Design note: CellDestroyed has no attribution
`CellDestroyed { was_required_to_clear }` carries no chip info. This is fine because damage tracking reads `DamageCell` directly, not `CellDestroyed`. If future features need "which chip killed this cell", the field would need adding.
