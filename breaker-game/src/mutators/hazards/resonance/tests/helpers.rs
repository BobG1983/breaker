//! Shared test helpers for the Resonance hazard test suite.

use std::{marker::PhantomData, time::Duration};

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;

use super::super::system::{
    ResonanceActiveSlows, ResonanceConfig, ResonanceSlowEntry, ResonanceTracker, ResonanceWave,
};
use crate::{
    breaker::components::Breaker,
    cells::components::Cell,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::{Destroyed, *},
};

/// Canonical `ResonanceConfig` used across the suite.
pub(super) fn canonical_config() -> ResonanceConfig {
    ResonanceConfig {
        kills_to_trigger:      2,
        base_window:           0.5,
        window_per_level:      0.3,
        wave_speed:            200.0,
        base_slow_duration:    1.5,
        base_slow_strength:    0.5,
        slow_duration_scaling: 0.2,
        slow_strength_scaling: 0.15,
        contact_threshold:     16.0,
        wave_max_lifetime:     10.0,
    }
}

/// TestAppBuilder-driven App at `NodeState::Playing` with `ActiveHazards`
/// and `Destroyed<Cell>` messages registered. Adds canonical resonance
/// resources (Config, Tracker, `ActiveSlows`) so individual tests can drop
/// straight into their behavior. Stack 1 of Resonance added by default.
pub(super) fn test_app_playing() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build();
    app.insert_resource(canonical_config());
    app.init_resource::<ResonanceTracker>();
    app.init_resource::<ResonanceActiveSlows>();
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Resonance);
    app
}

/// Variant of `test_app_playing` without any Resonance stack (gating tests).
pub(super) fn test_app_playing_no_stack() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .build();
    app.insert_resource(canonical_config());
    app.init_resource::<ResonanceTracker>();
    app.init_resource::<ResonanceActiveSlows>();
    app
}

/// TestAppBuilder-driven App at `NodeState::Playing` with full effects
/// pipeline wired up (for Groups F, G, I, J).
pub(super) fn test_app_playing_with_effects() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<Destroyed<Cell>>()
        .with_effects_pipeline()
        .build();
    app.insert_resource(canonical_config());
    app.init_resource::<ResonanceTracker>();
    app.init_resource::<ResonanceActiveSlows>();
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Resonance);
    app
}

/// Spawns a breaker at a given `Position2D`. Adds an empty
/// `EffectStack<SpeedBoostConfig>` so effect-pipeline tests have
/// something to assert against from the start.
pub(super) fn spawn_breaker_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            Position2D(pos),
            EffectStack::<SpeedBoostConfig>::default(),
        ))
        .id()
}

/// Spawns a breaker at `pos` with NO initial `EffectStack<SpeedBoostConfig>`.
pub(super) fn spawn_breaker_at_no_stack(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Breaker, Position2D(pos))).id()
}

/// Parameters for `spawn_wave`. Mirrors the fields of `ResonanceWave` plus a
/// `Position2D` for tests that exercise travel/contact/expire in isolation.
pub(super) struct SpawnWaveParams {
    pub pos:               Vec2,
    pub speed:             f32,
    pub slow_duration:     f32,
    pub slow_strength:     f32,
    pub target_pos:        Vec2,
    pub age:               f32,
    pub max_lifetime:      f32,
    pub contact_threshold: f32,
}

/// Spawns a standalone `ResonanceWave` entity with given fields and
/// `Position2D` for tests that exercise travel/contact/expire in isolation.
pub(super) fn spawn_wave(app: &mut App, params: SpawnWaveParams) -> Entity {
    app.world_mut()
        .spawn((
            ResonanceWave {
                speed:             params.speed,
                slow_duration:     params.slow_duration,
                slow_strength:     params.slow_strength,
                target_pos:        params.target_pos,
                age:               params.age,
                max_lifetime:      params.max_lifetime,
                contact_threshold: params.contact_threshold,
            },
            Position2D(params.pos),
        ))
        .id()
}

/// Writes a `Destroyed<Cell>` message at the given victim position.
pub(super) fn write_cell_destroyed(app: &mut App, pos: Vec2) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: pos,
            killer_pos: None,
            _marker:    PhantomData,
        });
}

/// Advances one fixed timestep of `dt`, updating the app.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Runs `count` ticks at `dt_secs` each. Returns the approximate
/// `Time<Fixed>::elapsed_secs()` that should be visible after the runs.
pub(super) fn tick_n(app: &mut App, count: u32, dt_secs: f32) -> f32 {
    let dt = Duration::from_secs_f32(dt_secs);
    for _ in 0..count {
        tick_with_dt(app, dt);
    }
    app.world().resource::<Time<Fixed>>().elapsed_secs()
}

/// Returns the list of source strings on the breaker's `SpeedBoostConfig`
/// stack that START WITH the given prefix.
pub(super) fn collect_sources_with_prefix(app: &App, breaker: Entity, prefix: &str) -> Vec<String> {
    app.world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .map(|stack| {
            stack
                .iter()
                .filter(|(s, _)| s.0.starts_with(prefix))
                .map(|(s, _)| s.0.clone().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Returns the count of entries in the breaker's `SpeedBoostConfig` stack
/// whose source EXACTLY matches `source`.
pub(super) fn count_stack_entries_with_source(app: &App, breaker: Entity, source: &str) -> usize {
    app.world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .map_or(0, |stack| {
            stack.iter().filter(|(s, _)| s.0.as_ref() == source).count()
        })
}

/// Fires a `SpeedBoostConfig { multiplier: OrderedFloat(0.5) }` onto the
/// breaker's stack via `Fireable::fire` directly (world-mut path used for
/// tests that need a deterministic pre-seeded state), and inserts a matching
/// `ResonanceSlowEntry` into `ResonanceActiveSlows.slows` with the same
/// multiplier.
///
/// The `0.5` multiplier is canonical across Group G. Storing the same value
/// in both the stack entry and the `ResonanceSlowEntry` is what lets the
/// production `reverse_effect` call find and remove the stack entry via
/// `(source, config)` match semantics.
pub(super) fn seed_active_slow(app: &mut App, breaker: Entity, source: &str, remaining: f32) {
    use crate::effect_v3::traits::Fireable;

    let source_s = source.to_owned();
    app.world_mut()
        .resource_mut::<ResonanceActiveSlows>()
        .slows
        .insert(
            source_s.clone(),
            ResonanceSlowEntry {
                remaining,
                multiplier: OrderedFloat(0.5),
            },
        );
    let config = SpeedBoostConfig {
        multiplier: OrderedFloat(0.5),
    };
    config.fire(
        breaker,
        &source_s,
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
}
