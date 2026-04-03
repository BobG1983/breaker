# Rendering Refactor

## Summary
Replace placeholder rectangle rendering with a full graphics system — custom materials, shaders, particles, VFX, screen effects, and entity visuals.

## Context
The game currently renders all entities as colored rectangles using basic `MeshMaterial2d<ColorMaterial>`. A comprehensive rendering architecture has been designed and documented (see the `rendering-refactor/` directory alongside this file for the full design).

## Scope
See the detailed design docs in `rendering-refactor/`:
- [index.md](rendering-refactor/index.md) — overview and roadmap
- [entity_visuals.md](rendering-refactor/entity_visuals.md) — bolt, breaker, cell, wall visuals
- [materials.md](rendering-refactor/materials.md) — custom material system
- [shaders.md](rendering-refactor/shaders.md) — shader architecture
- [particles.md](rendering-refactor/particles.md) — particle system design
- [screen_effects.md](rendering-refactor/screen_effects.md) — fullscreen post-processing
- [composition.md](rendering-refactor/composition.md) — render layer composition
- [scheduling.md](rendering-refactor/scheduling.md) — render system scheduling
- [module-map.md](rendering-refactor/module-map.md) — module structure
- [types.md](rendering-refactor/types.md) — type definitions
- [research/](rendering-refactor/research/) — spike research (trails, auras, GPU particles, shaders, etc.)

## Dependencies
- Depends on: Builder patterns for all entity types (builders own visual setup)
- Blocks: Bolt birthing animation (birthing will use a spawning animation from this system)

## Notes
This is a large, multi-phase effort. The design docs are extensive. Will need to be broken into smaller implementation tasks when ready to start.

## Status
`[NEEDS DETAIL]` — Design docs exist but need to be reviewed for currency against the current codebase state. May need updating after builder migrations. Implementation plan (phases, ordering, dependencies between rendering subsystems) not yet created.
