use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use crate::{
    bolt::components::Bolt,
    shared::{CleanupOnNodeExit, playing_state::PlayingState},
};

/// Marker for gravity well entities.
#[derive(Component)]
pub struct GravityWellMarker;

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

pub(crate) fn fire(
    entity: Entity,
    strength: f32,
    duration: f32,
    radius: f32,
    max: u32,
    _source_chip: &str,
    world: &mut World,
) {
    if max == 0 {
        return;
    }

    let position = super::super::entity_position(world, entity);

    // Enforce max active wells for this owner — despawn oldest if at cap.
    let mut owned: Vec<Entity> = Vec::new();
    {
        let mut query = world.query::<(Entity, &GravityWellConfig)>();
        for (well_entity, config) in query.iter(world) {
            if config.owner == entity {
                owned.push(well_entity);
            }
        }
    }

    // Despawn order is arbitrary (ECS query iteration is not guaranteed FIFO).
    while owned.len() >= max as usize {
        if let Some(oldest) = owned.first().copied() {
            world.despawn(oldest);
            owned.remove(0);
        }
    }

    world.spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength,
            radius,
            remaining: duration,
            owner: entity,
        },
        Position2D(position),
        CleanupOnNodeExit,
    ));
}

/// No-op — gravity wells self-despawn via their duration timer.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Decrement well timers and despawn expired wells.
pub(crate) fn tick_gravity_well(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GravityWellConfig), With<GravityWellMarker>>,
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
pub(crate) fn apply_gravity_pull(
    time: Res<Time>,
    wells: Query<(&Position2D, &GravityWellConfig), With<GravityWellMarker>>,
    mut bolts: Query<(&Position2D, &mut Velocity2D), With<Bolt>>,
) {
    let dt = time.delta_secs();
    for (well_position, config) in &wells {
        let well_pos = well_position.0;
        for (bolt_position, mut velocity) in &mut bolts {
            let bolt_pos = bolt_position.0;
            let delta = well_pos - bolt_pos;
            let distance = delta.length();
            if distance > 0.0 && distance <= config.radius {
                let direction = delta / distance;
                let pull = config.strength * dt;
                velocity.x = direction.x.mul_add(pull, velocity.x);
                velocity.y = direction.y.mul_add(pull, velocity.y);
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    use crate::bolt::BoltSystems;

    app.add_systems(
        FixedUpdate,
        (
            tick_gravity_well,
            apply_gravity_pull.before(BoltSystems::PrepareVelocity),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}
