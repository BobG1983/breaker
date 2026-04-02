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
/// components ([`Mesh2d`], [`MeshMaterial2d`], [`GameDrawLayer::Bolt`])
/// are added via `.rendered()` on the builder.
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

    // Compute random angle upfront (before moving resources out for rendered()).
    // Only needed for non-serving bolts (node index > 0).
    let random_angle = if serving {
        0.0
    } else {
        world
            .resource_mut::<GameRng>()
            .0
            .random_range(-DEFAULT_BOLT_ANGLE_SPREAD..=DEFAULT_BOLT_ANGLE_SPREAD)
    };

    // Extract asset stores so .rendered() can create mesh+material handles
    // without conflicting with the &mut World borrow in .spawn().
    let (mut meshes, mut materials) = (
        world.remove_resource::<Assets<Mesh>>().unwrap_or_default(),
        world
            .remove_resource::<Assets<ColorMaterial>>()
            .unwrap_or_default(),
    );

    let entity = if serving {
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .serving()
            .primary()
            .rendered(&mut meshes, &mut materials)
            .spawn(world)
    } else {
        let velocity = Velocity2D(Vec2::new(
            bolt_def.base_speed * random_angle.sin(),
            bolt_def.base_speed * random_angle.cos(),
        ));
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .with_velocity(velocity)
            .primary()
            .rendered(&mut meshes, &mut materials)
            .spawn(world)
    };

    // Re-insert asset stores.
    world.insert_resource(meshes);
    world.insert_resource(materials);

    debug!("bolt spawned entity={entity:?}");

    world
        .resource_mut::<Messages<BoltSpawned>>()
        .write(BoltSpawned);
}
