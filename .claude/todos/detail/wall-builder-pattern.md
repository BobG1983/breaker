# Wall Builder Pattern

## Summary
Apply the typestate builder pattern to Wall entities, replacing manual tuple assembly.

## Context
The Bolt entity was migrated to a typestate builder pattern (`Bolt::builder()...build()/spawn()`). Wall entities still use manual component assembly. Consistency across entity types simplifies the codebase and enforces component completeness at compile time.

## Scope
- In: `Wall::builder()...build()/spawn()` builder, replace manual tuple assembly in spawn systems and test helpers
- Out: Cell builder (separate todo), rendering (placeholder rectangles for now)

## Dependencies
- Depends on: Bolt builder pattern being stable (it is), Breaker builder (establishes the pattern for non-bolt entities)
- Blocks: Rendering refactor (builders own visual setup)

## Notes
Follow the pattern from `bolt/builder/`. Walls are simpler than bolts/breakers — likely fewer typestate dimensions.

## Status
`[NEEDS DETAIL]` — Missing: typestate dimensions (position, orientation, thickness, wall type?), how layout data feeds into the builder, whether walls need a config resource or are fully layout-driven
