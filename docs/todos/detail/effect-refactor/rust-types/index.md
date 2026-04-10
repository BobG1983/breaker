# Rust Types

Rust type definitions representing the RON syntax.

- [root-node.md](root-node.md) — Top-level enum: Stamp or Spawn
- [tree.md](tree.md) — Recursive effect tree enum
- [scoped-tree.md](scoped-tree.md) — Restricted tree inside During/Until
- [terminal.md](terminal.md) — Leaf operations (Fire or Route)
- [scoped-terminal.md](scoped-terminal.md) — Restricted terminal inside During/Until
- [trigger-context.md](trigger-context.md) — Trigger event context for On resolution
- [fireable.md](fireable.md) — Fireable trait: the fire contract for all effects
- [reversible.md](reversible.md) — Reversible trait: the reverse contract for reversible effects
- [enums/](enums/index.md) — All enum types
- [configs/](configs/index.md) — All config structs
- [type-migration.md](type-migration.md) — Where to replace RootEffect with RootNode
