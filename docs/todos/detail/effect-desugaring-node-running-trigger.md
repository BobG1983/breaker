# Effect Desugaring: NodeRunning Trigger + Unique Stamping

## Summary
Add a `NodeRunning` trigger and unique stamping to fix late-spawned entities missing `AllBolts`/`AllCells`/`AllWalls` effects and duplicate stamping on second entity spawns.

## Context
When `dispatch_breaker_effects` or `dispatch_chip_effects` encounters `Target::AllBolts`, it resolves to all existing bolt entities at dispatch time and stamps `BoundEffects` on each, desugared to `Once(When(NodeStarted, ...))`.

Two issues:
1. **Late-spawned entities miss effects.** Bolts spawned mid-node (by SpawnBolts, MirrorProtocol, etc.) weren't present at dispatch and never get AllBolts effects. `inherit: true` covers bolt-from-bolt, but not bolts spawned by breaker effects.
2. **Duplicate stamping.** If a second breaker spawns with the same definition, it re-stamps effects on all existing bolts — duplicating what the first breaker already applied.

## Scope
- In: `NodeRunning` trigger, trigger bridge systems for Added<Bolt>/Added<Cell>/Added<Wall>, change AllBolts/AllCells/AllWalls desugaring, `source_id` field on BoundEffects, dedup check in push_bound_effects, test updates
- Out: Co-op breaker spawning (future), inherit refactor

## Dependencies
- Depends on: Nothing specific, but complex — touches dispatch, triggers, bound effects
- Blocks: Future co-op / breaker clone mechanics

## Notes
**NodeRunning trigger**: fires once on node Playing state transition (same as NodeStarted), plus again when a new entity of the target type spawns mid-node (three systems: Added<Bolt>, Added<Cell>, Added<Wall>).

**Unique stamping**: Each BoundEffects entry carries `source_id: EffectSource{EntityArchetypeId, EffectIndex}`. Before stamping, check if target already has a BoundEffects entry with same source_id — skip if so.

Work required:
1. Add `NodeRunning` to Trigger enum
2. Add trigger bridge systems for entity spawn detection
3. Change AllBolts/AllCells/AllWalls desugaring to use NodeRunning
4. Add source_id field to BoundEffects entries
5. Add dedup check in push_bound_effects command
6. Update tests
7. Update scenario runner if it inspects trigger types

## Status
`[NEEDS DETAIL]` — Missing: how does NodeRunning interact with `inherit: true` on effect-spawned bolts (does inherit become redundant for AllBolts effects?), should NodeRunning also fire for entities spawned by scenario runner setup, what's the exact EntityArchetypeId structure (is it the definition hash? entity ID? something else?), ordering: does the Added<Bolt> bridge fire before or after the bolt's first physics tick?
