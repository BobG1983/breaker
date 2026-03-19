---
name: Key Patterns
description: Confirmed architectural patterns, conventions, and domain ownership rules
type: reference
---

## Key Patterns Confirmed
- Messages defined in sending domain's `messages.rs`, registered via `app.add_message::<T>()` in owning plugin
- `shared/` has passive types only: GameState, PlayingState, cleanup markers, playfield constants, shared math helpers (shared/math.rs)
- `game.rs` is the ONLY file that imports top-level plugin structs (sub-domain plugins are added by their parent)
- `screen/` owns state registration (init_state, add_sub_state) and cleanup systems
- `screen/` has six sub-domains: `loading/`, `main_menu/`, `run_end/`, `run_setup/`, `pause_menu/`, `upgrade_select/`
- `loading/` cross-references `main_menu::MainMenuDefaults` for config seeding — acceptable sibling import
- **Nested sub-domains allowed** (added 2026-03-13): a domain may contain child sub-domains with their own plugin, components, and systems. Same canonical layout. Parent plugin adds child plugins. Max one level of nesting.
- **Per-consequence layout** (updated 2026-03-16): `behaviors/` uses per-consequence file organization in a `consequences/` directory grouping (NOT a sub-domain). Each consequence file owns its Components, observer, and helpers. Generic `ConsequenceFired(Consequence)` event replaces per-consequence events.
- **Bevy observers for intra-domain dispatch**: Consequence events use `#[derive(Event)]` + `commands.trigger()` + `app.add_observer()`. Messages remain required for inter-domain communication.
- **Consumer-owns message pattern** for consequence-to-target messages: RunLost, ApplyTimePenalty, SpawnAdditionalBolt all defined in consuming domain.
- interpolate/ domain: FixedFirst (restore_authoritative), FixedPostUpdate (store_authoritative), PostUpdate (interpolate_transform). Entities opt in via InterpolateTransform + PhysicsTranslation components.
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- lib.rs visibility: pub for app/game/shared/bolt/breaker/input/run (widened for scenario runner), pub(crate) for remaining
- proptest dev-dependency present and used in shared/math.rs
- Physics domain reads other domains' components (acceptable per ECS convention)
- Physics owns collision detection + bolt reflection (collision response)
- Cross-domain ordering MUST use SystemSet enums, never bare fn refs
- Intra-domain ordering may use bare fn refs
- Config-to-entity materialization via init_*_params systems on OnEnter(Playing)

## Config-to-Entity Details (2026-03-12)
- breaker/components/ subfolder: core.rs, state.rs, movement.rs, dash.rs, bump.rs
- MaxReflectionAngle and MinAngleFromHorizontal defined in breaker/components/core.rs, sourced from BreakerConfig
- bolt/components.rs: BoltBaseSpeed, BoltMinSpeed, BoltMaxSpeed, BoltRadius, BoltSpawnOffsetY, BoltRespawnOffsetY, BoltInitialAngle
- cells/components.rs: CellDamageVisuals, CellWidth, CellHeight
- init_breaker_params + init_bolt_params: OnEnter(Playing), after spawn, guard via Without<sentinel>
- bolt/apply_bump_velocity reads multiplier from BumpPerformed message
- PhysicsConfig/PhysicsDefaults no longer exist (all fields moved to BreakerConfig)

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin)
- Tests are in-module `#[cfg(test)]` blocks
- Init system tests verify: all components inserted, values match config, skip guard works

## Doc Sync (2026-03-13)
All docs updated to match code reality. Key fixes: plugins.md, messages.md, ordering.md, content.md, standards.md, README.md, data.md.
