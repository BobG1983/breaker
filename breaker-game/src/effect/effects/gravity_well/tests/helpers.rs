use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::effect::*;
use crate::{
    bolt::{components::Bolt, resources::BoltConfig},
    shared::playing_state::PlayingState,
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::shared::game_state::GameState>();
    app.add_sub_state::<PlayingState>();
    app.add_systems(Update, tick_gravity_well);
    app.add_systems(Update, apply_gravity_pull);
    app
}

pub(super) fn spawn_bolt(app: &mut App, pos: Vec2, vel: Vec2) -> Entity {
    Bolt::builder()
        .at_position(pos)
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(vel))
        .primary()
        .spawn(app.world_mut())
}

pub(super) fn enter_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<crate::shared::game_state::GameState>>()
        .set(crate::shared::game_state::GameState::Playing);
    app.update();
}
