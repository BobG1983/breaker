//! Chip selection screen plugin registration.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};
use rantzsoft_stateflow::{Route, RoutingTableAppExt, cleanup_on_exit};

use super::{
    ChipSelectScreen,
    messages::ChipOfferSkipped,
    sets::ChipSelectSystems,
    systems::{
        generate_chip_offerings, handle_chip_input, reseed_chip_rng, spawn_chip_select,
        tick_chip_select_count, tick_chip_timer, update_chip_display,
    },
};
use crate::{prelude::*, shared::rng::ChipSelectCount, state::cleanup::cleanup_entities};

/// Plugin for the between-node chip selection screen.
pub(crate) struct ChipSelectPlugin;

impl Plugin for ChipSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChipSelectCount>();
        app.add_message::<ChipOfferSkipped>();

        // ChipSelectState routes — chip selection lifecycle
        app.add_route(
            Route::from(ChipSelectState::Loading)
                .to(ChipSelectState::AnimateIn)
                .when(|_| true),
        );
        app.add_route(
            Route::from(ChipSelectState::AnimateIn)
                .to(ChipSelectState::Selecting)
                .when(|_| true),
        );
        // Selecting → AnimateOut: message-triggered (handle_chip_input/tick_chip_timer)
        app.add_route(Route::from(ChipSelectState::Selecting).to(ChipSelectState::AnimateOut));
        app.add_route(
            Route::from(ChipSelectState::AnimateOut)
                .to(ChipSelectState::Teardown)
                .when(|_| true),
        );
        app.add_systems(
            OnEnter(ChipSelectState::Teardown),
            cleanup_on_exit::<ChipSelectState>,
        );

        app.add_systems(
            OnEnter(ChipSelectState::Selecting),
            (
                reseed_chip_rng.in_set(ChipSelectSystems::ReseedRng),
                generate_chip_offerings.in_set(ChipSelectSystems::GenerateOfferings),
                ApplyDeferred,
                spawn_chip_select.in_set(ChipSelectSystems::SpawnScreen),
                tick_chip_select_count.in_set(ChipSelectSystems::TickCount),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                handle_chip_input.in_set(ChipSelectSystems::HandleInput),
                tick_chip_timer,
                update_chip_display,
            )
                .chain()
                .run_if(in_state(ChipSelectState::Selecting)),
        )
        .add_systems(
            OnExit(ChipSelectState::Selecting),
            cleanup_entities::<ChipSelectScreen>,
        );
    }
}
