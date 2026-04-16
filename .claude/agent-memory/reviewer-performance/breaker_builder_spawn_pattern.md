---
name: BreakerBuilder spawn pattern
description: Builder-at-spawn, ~30 components in one spawn(), Vec<RootNode> clone — all confirmed acceptable
type: project
---

`BreakerBuilder::spawn()` in `breaker-game/src/breaker/builder/core.rs`:

- **Spawn frequency**: called at node start or run start, never per-frame. Not a hot path.
- **Component count**: `build_core()` produces ~30 components in a single `spawn()` call. Single-call spawn produces one archetype lookup, which is strictly better than the prior 4-system pipeline that inserted components incrementally (each incremental insert is an archetype move).
- **Archetype split (Primary vs Extra)**: Primary adds `PrimaryBreaker + CleanupOnRunEnd`; Extra adds `ExtraBreaker + CleanupOnNodeExit`. These are genuinely different entity lifetimes, not accidental fragmentation — mirrors the bolt builder pattern. At 1–2 breakers total this is completely negligible.
- **Rendered vs Headless**: Two additional archetypes from `Mesh2d + MeshMaterial2d` presence. Again, negligible at 1 entity; headless path is tests-only.
- **`Vec<RootNode>` clone in `spawn()`**: The clone happens before `self` is consumed by `build()` because the borrow checker requires it — `self.optional.effects` is inside `self` which moves into `build()`. The clone is at spawn time (not per-frame) and the Vec contains a small RON-defined chip effect tree (typically 0–10 nodes). Fully acceptable.
- **No systems added**: The builder is a pure type. Zero scheduling concerns.

**Why:** Breaker is always 1 entity. All performance concerns here are academic until entity count grows orders of magnitude.

**How to apply:** Do not flag the builder's clone or component count as concerns. They are spawn-time, not frame-time, and they affect exactly 1 entity.
