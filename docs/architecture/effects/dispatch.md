# Dispatch

Dispatch is **NOT** part of the effect domain. Each entity domain handles its own initialization and effect dispatch.

## Who Dispatches

- **Chip dispatch** (`chips/`) — reads ChipSelected messages, resolves On(target), pushes to BoundEffects, fires bare Do
- **Breaker init** (`breaker/`) — reads BreakerDefinition, resolves On(target), pushes to BoundEffects, fires bare Do
- **Cell init** (`cells/`) — reads CellDefinition (optional `effects` field, defaults to None), same pattern

## Dispatch Logic

All three follow the same pattern:

1. For each `RootEffect::On { target, then }` in the definition's effects list
2. Resolve `target` to concrete entity/entities
3. For each child in `then`:
   - **Bare `Do(effect)`** → `commands.fire_effect(target_entity, effect)` — fires immediately (passives like Piercing, DamageBoost)
   - **Non-Do** (When, Until, Once, On) → push to the target entity's BoundEffects

## Target Resolution at Dispatch

| Target | Resolves to |
|--------|------------|
| `Bolt` | Primary bolt entity. New bolts inherit BoundEffects if spawned with `inherit: true`. |
| `Breaker` | The breaker entity |
| `Cell` | No-op (cells may not exist at dispatch time) |
| `Wall` | No-op (walls may not exist at dispatch time) |

### All* Target Desugaring AT DISPATCH TIME ONLY

`AllBolts`, `AllCells`, `AllWalls` — target entities may not all exist at dispatch time (chip selection happens between nodes). These are **desugared** into deferred resolution:

`On(AllCells, children)` becomes `When(NodeStart, On(AllCells, permanent: true, children))`

This is pushed onto the primary breaker entity's BoundEffects. When the next node starts:
1. `NodeStart` fires (global sweep)
2. The `When` matches → inner `On(AllCells, permanent: true, children)` resolves to all cell entities (which now exist)

This `When` lives in the Breaker's BoundEffects permanently — it re-fires on every NodeStart for the rest of the run, ensuring new bolts/cells/walls in subsequent nodes also receive the effect.

Example — a chip targeting all cells:
```ron
// RON (what the chip author writes)
On(target: AllCells, then: [When(trigger: Died, then: [Do(Shockwave(...))])])

// Dispatch desugars this to:
When(trigger: NodeStart, then: [
    On(target: AllCells, then: [When(trigger: Died, then: [Do(Shockwave(...))])])
])
// ...and pushes it to the breaker's BoundEffects
```

## No Selected Trigger

There is no `Selected` trigger. The dispatch system fires bare Do children directly — no trigger machinery needed for passives.

## Command Extension Dispatch Model

walk_effects defers all effect execution and cross-entity mutation through `EffectCommandsExt` on Bevy `Commands`. Bridge systems are regular Bevy systems with `Query` + `Commands` parameters — no exclusive world access required.

Same-entity StagedEffects mutations (drain matching entries) happen inline during the walk. Cross-entity operations (Stamp/Transfer onto another entity) and effect execution (Fire/Reverse) are deferred via command extensions. Bevy applies commands at schedule flush points with exclusive `&mut World` access.

See [Commands](commands.md) for the EffectCommandsExt trait and each command's behavior.
