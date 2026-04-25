//! Shared test fixtures for Haste hazard tests.
//!
//! State-hierarchy apps (`NodeState::Playing` and a non-Playing variant),
//! bolt spawn helpers, a fixed-timestep ticker, and `ActiveHazards` /
//! `HasteConfig` installers. Haste emits no messages, so unlike Renewal or
//! Cascade there is no message collector here.

use bevy::prelude::*;

use super::super::system::{HasteConfig, haste_apply_speed};
use crate::{
    bolt::components::Bolt,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
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
/// The default state is not `Playing` — mirrors the renewal pattern in
/// `renewal/tests/register.rs::system_skipped_when_not_in_node_playing`.
pub(super) fn test_app_not_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .build()
}

/// Spawns a `(Bolt,)` entity and returns its id.
pub(super) fn spawn_bolt(app: &mut App) -> Entity {
    app.world_mut().spawn(Bolt).id()
}

/// Spawns a `(Bolt, EffectStack<SpeedBoostConfig>)` entity seeded with the
/// given stack. Returns the entity id.
pub(super) fn spawn_bolt_with_stack(app: &mut App, seed: EffectStack<SpeedBoostConfig>) -> Entity {
    app.world_mut().spawn((Bolt, seed)).id()
}

/// Runs one full `Time<Fixed>` timestep.
pub(super) fn run_fixed_update(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Adds `count` Haste stacks to `ActiveHazards`.
pub(super) fn add_haste_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Haste);
    }
}

/// Inserts the given `HasteConfig` as a resource.
pub(super) fn install_haste_config(app: &mut App, cfg: HasteConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Wires only `haste_apply_speed` in `FixedUpdate` — bypasses run-conditions.
/// Used by Groups B, C, F, G to exercise the system directly without the
/// `hazard_active` / `in_state` gates `register` installs.
pub(super) fn wire_apply_only(app: &mut App) {
    app.add_systems(FixedUpdate, haste_apply_speed);
}

/// Collects `(source, config)` entries from an `EffectStack<SpeedBoostConfig>`.
/// Used to assert source membership / cardinality / multiplier values.
pub(super) fn haste_entries(
    stack: &EffectStack<SpeedBoostConfig>,
) -> Vec<(SourceId, SpeedBoostConfig)> {
    stack.iter().map(|(s, c)| (s.clone(), c.clone())).collect()
}

/// Canonical tuning pair used across most tests: `base=20.0, per_level=10.0`.
pub(super) const fn canonical_config() -> HasteConfig {
    HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    }
}
