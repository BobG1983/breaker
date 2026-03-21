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
- **Per-effect layout** (updated 2026-03-21, supersedes per-consequence layout from 2026-03-16): `behaviors/` uses per-effect file organization in an `effects/` directory (NOT a sub-domain). Each effect file owns its observer and helpers. `behaviors/consequences/` and `ConsequenceFired` were DELETED in refactor/unify-behaviors. The unified `EffectFired { effect: TriggerChain, bolt: Option<Entity> }` observer event (behaviors/events.rs) replaces both ConsequenceFired and OverclockEffectFired. Current effects: shockwave.rs, life_lost.rs, time_penalty.rs, spawn_bolt.rs.
- **Bevy observers for intra-domain dispatch**: Consequence events use `#[derive(Event)]` + `commands.trigger()` + `app.add_observer()`. Messages remain required for inter-domain communication.
- **Consumer-owns message pattern** for consequence-to-target messages: RunLost, ApplyTimePenalty, SpawnAdditionalBolt all defined in consuming domain.
- interpolate/ domain: FixedFirst (restore_authoritative), FixedPostUpdate (store_authoritative), PostUpdate (interpolate_transform). Entities opt in via InterpolateTransform + PhysicsTranslation components.
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- lib.rs visibility: pub for app/game/shared/bolt/breaker/chips/input/run/behaviors (behaviors widened 2026-03-21 for ActiveChains in scenario runner), pub(crate) for remaining
- proptest dev-dependency present and used in shared/math.rs
- Physics domain reads other domains' components (acceptable per ECS convention)
- Physics owns collision detection + bolt reflection (collision response)
- Chip effect components owned by chips/, stamped by chips/effects/* observers, read by production systems via Option<&T> queries. No messages needed — normal ECS read-only queries. Observer dispatch via ChipEffectApplied event (intra-domain). Stacking via stack_u32/stack_f32 helpers in chips/effects/mod.rs.
- **behaviors/ domain unification** (2026-03-21, refactor/unify-behaviors): bolt/behaviors/ sub-domain DELETED and BoltBehaviorsPlugin REMOVED. Overclock evaluation engine merged into top-level behaviors/ domain. ActiveOverclocks→ActiveChains (behaviors/active.rs). OverclockEffectFired→EffectFired (behaviors/events.rs). OverclockTriggerKind→TriggerKind (behaviors/evaluate.rs). All bridge systems in behaviors/bridges.rs. behaviors/consequences/ DELETED; replaced by behaviors/effects/ (observers: shockwave, life_lost, time_penalty, spawn_bolt). ConsequenceFired GONE; EffectFired is now the unified dispatch event. ArmedTriggers component (pub(crate)) in behaviors/armed.rs. ActiveChains resource pub in behaviors/active.rs.
- **EntityScale pattern** (2026-03-20): shared/ passive component stamped by per-domain OnEnter systems (apply_entity_scale_to_breaker, apply_entity_scale_to_bolt) reading ActiveNodeLayout. Physics/visual systems consume via Option<&EntityScale> for backward compatibility. Mid-node spawns (spawn_additional_bolt) stamp EntityScale at spawn time from Option<Res<ActiveNodeLayout>>.
- Cross-domain ordering MUST use SystemSet enums, never bare fn refs
- Intra-domain ordering may use bare fn refs
- Config-to-entity materialization via init_*_params systems on OnEnter(Playing)

## Config-to-Entity Details (2026-03-12)
- breaker/components/ subfolder: core.rs, state.rs, movement.rs, dash.rs, bump.rs
- MaxReflectionAngle and MinAngleFromHorizontal defined in breaker/components/core.rs, sourced from BreakerConfig
- bolt/components.rs: BoltBaseSpeed, BoltMinSpeed, BoltMaxSpeed, BoltRadius, BoltSpawnOffsetY, BoltRespawnOffsetY, BoltInitialAngle
- cells/components.rs: CellDamageVisuals, CellWidth, CellHeight
- init_breaker_params + init_bolt_params: OnEnter(Playing), after spawn, guard via Without<sentinel>
- NOTE (2026-03-21): bolt/apply_bump_velocity DELETED — velocity scaling now via TriggerChain::SpeedBoost leaf in archetype RON, handled by handle_speed_boost observer in behaviors/effects/speed_boost.rs. BumpPerformed no longer carries a multiplier field.
- PhysicsConfig/PhysicsDefaults no longer exist (all fields moved to BreakerConfig)

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin)
- Tests are in-module `#[cfg(test)]` blocks
- Init system tests verify: all components inserted, values match config, skip guard works

## Doc Sync (2026-03-13)
All docs updated to match code reality. Key fixes: plugins.md, messages.md, ordering.md, content.md, standards.md, README.md, data.md.
