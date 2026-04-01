pub(super) use bevy::prelude::*;
pub(super) use rantzsoft_spatial2d::components::Velocity2D;

pub(super) use super::super::effect::*;
pub(super) use crate::{
    breaker::components::{Breaker, DashState},
    effect::effects::bump_force::ActiveBumpForces,
    shared::{game_state::GameState, playing_state::PlayingState},
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, tick_anchor);
    app
}

pub(super) fn test_app_fixed() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, tick_anchor);
    app
}

/// Accumulates one fixed timestep then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn register_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.add_sub_state::<PlayingState>();
    // Transition into Playing state so PlayingState::Active becomes active
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    app
}
