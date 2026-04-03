use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::super::effect::*;
use crate::{
    bolt::{components::Bolt, definition::BoltDefinition},
    shared::PlayingState,
};

fn test_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 400.0,
        min_speed: 200.0,
        max_speed: 800.0,
        radius: 8.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::shared::GameState>();
    app.add_sub_state::<PlayingState>();
    app.add_systems(Update, tick_gravity_well);
    app.add_systems(Update, apply_gravity_pull);
    app
}

pub(super) fn spawn_bolt(app: &mut App, pos: Vec2, vel: Vec2) -> Entity {
    let def = test_bolt_definition();
    Bolt::builder()
        .at_position(pos)
        .definition(&def)
        .with_velocity(Velocity2D(vel))
        .primary()
        .headless()
        .spawn(app.world_mut())
}

pub(super) fn enter_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<crate::shared::GameState>>()
        .set(crate::shared::GameState::Playing);
    app.update();
}
