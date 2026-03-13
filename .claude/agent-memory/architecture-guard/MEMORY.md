# Architecture Guard Memory

## Project State
- Phase 0 scaffolding complete, reviewed 2025-03-10
- Phase 1 core mechanics implemented, reviewed 2025-03-10
- Main menu screen implemented, reviewed 2026-03-11
- Full audit completed 2026-03-11: clean, no critical violations
- Post-Phase1 additions audit 2026-03-11: BoltServing, hover_bolt, launch_bolt, BumpVisual, RunState, cleanup_entities<T> — all clean
- Config-to-entity extraction refactor audited 2026-03-12: PASS, no violations
- Full doc-vs-code audit 2026-03-13: 22 mismatches found and FIXED in docs (see below)
- Bevy 0.18.1, bevy_egui 0.39, edition 2024
- Single crate, plugin-per-domain, message-driven decoupling
- Also depends on: bevy_asset_loader 0.25, bevy_common_assets 0.15, iyes_progress 0.16
- Architecture docs in `docs/architecture/` (README, layout, messages, ordering, plugins, state, physics, content, standards, data)
- wall/ domain extracted from physics (2026-03-13 branch: refactor/extract-wall-domain)

## Key Patterns Confirmed
- Messages defined in sending domain's `messages.rs`, registered via `app.add_message::<T>()` in owning plugin
- `shared.rs` has passive types only: GameState, PlayingState, cleanup markers, playfield constants
- `game.rs` is the ONLY file that imports top-level plugin structs (sub-domain plugins are added by their parent)
- `screen/` owns state registration (init_state, add_sub_state) and cleanup systems
- **Nested sub-domains allowed** (added 2026-03-13): a domain may contain child sub-domains with their own plugin, components, and systems. Same canonical layout. Parent plugin adds child plugins. Max one level of nesting. Sub-domains may import parent's shared components. See `docs/architecture/layout.md`.
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- lib.rs visibility correct: pub for app/game/shared, pub(crate) for all domain modules
- proptest dev-dependency is present in Cargo.toml (planned, not yet used)
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

## Current Ordering Chain (verified 2026-03-13)
```
BreakerSystems::Move
  <- (hover_bolt, prepare_bolt_velocity) .after(BreakerSystems::Move)
    BoltSystems::PrepareVelocity
      <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
        <- bolt_breaker_collision .after(bolt_cell_collision)
          PhysicsSystems::BreakerCollision
            <- apply_bump_velocity .after(PhysicsSystems::BreakerCollision)
                                   .before(PhysicsSystems::BoltLost)
            <- grade_bump .after(update_bump)
                          .after(PhysicsSystems::BreakerCollision)
              <- (perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text) .after(grade_bump)
            <- bolt_lost .after(bolt_breaker_collision)
              PhysicsSystems::BoltLost
```

Breaker intra-domain: update_bump → move_breaker → update_breaker_state → grade_bump
trigger_bump_visual .after(update_bump)
Update schedule: animate_bump_visual, animate_tilt_visual

## Message Inventory
See [message-inventory.md](message-inventory.md) for full table.

Active messages (Phase 1, consumed in code):
- BoltHitBreaker: physics → breaker (grade_bump)
- BoltHitCell: physics → cells (handle_cell_hit)
- BoltLost: physics → bolt (spawn_bolt_lost_text)
- BumpPerformed: breaker → bolt (apply_bump_velocity), breaker (bump_feedback, perfect_bump_dash_cancel)
- BumpWhiffed: breaker → breaker (spawn_whiff_text)

Registered but no consumers yet: CellDestroyed, NodeCleared, UpgradeSelected, TimerExpired

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin)
- Tests are in-module `#[cfg(test)]` blocks
- Init system tests verify: all components inserted, values match config, skip guard works
- 230+ tests as of 2026-03-13

## Accepted Architectural Compromises
- Physics domain mutates bolt Transform + BoltVelocity for collision response (minimum necessary)
- bolt/apply_bump_velocity reads breaker entity components (BumpPerfectMultiplier, BumpWeakMultiplier) — read-only
- Screen domain seeds ALL domain configs during loading (centralized boot sequence)
- bolt/hover_bolt reads breaker Transform (read-only cross-domain query, acceptable ECS pattern)
- bolt/spawn_bolt reads BreakerConfig and RunState (read-only, config access for spawn positioning)
- physics/bolt_lost reads breaker Transform (read-only, for respawn position)
- physics/ccd.rs exists outside canonical layout (shared CCD math helpers for physics systems)

## Doc Sync (2026-03-13)
All docs updated to match code reality. Key fixes applied:
- plugins.md: added wall/ domain, marked stub domains with phase info, updated physics description
- messages.md: split into Active (Phase 1) and Registered (Phase 2+) tables, fixed all consumer lists
- ordering.md: complete chain with PhysicsSystems::BoltLost, .before() constraints, intra-domain note
- content.md: added "Not Yet Implemented" disclaimer
- standards.md: updated cleanup to generic pattern, boot sequence to actual, debug features to actual
- README.md: added "serialize" to features list
- data.md: generalized config resource examples

Previous mismatches (quadtree references, missing input domain) were already fixed in prior sessions.
