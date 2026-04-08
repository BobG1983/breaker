# New Cell Modifiers

## Summary
Implement 7 new cell modifiers: Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal. Each adds unique behavior to standard cells via the CellBehavior enum and builder API.

## Context
The cell builder todo establishes the modifier pattern with existing modifiers (Locked, Regen, Shielded). This todo adds the new modifiers designed in [cell-modifiers.md](cell-builder-pattern/cell-modifiers.md). Each modifier gets its own `cells/behaviors/<name>/` folder with components and systems.

Modifiers are composable — a cell can have multiple modifiers (e.g., Armored + Volatile, Phantom + Sequence). All combos are valid.

## Scope
- In: 7 new CellBehavior enum variants (Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal)
- In: Components and systems for each modifier in `cells/behaviors/<name>/`
- In: Builder sugar for each modifier (`.volatile()`, `.armored(value)`, etc.)
- In: Modifier combination interactions
- Out: Cell builder typestate (done in builder todo)
- Out: Toughness / HP scaling (separate todo)
- Out: Visual effects for modifiers (Phase 5)
- Out: Node sequencing / skeleton integration (separate todo)

## Modifier Summary

| Modifier | Tier | Key mechanic |
|----------|------|-------------|
| Volatile | 1 | AoE explosion on death, chain reactions |
| Sequence | 3 | Numbered group, must clear in order, out-of-order repairs to full HP |
| Survival | 4 | Bolt-immune turret, pattern-based attacks, self-destruct timer, bump-vulnerable |
| Armored | 5 | Directional weak point (back), piercing interaction |
| Phantom | 6 | Solid/ghost phase cycling (~3s), variable starting phase |
| Magnetic | 7 | Pulls bolt within radius, gravity assist for experts |
| Portal | 5 (volatile) | On death spawns portal, bolt enters sub-level, complex lifecycle |

Full designs in [cell-modifiers.md](cell-builder-pattern/cell-modifiers.md).

## Dependencies
- Depends on: Cell builder pattern (provides builder API and behavior folder structure)
- Depends on: Toughness + HP scaling (for appropriate HP values) — soft dependency, can use `.hp(value)` initially
- Blocks: Phase 5 modifier visuals

## Notes
- Per-modifier component definitions, system designs, and interaction details still need fleshing out
- Survival attack pattern catalog needs design
- Phantom phase timing and telegraph specifics need design
- Magnetic field physics (force curve, max pull) need design
- Portal is the most complex — sub-level generation, transition state management, entity lifecycle
- Consider implementing in order of tier introduction (Volatile first, Portal last)

## Status
`[NEEDS DETAIL]` — modifier catalog designed at high level, per-modifier component/system detail needed
