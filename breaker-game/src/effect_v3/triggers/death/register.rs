//! Registration for death trigger bridges.

use bevy::prelude::*;

use super::bridges;
use crate::shared::death_pipeline::sets::DeathPipelineSystems;

/// Registers all death trigger bridge systems ordered after
/// [`DeathPipelineSystems::HandleKill`] so they observe the `Destroyed<T>`
/// messages on the same tick the victim entity is still alive.
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_cell_destroyed,
            bridges::on_bolt_destroyed,
            bridges::on_wall_destroyed,
            bridges::on_breaker_destroyed,
        )
            .after(DeathPipelineSystems::HandleKill),
    );
}
