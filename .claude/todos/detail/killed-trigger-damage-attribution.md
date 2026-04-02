# Killed Trigger + Damage Source Attribution

## Summary
Add a Killed trigger (fires on dying entity with killer in TriggerContext) and the damage attribution infrastructure it requires.

## Context
Died fires on dying entities with no context — intentionally. This enables "mark and reward" (Powder Keg, Death Lightning): a bolt marks a cell on impact, the effect fires when it dies from any cause.

Killed is a separate trigger that carries the killer in context. This enables two additional patterns that Died can't support:

### Three death-reaction patterns

| Trigger | Context | Pattern | Example |
|---------|---------|---------|---------|
| Died | none | **Mark and reward** | Powder Keg: bolt marks cell, cell dies from any cause, effect fires |
| Killed | killer | **Kill reward** | Chip: "when I kill a cell, boost my speed" — reward goes to the killer bolt |
| Killed | killer | **Cell revenge** | Cell type: "when destroyed, cripple the bolt that killed me" |

Died can't carry killer context without breaking mark-and-reward (the bolt that marked ≠ the bolt that killed).

### Cell revenge (the primary motivation)

A "Volatile" cell type that punishes the killer:
```ron
On(target: Cell, then: [
    When(trigger: Killed, then: [
        On(target: Bolt, then: [
            Do(SpeedBoost(multiplier: 0.3))
        ])
    ])
])
```
"When I'm destroyed, cripple the bolt that killed me to 30% speed." Creates tactical choices — which bolt do you sacrifice to break volatile cells? Cell type definitions already have `effects: Vec<RootEffect>`, so this just works once Killed exists.

### Kill reward

A chip that rewards finishing blows:
```ron
On(target: Bolt, then: [
    When(trigger: Killed, then: [
        Do(SpeedBoost(multiplier: 1.3))
    ])
])
```
Wait — Killed fires on the dying cell, not the killer bolt. For kill reward, the bolt needs to hear about the kill. This requires either:
- Killed also fires on the killer entity (targeted, not global)
- Or a chip uses the Impacted→On(Cell)→Killed pattern to stage the trigger on the cell, then retarget back to the bolt via context

The Impacted→Killed pattern:
```ron
On(target: Bolt, then: [
    When(trigger: Impacted(Cell), then: [
        On(target: Cell, then: [
            When(trigger: Killed, then: [
                On(target: Bolt, then: [
                    Do(SpeedBoost(multiplier: 1.3))
                ])
            ])
        ])
    ])
])
```
On(Bolt) inside Killed resolves to the killer bolt via context. The marking bolt and the killer may be different — the reward goes to whoever dealt the killing blow, not whoever marked the cell.

## Scope
- In:
  - `Trigger::Killed` variant
  - `DamageCell.source_entity: Option<Entity>` field
  - `LastDamageSource(Option<Entity>)` component on cells, updated on every DamageCell
  - `bridge_killed` system — fires Killed on dying entity, reads LastDamageSource to populate TriggerContext
  - `FireEffectCommand` carries `TriggerContext` so effect-sourced damage traces back to the originating entity
  - Tests for attribution and context correctness
- Out:
  - Kill trigger (Death already fires globally with dying entity as context)
  - Chain attribution (deferred — start with last-hitter only)

## Dependencies
- Depends on: TriggerContext infrastructure (done)
- Blocks: Cell revenge mechanics, kill-reward chips

## Notes
- `DamageCell.source_entity` for direct bolt hits is straightforward — bolt_cell_collision has the bolt entity
- Effect-sourced damage (Explode, ChainLightning) needs TriggerContext threaded through FireEffectCommand → fire() to populate source_entity. Currently fire() only receives the entity it fires ON.
- Chain attribution (effect kills cell A, which triggers effect that kills cell B — who killed B?) is deferred. Last-hitter is sufficient.

## Status
`[NEEDS DETAIL]` — Missing: FireEffectCommand context threading design, DamageCell.source_entity test impact assessment, Explode/ChainLightning source_entity policy (populate or None)
