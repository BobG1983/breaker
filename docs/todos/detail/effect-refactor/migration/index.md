# Migration

Everything needed to move from the old effect system to the new one.

- [implementation/](implementation/index.md) — **Start here** — ordered implementation waves (21 waves, TDD gates)
- [folder-structure.md](folder-structure.md) — Target directory tree for src/effect/
- [rust-type-swaps.md](rust-type-swaps.md) — Old types → new types replacement tables
- [new-dependencies.md](new-dependencies.md) — New crate dependencies required
- [finalized-assets/](finalized-assets/) — All asset RON files rewritten in the new syntax
- [new-effect-implementations/](new-effect-implementations/index.md) — Per-effect behavioral specs
- [new-trigger-implementations/](new-trigger-implementations/index.md) — Per-trigger bridge specs
- [plugin-wiring/](plugin-wiring/index.md) — EffectPlugin, system sets, ordering
