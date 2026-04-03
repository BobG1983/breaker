---
name: Wall construction sites
description: Complete inventory of Wall entity spawn sites, component sets, WallsSpawned message flow, and query sites — gathered for wall builder pattern work
type: project
---

Researched 2026-04-02. Report at `docs/todos/detail/wall-builder-pattern/research/wall-construction-sites.md`.

Key findings:
- 2 production spawn sites: `spawn_walls` (3 canonical walls) + `second_wind::fire` (1 bottom wall)
- 15+ test spawn locations, all inline — no builder, component sets vary widely
- `WallSize {}` is an empty marker; `Aabb2D` carries the actual extents
- `SecondWindWall` subset: missing `GameDrawLayer::Wall` vs canonical walls
- `WallsSpawned` message: 1 sender (`spawn_walls`), 1 receiver (`check_spawn_complete`), low coupling

**Why:** Wall builder pattern being implemented on `feature/breaker-builder-pattern` branch.
**How to apply:** When speccing the builder, account for the SecondWind wall divergence and the test helper component-set variations — the builder needs a headless/physics-only mode analogous to `Bolt::builder().headless()`.
