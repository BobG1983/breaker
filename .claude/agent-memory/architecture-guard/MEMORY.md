# Architecture Guard Memory

## Project State
- Phase 0 scaffolding complete, reviewed 2025-03-10
- Phase 1 core mechanics implemented, reviewed 2025-03-10
- Main menu screen implemented, reviewed 2026-03-11
- Full audit completed 2026-03-11: clean, no critical violations
- Post-Phase1 additions audit 2026-03-11: BoltServing, hover_bolt, launch_bolt, BumpVisual, RunState, cleanup_entities<T> — all clean
- Config-to-entity extraction refactor audited 2026-03-12: PASS, no violations
- Full doc-vs-code audit 2026-03-12: 8 doc-code mismatches, 2 stale code items, 2 minor issues, 0 critical violations
- Bevy 0.18.1, bevy_egui 0.39, edition 2024
- Single crate, plugin-per-domain, message-driven decoupling
- Also depends on: bevy_asset_loader 0.25, bevy_common_assets 0.15, iyes_progress 0.16
- Architecture docs in `docs/architecture/` (README, layout, messages, ordering, plugins, state, physics, content, standards, data)

## Key Patterns Confirmed
- Messages defined in sending domain's `messages.rs`, registered via `app.add_message::<T>()` in owning plugin
- `shared.rs` has passive types only: GameState, PlayingState, cleanup markers, playfield constants
- `game.rs` is the ONLY file that imports all plugin structs
- `screen/` owns state registration (init_state, add_sub_state) and cleanup systems
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- lib.rs visibility correct: pub for app/game/shared, pub(crate) for all domain modules
- proptest dev-dependency is present in Cargo.toml
- Physics domain reads other domains' components (acceptable per ECS convention)
- Physics owns collision detection + bolt reflection (collision response)
- Cross-domain ordering MUST use SystemSet enums, never bare fn refs (docs/architecture/ordering.md)
- Intra-domain ordering may use bare fn refs
- Config-to-entity materialization via init_*_params systems on OnEnter(Playing) — canonical pattern

## Config-to-Entity Extraction (2026-03-12)
- breaker/components/ subfolder: core.rs, state.rs, movement.rs, dash.rs, bump.rs
- MaxReflectionAngle and MinAngleFromHorizontal defined in breaker/components/core.rs, sourced from BreakerConfig
- bolt/components.rs: BoltBaseSpeed, BoltMinSpeed, BoltMaxSpeed, BoltRadius, BoltSpawnOffsetY, BoltRespawnOffsetY, BoltInitialAngle
- cells/components.rs: CellDamageVisuals, CellWidth, CellHeight
- init_breaker_params + init_bolt_params: OnEnter(Playing), after spawn, guard via Without<sentinel>
- Systems now read entity components instead of Res<Config> for gameplay params
- bump_visual.rs reads BumpVisualParams from entity (fully extracted)
- bolt/apply_bump_velocity reads BumpPerfectMultiplier/BumpWeakMultiplier from breaker entity (read-only)
- PhysicsConfig/PhysicsDefaults no longer exist (all fields moved to BreakerConfig)

## Current Ordering Chain (verified 2026-03-12)
```
BreakerSystems::Move
  <- (hover_bolt, prepare_bolt_velocity) .after(BreakerSystems::Move)
    BoltSystems::PrepareVelocity
      <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
        <- bolt_breaker_collision .after(bolt_cell_collision)
          PhysicsSystems::BreakerCollision
            <- apply_bump_velocity .after(PhysicsSystems::BreakerCollision)
            <- grade_bump .after(PhysicsSystems::BreakerCollision)
            <- bolt_lost .after(bolt_breaker_collision)
```

## Message Inventory
See [message-inventory.md](message-inventory.md) for full table.

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin)
- Tests are in-module `#[cfg(test)]` blocks
- Init system tests verify: all components inserted, values match config, skip guard works

## Accepted Architectural Compromises
- Physics domain mutates bolt Transform + BoltVelocity for collision response (minimum necessary)
- bolt/apply_bump_velocity reads breaker entity components (BumpPerfectMultiplier, BumpWeakMultiplier) — read-only
- Screen domain seeds ALL domain configs during loading (centralized boot sequence)
- bolt/hover_bolt reads breaker Transform (read-only cross-domain query, acceptable ECS pattern)
- bolt/spawn_bolt reads BreakerConfig and RunState (read-only, config access for spawn positioning)
- physics/bolt_lost reads breaker Transform (read-only, for respawn position)
- physics/ccd.rs exists outside canonical layout (shared CCD math helpers for physics systems)

## Doc-Code Mismatches (found 2026-03-12, not yet fixed in docs)
- physics.md describes quadtree but code uses CCD ray-casting (no quadtree exists)
- plugins.md, README.md, standards.md reference quadtree — stale
- plugins.md and CLAUDE.md omit input/ domain
- messages.md missing BumpWhiffed; BoltHitBreaker consumers missing breaker; BoltLost consumers missing bolt
- ordering.md chain missing grade_bump.after(PhysicsSystems::BreakerCollision)
- physics/resources.rs is empty file (should be deleted per layout.md rules)
