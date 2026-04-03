# Cell Builder Pattern

## Summary
Apply the typestate builder pattern to Cell entities, replacing manual tuple assembly.

## Context
The Bolt entity was migrated to a typestate builder pattern (`Bolt::builder()...build()/spawn()`). Cell entities still use manual component assembly. Consistency across entity types simplifies the codebase and enforces component completeness at compile time.

## Scope
- In: `Cell::builder()...build()/spawn()` builder, replace manual tuple assembly in spawn systems and test helpers, handle all cell variants
- Out: Wall builder (separate todo), rendering (placeholder rectangles for now)

## Dependencies
- Depends on: Bolt builder pattern being stable (it is), Breaker builder (establishes the pattern for non-bolt entities)
- Blocks: Rendering refactor (builders own visual setup)

## Notes
Follow the pattern from `bolt/builder/`. Cells are more complex than walls — multiple cell types (standard, armored, explosive, etc.), health, color, dimensions. Need to decide whether variants are typestate dimensions or runtime config.

## Status
`[NEEDS DETAIL]` — Missing: typestate dimensions (position, cell type, health, dimensions, color?), whether cell variants are typestate or runtime, how RON layout data feeds into the builder, whether the builder handles all cell types or just the base with variant-specific extensions
