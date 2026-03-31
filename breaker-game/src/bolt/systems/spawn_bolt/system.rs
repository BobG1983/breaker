//! System to spawn the bolt entity.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;
use tracing::debug;

use crate::{
    bolt::{components::Bolt, messages::BoltSpawned, resources::BoltConfig},
    breaker::{BreakerConfig, components::Breaker},
    run::RunState,
};

/// Spawns the bolt entity above the breaker using the [`Bolt`] builder.
///
/// Reads the breaker's Y position from its [`Position2D`] when available,
/// falling back to [`BreakerConfig::y_position`] when the breaker entity
/// does not exist yet (both systems run on `OnEnter(Playing)` and deferred
/// commands mean the breaker entity may not exist yet).
///
/// On the first node (`RunState.node_index == 0`), the bolt spawns with
/// zero velocity and a [`BoltServing`] marker — it hovers until the player
/// presses the bump button. On subsequent nodes it launches immediately.
///
/// The builder's [`from_config`] inserts all config-derived components
/// (speed, angle, radius, bolt params) in a single call. Render
/// components ([`Mesh2d`], [`MeshMaterial2d`]) are added post-spawn.
pub(crate) fn spawn_bolt(world: &mut World) {
    use rantzsoft_spatial2d::components::Position2D;

    let existing_count = world
        .query_filtered::<Entity, With<Bolt>>()
        .iter(world)
        .count();

    if existing_count > 0 {
        world
            .resource_mut::<Messages<BoltSpawned>>()
            .write(BoltSpawned);
        return;
    }

    let config = world.resource::<BoltConfig>().clone();
    let breaker_default_y = world.resource::<BreakerConfig>().y_position;
    let run_state_node_index = world.resource::<RunState>().node_index;

    let breaker_pos = world
        .query_filtered::<&Position2D, With<Breaker>>()
        .iter(world)
        .next()
        .map(|p| p.0);

    let breaker_y = breaker_pos.map_or(breaker_default_y, |p| p.y);
    let breaker_x = breaker_pos.map_or(0.0, |p| p.x);

    let spawn_pos = Vec2::new(breaker_x, breaker_y + config.spawn_offset_y);

    let serving = run_state_node_index == 0;

    let entity = if serving {
        Bolt::builder()
            .at_position(spawn_pos)
            .config(&config)
            .serving()
            .primary()
            .spawn(world)
    } else {
        let v = config.initial_velocity();
        Bolt::builder()
            .at_position(spawn_pos)
            .config(&config)
            .with_velocity(Velocity2D(Vec2::new(v.x, v.y)))
            .primary()
            .spawn(world)
    };

    // Render components are not part of the builder (rendering concern).
    let mesh = world.resource_mut::<Assets<Mesh>>().add(Circle::new(1.0));
    let material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from_color(config.color()));
    world
        .entity_mut(entity)
        .insert((Mesh2d(mesh), MeshMaterial2d(material)));

    debug!("bolt spawned entity={entity:?}");

    world
        .resource_mut::<Messages<BoltSpawned>>()
        .write(BoltSpawned);
}
