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
- `screen/` has six sub-domains: `loading/`, `main_menu/`, `run_end/`, `run_setup/`, `pause_menu/`, `chip_select/` (was `upgrade_select/` — renamed; also has a `systems/` module for shared cleanup)
- `loading/` cross-references `main_menu::MainMenuDefaults` for config seeding — acceptable sibling import
- **Nested sub-domains allowed** (added 2026-03-13): a domain may contain child sub-domains with their own plugin, components, and systems. Same canonical layout. Parent plugin adds child plugins. Max one level of nesting.
- **Per-effect layout** (updated 2026-03-24): `effect/` domain (renamed from `behaviors/` in B12) uses per-effect file organization in an `effects/` directory (NOT a sub-domain). Each effect file owns its observer, components, and typed event. Current triggered effects (13): shockwave.rs, life_lost.rs, time_penalty.rs, spawn_bolt.rs, speed_boost.rs, chain_bolt.rs, multi_bolt.rs, shield.rs, chain_lightning.rs, spawn_phantom.rs, piercing_beam.rs, gravity_well.rs, second_wind.rs. B12c migrated from catchall `EffectFired` to per-effect typed events (ShockwaveFired, LoseLifeFired, etc.). Passive chip effects still in `chips/effects/` (10 handlers). typed_events.rs is transitional — event types should co-locate with handlers.
- **Bevy observers for intra-domain dispatch**: Consequence events use `#[derive(Event)]` + `commands.trigger()` + `app.add_observer()`. Messages remain required for inter-domain communication.
- **Consumer-owns message pattern** for consequence-to-target messages: RunLost, ApplyTimePenalty, SpawnAdditionalBolt all defined in consuming domain.
- **rantzsoft_spatial2d domain** (2026-03-23): Canonical position is `Position2D` (from rantzsoft_spatial2d). `RantzSpatial2dPlugin<GameDrawLayer>` registered in game.rs. Systems: FixedFirst (save_previous_positions), RunFixedMainLoop/AfterFixedMainLoop (propagate_position, propagate_rotation, propagate_scale chained). Entities opt in via `Spatial2D` marker (which #[require]s Position2D, Rotation2D, Scale2D, PreviousPosition, PreviousRotation, PositionPropagation, RotationPropagation, ScalePropagation). Add `InterpolateTransform2D` for interpolated rendering. Add `GameDrawLayer` variant for Z ordering. `GameDrawLayer` defined in shared/draw_layer.rs (passive type).
- ~~interpolate/ domain~~: DELETED as of 2026-03-24 (spatial/physics extraction). InterpolatePlugin and PhysicsPlugin (game domain) fully removed from game.rs. Old pattern: FixedFirst (restore_authoritative), FixedPostUpdate (store_authoritative), PostUpdate (interpolate_transform) with InterpolateTransform + PhysicsTranslation components. Replaced by rantzsoft_spatial2d pipeline: see rantzsoft_spatial2d domain entry in this file.
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- lib.rs visibility: pub for app/game/shared/bolt/breaker/chips/input/run/effect (effect domain widened — was behaviors — for ActiveEffects in scenario runner), pub(crate) for remaining
- proptest dev-dependency present and used in shared/math.rs
- Physics domain reads other domains' components (acceptable per ECS convention)
- Physics owns collision detection + bolt reflection (collision response)
- Chip effect components owned by chips/, stamped by chips/effects/* observers, read by production systems via Option<&T> queries. No messages needed — normal ECS read-only queries. Observer dispatch via ChipEffectApplied event (intra-domain). Stacking via stack_u32/stack_f32 helpers in chips/effects/mod.rs.
- **B1-B3 TriggerChain flattening** (2026-03-24): ChipEffect enum removed. All chip effects now expressed as TriggerChain variants. apply_chip_effect dispatches three ways: OnSelected → fires ChipEffectApplied per leaf; bare leaf → fires ChipEffectApplied directly; triggered chain → pushes to ActiveChains (cross-domain write to behaviors resource, accepted compromise). handle_overclock observer DELETED — replaced by direct push in apply_chip_effect. TriggerChain trigger wrappers use Vec<Self> (not Box<Self> as content.md shows).
- **behaviors→effect domain rename** (2026-03-25, C7-R): `behaviors/` renamed to `effect/`. BehaviorsPlugin→EffectPlugin. BehaviorSystems→EffectSystems. ActiveChains→ActiveEffects. ArmedTriggers→ArmedEffects. EffectFired catchall DELETED — replaced by per-effect typed events (ShockwaveFired, LoseLifeFired, etc.) via fire_typed_event(). New type system: Trigger/Effect/EffectNode enums (effect/definition.rs) parallel to TriggerChain (chips/definition.rs) — transitional. trigger_chain_to_effect() converts between them. BreakerDefinition moved to breaker/definition.rs. ActiveEffects pub for scenario runner.
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
- NOTE (2026-03-21): bolt/apply_bump_velocity DELETED — velocity scaling now via Effect::SpeedBoost { multiplier } leaf in archetype RON, handled by handle_speed_boost observer in effect/effects/speed_boost.rs. BumpPerformed no longer carries a multiplier field.
- PhysicsConfig/PhysicsDefaults no longer exist (all fields moved to BreakerConfig)

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin)
- Tests are in-module `#[cfg(test)]` blocks
- Init system tests verify: all components inserted, values match config, skip guard works

## Doc Sync (2026-03-13)
All docs updated to match code reality. Key fixes: plugins.md, messages.md, ordering.md, content.md, standards.md, README.md, data.md.
