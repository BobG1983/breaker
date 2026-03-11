# Architecture Guard Memory

## Project State
- Phase 0 scaffolding complete, reviewed 2025-03-10
- Phase 1 core mechanics implemented, reviewed 2025-03-10
- Main menu screen implemented, reviewed 2026-03-11
- Full audit completed 2026-03-11: clean, no critical violations
- Post-Phase1 additions audit 2026-03-11: BoltServing, hover_bolt, launch_bolt, BumpVisual, RunState, cleanup_entities<T> — all clean
- Bevy 0.18.1, bevy_egui 0.39, edition 2024
- Single crate, plugin-per-domain, message-driven decoupling
- Also depends on: bevy_asset_loader 0.25, bevy_common_assets 0.15, iyes_progress 0.16

## Key Patterns Confirmed
- Messages defined in sending domain's `messages.rs`, registered via `app.add_message::<T>()` in owning plugin
- `shared.rs` has passive types only: GameState, PlayingState, cleanup markers, playfield constants
- `game.rs` is the ONLY file that imports all plugin structs
- `screen/` owns state registration (init_state, add_sub_state) and cleanup systems
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- All 8 messages from architecture table are implemented and registered
- lib.rs visibility correct: pub for app/game/shared, pub(crate) for all domain modules
- proptest dev-dependency is present in Cargo.toml
- Physics domain reads other domains' components (acceptable per ECS convention)
- Physics owns collision detection + bolt reflection (collision response)

## Phase 1 Boundary Violations — All RESOLVED
- V1: apply_bump_velocity in bolt domain reads BumpPerformed, mutates only BoltVelocity
- V2: physics writes BoltHitCell only, cells domain handles damage/despawn
- V3: enforce_min_angle is now a method on BoltVelocity in bolt/components.rs
- M1: CellDestroyed written by cells domain via handle_cell_hit
- M2: grade_bump reads BoltHitBreaker messages correctly
- O1: Cross-plugin physics chain ordering implemented

## Screen Domain — Clean (2026-03-11)
- All prior violations resolved
- DefaultsCollection now in screen/resources.rs (confirmed)
- apply_bump_velocity one-frame delay is expected message-driven behavior (confirmed)

## Physics Improvements (Phase 1 iteration)
- bolt_breaker_collision: side-hit vs top-hit via overlap depth comparison
- bolt_cell_collision: nearest-cell selection (min penetration), swept ray-AABB for tunneling
- bolt_lost: respawn straight up (vx=0, vy=min_speed) as penalty
- apply_bump_velocity: speed clamping via BoltConfig.max_speed after bump multiplier
- Note: MIN_PHYSICS_FPS constant in bolt_cell_collision should track FixedUpdate rate if it becomes configurable

## Message Inventory
See [message-inventory.md](message-inventory.md) for full table.

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin)
- Tests are in-module `#[cfg(test)]` blocks

## Accepted Architectural Compromises
- Cross-domain system fn refs in .before()/.after() ordering (no alternative in Bevy API)
- Physics domain mutates bolt Transform + BoltVelocity for collision response (minimum necessary)
- bolt/apply_bump_velocity reads BreakerConfig for bump multipliers (read-only, acceptable)
- Screen domain seeds ALL domain configs during loading (centralized boot sequence per ARCHITECTURE.md)
- bolt/hover_bolt reads breaker Transform (read-only cross-domain query, acceptable ECS pattern)
- bolt/spawn_bolt reads BreakerConfig and RunState (read-only, config access for spawn positioning)
- physics/bolt_lost reads breaker Transform (read-only, for respawn position)

## Open Issues (Phase 0, still valid)
- ARCHITECTURE.md file tree shows assets/ under src/ but actual is project root
- "upgrades" module uses generic term; TERMINOLOGY.md vs ARCHITECTURE.md contradiction
- ARCHITECTURE.md ordering section should explicitly document the cross-domain system ref carve-out
