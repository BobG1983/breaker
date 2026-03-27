# Impact(X)

**Scope**: Global

"There was an impact involving an X." Sweeps all entities with BoundEffects.

Variants: `Impact(Cell)`, `Impact(Bolt)`, `Impact(Wall)`, `Impact(Breaker)`.

When a bolt hits a cell, both `Impact(Cell)` and `Impact(Bolt)` fire as global triggers. See [Impacted](impacted.md) for the targeted counterpart.

A single collision fires FOUR triggers total:
1. `Impact(Cell)` — global
2. `Impact(Bolt)` — global
3. `Impacted(Cell)` — targeted on the bolt
4. `Impacted(Bolt)` — targeted on the cell

Any entity type can be on either side as mechanics expand (moving cells, breaker collisions, etc.).
