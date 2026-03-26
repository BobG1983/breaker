---
name: damage-attribution-flow
description: End-to-end damage attribution pipeline from ActiveEffects through typed events to DamageCell to MostPowerfulEvolution highlight -- fully wired as of b9a5fb4; updated for C7-R (2026-03-25)
type: reference
---

# Damage Attribution Flow Map

## ActiveEffects tuple structure
`ActiveEffects(Vec<(Option<String>, EffectNode)>)` -- chip_name is `None` for archetype/breaker chains, `Some(name)` for chip/evolution chains. (Was `ActiveChains(Vec<(Option<String>, TriggerChain)>)` before C7-R.)

## Two populations
1. **Breaker init** (`init_breaker`): pushes with `(None, node)` -- no chip attribution (was `init_archetype` before C7-R)
2. **Chip effect dispatch** (`dispatch_chip_effects`): pushes with `(Some(event.chip_name.clone()), node)` -- has chip name (was `handle_overclock` before C7-R)

## Bridge systems thread chip_name into typed event source_chip
All bridge systems in effect/triggers/ (bridge_bump, bridge_cell_impact, bridge_breaker_impact, bridge_wall_impact, bridge_bolt_lost, bridge_bump_whiff, bridge_cell_destroyed) iterate `active.0` and pass `chip_name.clone()` as `source_chip` on every typed event they fire. (Was EffectFired.source_chip before C7-R — EffectFired deleted in C7-R.)

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
