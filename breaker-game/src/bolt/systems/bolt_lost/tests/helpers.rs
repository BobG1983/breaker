use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::Bolt, definition::BoltDefinition, messages::BoltLost,
        systems::bolt_lost::system::bolt_lost,
    },
    shared::{
        GameRng,
        death_pipeline::{
            despawn_entity::DespawnEntity,
            destroyed::Destroyed,
            kill_yourself::KillYourself,
            systems::{handle_kill, process_despawn_requests},
        },
    },
};

pub(super) fn make_default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name:                 "Bolt".to_string(),
        base_speed:           720.0,
        min_speed:            360.0,
        max_speed:            1440.0,
        radius:               14.0,
        base_damage:          10.0,
        effects:              vec![],
        color_rgb:            [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical:   5.0,
        min_radius:           None,
        max_radius:           None,
    }
}

pub(super) fn test_app() -> App {
    use crate::shared::test_utils::TestAppBuilder;

    TestAppBuilder::new()
        .with_playfield()
        .with_resource::<GameRng>()
        .with_message::<BoltLost>()
        .with_message::<KillYourself<Bolt>>()
        .with_message::<Destroyed<Bolt>>()
        .with_message::<DespawnEntity>()
        .with_system(FixedUpdate, bolt_lost)
        .with_system(FixedUpdate, handle_kill::<Bolt>.after(bolt_lost))
        .with_system(FixedPostUpdate, process_despawn_requests)
        .build()
}

pub(super) use crate::shared::test_utils::tick;

/// Spawns a bolt at the given position with the given velocity using the builder
/// with `.definition()`.
pub(super) fn spawn_bolt(app: &mut App, pos: Vec2, vel: Vec2) -> Entity {
    let def = make_default_bolt_definition();
    let world = app.world_mut();
    let entity = Bolt::builder()
        .at_position(pos)
        .definition(&def)
        .with_velocity(Velocity2D(vel))
        .primary()
        .headless()
        .spawn(&mut world.commands());
    world.flush();
    entity
}

/// Spawns a bolt with a custom `BoltDefinition`.
pub(super) fn spawn_bolt_with_definition(
    app: &mut App,
    pos: Vec2,
    vel: Vec2,
    def: &BoltDefinition,
) -> Entity {
    let world = app.world_mut();
    let entity = Bolt::builder()
        .at_position(pos)
        .definition(def)
        .with_velocity(Velocity2D(vel))
        .primary()
        .headless()
        .spawn(&mut world.commands());
    world.flush();
    entity
}

#[derive(Resource, Default)]
pub(super) struct BoltLostCount(pub(super) u32);

pub(super) fn count_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut count: ResMut<BoltLostCount>,
) {
    for _msg in reader.read() {
        count.0 += 1;
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedKillYourselfBolt(pub(super) Vec<KillYourself<Bolt>>);

pub(super) fn capture_kill_yourself_bolt(
    mut reader: MessageReader<KillYourself<Bolt>>,
    mut captured: ResMut<CapturedKillYourselfBolt>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedBoltLost(pub(super) Vec<BoltLost>);

pub(super) fn capture_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut captured: ResMut<CapturedBoltLost>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}
