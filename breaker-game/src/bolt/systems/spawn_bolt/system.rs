//! System to spawn the bolt entity.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;
use tracing::{debug, warn};

use crate::{
    bolt::{
        components::Bolt,
        messages::BoltSpawned,
        registry::BoltRegistry,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    breaker::{BreakerRegistry, SelectedBreaker, components::Breaker},
    run::RunState,
    shared::GameRng,
};

/// Spawns the bolt entity above the breaker using the [`Bolt`] builder.
///
/// Looks up the bolt definition from [`BoltRegistry`] via the
/// [`SelectedBreaker`] -> [`BreakerRegistry`] chain. Falls back to
/// [`BreakerDefinition::y_position`] when the breaker entity does not exist
/// yet (both systems run on `OnEnter(Playing)` and deferred commands
/// mean the breaker entity may not exist yet).
///
/// On the first node (`RunState.node_index == 0`), the bolt spawns with
/// zero velocity and a [`BoltServing`] marker -- it hovers until the player
/// presses the bump button. On subsequent nodes it launches immediately
/// with a random angle within `DEFAULT_BOLT_ANGLE_SPREAD`.
///
/// The builder's [`definition`] inserts all definition-derived components
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

    // Clone/copy all data from immutable borrows BEFORE any mutable borrows.
    let selected_name = world.resource::<SelectedBreaker>().0.clone();
    let Some(breaker_def) = world
        .resource::<BreakerRegistry>()
        .get(&selected_name)
        .cloned()
    else {
        warn!("Breaker '{selected_name}' not found in BreakerRegistry");
        return;
    };
    let bolt_name = &breaker_def.bolt;
    let Some(bolt_def) = world.resource::<BoltRegistry>().get(bolt_name).cloned() else {
        warn!("Bolt '{bolt_name}' (from breaker '{selected_name}') not found in BoltRegistry");
        return;
    };
    let breaker_default_y = breaker_def.y_position;
    let run_state_node_index = world.resource::<RunState>().node_index;

    let breaker_pos = world
        .query_filtered::<&Position2D, With<Breaker>>()
        .iter(world)
        .next()
        .map(|p| p.0);

    let breaker_y = breaker_pos.map_or(breaker_default_y, |p| p.y);
    let breaker_x = breaker_pos.map_or(0.0, |p| p.x);

    let spawn_pos = Vec2::new(breaker_x, breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    let serving = run_state_node_index == 0;

    let entity = if serving {
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .serving()
            .primary()
            .spawn(world)
    } else {
        // Random angle within +/- DEFAULT_BOLT_ANGLE_SPREAD
        let angle = world
            .resource_mut::<GameRng>()
            .0
            .random_range(-DEFAULT_BOLT_ANGLE_SPREAD..=DEFAULT_BOLT_ANGLE_SPREAD);
        let velocity = Velocity2D(Vec2::new(
            bolt_def.base_speed * angle.sin(),
            bolt_def.base_speed * angle.cos(),
        ));
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .with_velocity(velocity)
            .primary()
            .spawn(world)
    };

    // Render components are not part of the builder (rendering concern).
    let mesh = world.resource_mut::<Assets<Mesh>>().add(Circle::new(1.0));
    let color = Color::linear_rgb(
        bolt_def.color_rgb[0],
        bolt_def.color_rgb[1],
        bolt_def.color_rgb[2],
    );
    let material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from_color(color));
    world
        .entity_mut(entity)
        .insert((Mesh2d(mesh), MeshMaterial2d(material)));

    debug!("bolt spawned entity={entity:?}");

    world
        .resource_mut::<Messages<BoltSpawned>>()
        .write(BoltSpawned);
}
