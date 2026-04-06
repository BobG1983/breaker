use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_spatial2d::prelude::*;

use crate::{
    bolt::queries::{BoltSpeedData, apply_velocity_formula},
    shared::GameDrawLayer,
    state::types::NodeState,
};

/// Placeholder gravity well color — HDR purple.
const GRAVITY_WELL_COLOR: Color = Color::linear_rgb(1.0, 0.2, 2.0);

/// Marker for gravity well entities.
#[derive(Component)]
pub struct GravityWell;

/// Configuration and runtime state for a gravity well.
#[derive(Component)]
pub struct GravityWellConfig {
    /// Pull strength applied to bolts within radius.
    pub strength: f32,
    /// Attraction radius in world units.
    pub radius: f32,
    /// Remaining duration in seconds.
    pub remaining: f32,
    /// Entity that spawned this well.
    pub owner: Entity,
}

/// Monotonically increasing per-owner spawn order stamped on each gravity well.
/// Lower values are older. Used for deterministic FIFO despawn ordering.
#[derive(Component)]
pub struct GravityWellSpawnOrder(pub u64);

/// Per-owner counter tracking the next spawn order value to assign.
/// Lazily initialized in `fire()` on first use.
#[derive(Resource, Default)]
pub struct GravityWellSpawnCounter(pub HashMap<Entity, u64>);

pub(crate) fn fire(
    entity: Entity,
    strength: f32,
    duration: f32,
    radius: f32,
    max: u32,
    _source_chip: &str,
    world: &mut World,
) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if max == 0 {
        return;
    }

    let position = super::super::entity_position(world, entity);

    // SCOPE A — read counter value from resource (copy out, drop borrow).
    let counter_value: u64 = {
        let counter_resource = world.get_resource_or_insert_with(GravityWellSpawnCounter::default);
        *counter_resource.0.get(&entity).unwrap_or(&0)
    };

    // SCOPE B — query owned wells with spawn order for deterministic FIFO despawn.
    let to_despawn: Vec<Entity> = {
        let mut query = world.query::<(Entity, &GravityWellConfig, &GravityWellSpawnOrder)>();
        let mut owned: Vec<(Entity, u64)> = query
            .iter(world)
            .filter(|(_, config, _)| config.owner == entity)
            .map(|(e, _, order)| (e, order.0))
            .collect();
        owned.sort_by_key(|(_, order)| *order); // ascending — lowest = oldest

        let mut despawn_list: Vec<Entity> = Vec::new();
        while owned.len() - despawn_list.len() >= max as usize {
            despawn_list.push(owned[despawn_list.len()].0);
        }
        despawn_list
    };

    // Despawn outside the query scope.
    for e in &to_despawn {
        world.despawn(*e);
    }

    // Spawn the new well with its spawn order stamp.
    let visual = {
        let mesh = world
            .get_resource_mut::<Assets<Mesh>>()
            .map(|mut m| m.add(Circle::new(1.0)));
        let mat = world
            .get_resource_mut::<Assets<ColorMaterial>>()
            .map(|mut m| m.add(ColorMaterial::from_color(GRAVITY_WELL_COLOR)));
        mesh.zip(mat)
    };

    let mut well = world.spawn((
        GravityWell,
        GravityWellConfig {
            strength,
            radius,
            remaining: duration,
            owner: entity,
        },
        GravityWellSpawnOrder(counter_value),
        Spatial::builder().at_position(position).build(),
        Scale2D {
            x: radius,
            y: radius,
        },
        GameDrawLayer::Fx,
        CleanupOnExit::<NodeState>::default(),
    ));
    if let Some((mesh, mat)) = visual {
        well.insert((Mesh2d(mesh), MeshMaterial2d(mat)));
    }

    // SCOPE C — re-borrow resource to store incremented counter.
    {
        if let Some(mut counter_resource) = world.get_resource_mut::<GravityWellSpawnCounter>() {
            counter_resource.0.insert(entity, counter_value + 1);
        }
    }
}

/// No-op — gravity wells self-despawn via their duration timer.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Decrement well timers and despawn expired wells.
pub(crate) fn tick_gravity_well(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GravityWellConfig), With<GravityWell>>,
) {
    let dt = time.delta_secs();
    for (entity, mut config) in &mut query {
        config.remaining -= dt;
        if config.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Pull bolts toward active gravity wells.
///
/// `Without<GravityWell>` on the bolt query is required for Bevy query disjointness —
/// both queries access `Position2D`, so the type system needs proof they can't overlap.
pub(crate) fn apply_gravity_pull(
    time: Res<Time>,
    wells: Query<(&Position2D, &GravityWellConfig), With<GravityWell>>,
    mut bolts: Query<BoltSpeedData, Without<GravityWell>>,
) {
    let dt = time.delta_secs();
    for (well_position, config) in &wells {
        let well_pos = well_position.0;
        for mut bolt in &mut bolts {
            let bolt_pos = bolt.spatial.position.0;
            let delta = well_pos - bolt_pos;
            let distance = delta.length();
            if distance > 0.0 && distance <= config.radius {
                let direction = delta / distance;
                let steering = direction * config.strength * dt;
                bolt.spatial.velocity.0 = (bolt.spatial.velocity.0 + steering).normalize_or_zero();
                apply_velocity_formula(&mut bolt.spatial, bolt.active_speed_boosts);
            }
        }
    }
}

/// Syncs `Scale2D` to match `GravityWellConfig.radius` each tick so the visual
/// mesh tracks the gravity well's influence area.
pub(crate) fn sync_gravity_well_visual(
    mut query: Query<(&GravityWellConfig, &mut Scale2D), With<GravityWell>>,
) {
    for (config, mut scale) in &mut query {
        scale.x = config.radius;
        scale.y = config.radius;
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_gravity_well,
            sync_gravity_well_visual,
            apply_gravity_pull,
        )
            .chain()
            .run_if(in_state(NodeState::Playing)),
    );
}
