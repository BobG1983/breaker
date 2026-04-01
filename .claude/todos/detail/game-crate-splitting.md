# Game Crate Splitting

## Summary
Investigate splitting `breaker-game` monolith into sub-crates to improve compile times and organization.

## Context
`breaker-game` is a single crate containing all game domains (bolt, breaker, cells, chips, effect, run, fx, audio, ui, debug, input, wall). As the codebase grows, incremental compile times increase because any change recompiles the entire crate. Sub-crates would allow Cargo to skip recompilation of unchanged domains.

Potential splits discussed:
- `breaker-effects` — effect system, trigger bridges, dispatch
- `breaker-content` — chip definitions, cell types, RON data loading
- Others TBD based on investigation

## Scope
- In: Investigation of whether splitting is worth it (compile time measurements, dependency analysis), identification of logical split boundaries, migration plan if worthwhile
- Out: Actually performing the split (that would be its own implementation work per sub-crate)

## Dependencies
- Depends on: Nothing — this is investigation work
- Blocks: Nothing directly, but findings would inform future crate organization

## Notes
Key questions to answer:
1. What are current incremental compile times for common change patterns? (baseline measurement)
2. What are the dependency edges between domains? (which domains import from which)
3. Are there circular dependencies that would prevent clean splits?
4. What's the message/type boundary — do domains share types that would need to live in a shared crate?
5. Would the cargo workspace overhead (more crates = more linking) offset the incremental compile gains?
6. How would this interact with the existing `rantzsoft_*` crate boundaries?

This needs `/todo research` before any design work — specifically researcher-codebase to map cross-domain dependencies and researcher-system-dependencies to map system ordering constraints across domains.

## Status
`[NEEDS DETAIL]` — Needs investigation before any design. Missing: compile time baselines, cross-domain dependency map, circular dependency analysis, proposed split boundaries, cost/benefit assessment
