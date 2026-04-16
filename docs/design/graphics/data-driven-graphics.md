# Data-Driven Graphics

**Status: NOT IMPLEMENTED.** The types described here (`CellShape`, `EntityVisualConfig`, `AttachVisuals`, etc.) do not exist in code. This is a forward-looking spec for Phase 5 work.

The content has been filed as part of the Phase 5 todos. See:
- `docs/todos/detail/phase-5i-cell-visuals.md` — cell visual composition
- `docs/todos/detail/phase-5g-bolt-visuals.md` — bolt visual composition
- `docs/todos/detail/phase-5h-breaker-visuals.md` — breaker visual composition

## Design Principle (retained)

Every visual aspect of a game entity should be definable through **enum composition in RON files**. A new cell type, breaker archetype, or effect variant should be creatable by combining existing visual building blocks — shape, color, aura, trail, state handling — without writing new rendering code.

New rendering code is only needed when a genuinely new visual primitive is required (a new shader, a new particle behavior, a new shape type). But combinations of existing primitives are pure data.
