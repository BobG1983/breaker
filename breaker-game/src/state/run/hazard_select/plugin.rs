//! Hazard selection screen plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};
use rantzsoft_stateflow::{Route, RoutingTableAppExt, cleanup_on_exit};

use super::{
    HazardSelectScreen,
    sets::HazardSelectSystems,
    systems::{
        generate_hazard_offerings, handle_hazard_input, spawn_hazard_select, tick_hazard_timer,
    },
};
use crate::{prelude::*, state::cleanup::cleanup_entities};

/// Plugin for the tier-9+ hazard selection screen.
pub(crate) struct HazardSelectPlugin;

impl Plugin for HazardSelectPlugin {
    fn build(&self, app: &mut App) {
        // HazardSelectState routes — hazard selection lifecycle
        app.add_route(
            Route::from(HazardSelectState::Loading)
                .to(HazardSelectState::AnimateIn)
                .when(|_| true),
        );
        app.add_route(
            Route::from(HazardSelectState::AnimateIn)
                .to(HazardSelectState::Selecting)
                .when(|_| true),
        );
        // Selecting → AnimateOut: message-triggered (handle_hazard_input/tick_hazard_timer)
        app.add_route(Route::from(HazardSelectState::Selecting).to(HazardSelectState::AnimateOut));
        app.add_route(
            Route::from(HazardSelectState::AnimateOut)
                .to(HazardSelectState::Teardown)
                .when(|_| true),
        );
        app.add_systems(
            OnEnter(HazardSelectState::Teardown),
            cleanup_on_exit::<HazardSelectState>,
        );

        app.add_systems(
            OnEnter(HazardSelectState::Selecting),
            (
                generate_hazard_offerings.in_set(HazardSelectSystems::GenerateOfferings),
                ApplyDeferred,
                spawn_hazard_select.in_set(HazardSelectSystems::SpawnScreen),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                handle_hazard_input.in_set(HazardSelectSystems::HandleInput),
                tick_hazard_timer.in_set(HazardSelectSystems::TickTimer),
            )
                .chain()
                .run_if(in_state(HazardSelectState::Selecting)),
        )
        .add_systems(
            OnExit(HazardSelectState::Selecting),
            cleanup_entities::<HazardSelectScreen>,
        );
    }
}
