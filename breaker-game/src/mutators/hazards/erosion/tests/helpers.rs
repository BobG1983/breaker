//! Shared test fixtures for Erosion hazard tests.
//!
//! Mirrors Haste's helpers layout: state-hierarchy apps (`NodeState::Playing`
//! and a non-Playing variant), breaker spawn helpers, fixed-timestep
//! tickers, and `ActiveHazards` / `ErosionConfig` / `ErosionState`
//! installers. Also provides per-group wiring helpers.

use std::time::Duration;

use bevy::prelude::*;

use super::super::system::{
    ErosionConfig, ErosionState, erosion_apply_width, erosion_restore, erosion_shrink,
};
use crate::{
    breaker::{components::Breaker, messages::BumpPerformed},
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

/// Default builder: state hierarchy at `NodeState::Playing`, `ActiveHazards`.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .build()
}

/// State-hierarchy app that is NOT driven into `NodeState::Playing`.
/// The default state is not `Playing` — mirrors the Haste pattern.
pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .build()
}

/// Spawns a `(Breaker,)` entity and returns its id.
pub(super) fn spawn_breaker(app: &mut App) -> Entity {
    app.world_mut().spawn(Breaker).id()
}

/// Spawns a `(Breaker, EffectStack<SizeBoostConfig>)` entity seeded with the
/// given stack. Returns the entity id.
pub(super) fn spawn_breaker_with_stack(
    app: &mut App,
    seed: EffectStack<SizeBoostConfig>,
) -> Entity {
    app.world_mut().spawn((Breaker, seed)).id()
}

/// Sets the fixed timestep to `dt`, accumulates the overstep, then updates.
pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Adds `count` Erosion stacks to `ActiveHazards`.
pub(super) fn add_erosion_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Erosion);
    }
}

/// Inserts the given `ErosionConfig` as a resource.
pub(super) fn install_erosion_config(app: &mut App, cfg: ErosionConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts an `ErosionState { width_fraction: width }` resource.
pub(super) fn install_erosion_state(app: &mut App, width: f32) {
    app.world_mut().insert_resource(ErosionState {
        width_fraction: width,
    });
}

/// Canonical tuning used across most tests:
/// `shrink_rate = 0.05, min_width_frac = 0.35, restore_nonwhiff = 0.25,
/// restore_perfect = 0.50`.
pub(super) const fn canonical_config() -> ErosionConfig {
    ErosionConfig {
        shrink_rate:      0.05,
        min_width_frac:   0.35,
        restore_nonwhiff: 0.25,
        restore_perfect:  0.50,
    }
}

/// Collects `(source, config)` entries from an `EffectStack<SizeBoostConfig>`.
pub(super) fn erosion_entries(
    stack: &EffectStack<SizeBoostConfig>,
) -> Vec<(SourceId, SizeBoostConfig)> {
    stack.iter().map(|(s, c)| (s.clone(), c.clone())).collect()
}

/// Wires only `erosion_shrink` in `FixedUpdate` — bypasses run-conditions.
/// Used by Group A.
pub(super) fn wire_shrink_only(app: &mut App) {
    app.add_systems(FixedUpdate, erosion_shrink);
}

/// Wires only `erosion_restore` in `FixedUpdate` + registers the
/// `BumpPerformed` message. Used by Group B.
///
/// Also seeds 1 Erosion stack: the retrofit moved the `hazard_active`
/// gate from a `.run_if(...)` into the reader body so buffered
/// `BumpPerformed` messages drain cleanly when the hazard is off.
/// Group B tests exercise the restore path under the happy path, so
/// the hazard must be active when this helper wires the system.
pub(super) fn wire_restore_only(app: &mut App) {
    app.add_message::<BumpPerformed>();
    app.add_systems(FixedUpdate, erosion_restore);
    add_erosion_stacks(app, 1);
}

/// Wires only `erosion_apply_width` in `FixedUpdate` — bypasses
/// run-conditions. Used by Groups C and F.
pub(super) fn wire_apply_width_only(app: &mut App) {
    app.add_systems(FixedUpdate, erosion_apply_width);
}
