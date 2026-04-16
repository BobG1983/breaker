//! Registration for death trigger bridges.

use bevy::prelude::*;

use super::bridges;
use crate::{effect_v3::EffectV3Systems, shared::death_pipeline::sets::DeathPipelineSystems};

/// Registers all death trigger bridge systems in [`EffectV3Systems::Death`],
/// which is ordered `.after(DeathPipelineSystems::HandleKill)` so the bridges
/// observe the `Destroyed<T>` messages on the same tick the victim entity is
/// still alive in the world (despawn runs later in `FixedPostUpdate`).
///
/// The `Death` set is the cross-domain ordering anchor — consumers that need
/// to run before or after the death-bridge phase should use
/// `.before(EffectV3Systems::Death)` / `.after(EffectV3Systems::Death)`.
pub fn register(app: &mut App) {
    app.configure_sets(
        FixedUpdate,
        EffectV3Systems::Death.after(DeathPipelineSystems::HandleKill),
    );
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_cell_destroyed,
            bridges::on_bolt_destroyed,
            bridges::on_wall_destroyed,
            bridges::on_breaker_destroyed,
            bridges::on_salvo_destroyed,
        )
            .in_set(EffectV3Systems::Death),
    );
}
