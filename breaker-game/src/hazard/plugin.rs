//! Hazard domain plugin.

use bevy::prelude::*;

use super::{
    hazards,
    messages::HazardSelected,
    resources::{ActiveHazards, HazardOffers},
    systems::dispatch_hazard_selection,
};
use crate::{prelude::*, state::run::hazard_select::sets::HazardSelectSystems};

/// Registers hazard resources and messages. Per-hazard modules register
/// themselves here from their own `register(app)` functions as they land.
pub(crate) struct HazardPlugin;

impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveHazards>()
            .init_resource::<HazardOffers>()
            .add_message::<HazardSelected>()
            .add_systems(
                Update,
                dispatch_hazard_selection
                    .after(HazardSelectSystems::HandleInput)
                    .after(HazardSelectSystems::TickTimer)
                    .run_if(in_state(HazardSelectState::Selecting)),
            );
        hazards::register(app);
    }
}
