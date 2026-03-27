# Impacted(X)

**Scope**: Targeted (both participants)

"You were in an impact, and X was the other entity." Fires on BOTH participants in the collision.

Variants: `Impacted(Cell)`, `Impacted(Bolt)`, `Impacted(Wall)`, `Impacted(Breaker)`.

When a bolt hits a cell:
- The bolt gets `Impacted(Cell)` — "you were in an impact with a cell"
- The cell gets `Impacted(Bolt)` — "you were in an impact with a bolt"

See [Impact](impact.md) for the global counterpart.
