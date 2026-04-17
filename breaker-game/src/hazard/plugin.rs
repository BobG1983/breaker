//! Hazard domain plugin.

use bevy::prelude::*;

use super::{
    messages::HazardSelected,
    resources::{ActiveHazards, HazardOffers},
};

/// Registers hazard resources and messages. Per-hazard modules register
/// themselves here from their own `register(app)` functions as they land.
pub(crate) struct HazardPlugin;

impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveHazards>()
            .init_resource::<HazardOffers>()
            .add_message::<HazardSelected>();
    }
}
