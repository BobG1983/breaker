use bevy::prelude::*;
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};

use crate::{
    bolt::{definition::BoltDefinition, messages::BoltLost, systems::bolt_lost::system::bolt_lost},
    prelude::*,
    shared::rng::BoltRng,
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
    let mut app = TestAppBuilder::new()
        .with_playfield()
        .with_resource::<BoltRng>()
        .with_message::<BoltLost>()
        .with_system(FixedUpdate, bolt_lost)
        .build();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Bolt>();
    app
}

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
