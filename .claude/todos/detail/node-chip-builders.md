# Node and Chip Builder Patterns

## Summary
Create typestate builders for nodes and chips, following the same pattern established by bolt and breaker builders. Also create template.ron reference files for wall, cell, and node definitions.

## Context
The project is establishing a typestate builder pattern for all entity types. Bolt builder is done, breaker builder is in progress (todo #1). Wall (#3) and cell (#4) builders come next. After those are complete, nodes and chips should follow the same pattern.

Additionally, template.ron files (annotated RON references that aren't loaded by the game) should be created for all entity types that don't already have them. Bolt, breaker, and chip templates already exist. Still needed: wall.example.ron, cell.example.ron, node.example.ron.

## Scope
- In: Node builder, chip builder, wall/cell/node template.ron files
- Out: Wall builder (todo #3), cell builder (todo #4) — those are separate items

## Dependencies
- Depends on: Wall builder (#3), Cell builder (#4) — those establish patterns this builds on
- Depends on: Breaker builder (#1) — the reference pattern

## Notes
- Template files use .example.ron extension so registries don't load them
- Each template documents every field, what it controls, and which fields are required vs defaulted
- Node builder may have different typestate dimensions than entity builders (nodes define layouts, not physics)
- Chip builder may be simpler since ChipDefinition is constructed from ChipTemplate expansion, not directly

## Status
`[NEEDS DETAIL]` — needs investigation into what node and chip builders would look like (typestate dimensions, required vs optional data)
