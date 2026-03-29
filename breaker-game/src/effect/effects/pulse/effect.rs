use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Emitter component attached to a bolt entity. Drives periodic ring emission.
#[derive(Component)]
pub struct PulseEmitter {
    /// Base range of emitted rings.
    pub base_range: f32,
    /// Additional range per stack level.
    pub range_per_level: f32,
    /// Current stack count.
    pub stacks: u32,
    /// Expansion speed of emitted rings in world units per second.
    pub speed: f32,
    /// Seconds between ring emissions.
    pub interval: f32,
    /// Accumulated time since last emission.
    pub timer: f32,
}

impl PulseEmitter {
    /// Effective maximum radius: `base_range + (stacks - 1) * range_per_level`.
    #[must_use]
    pub fn effective_max_radius(&self) -> f32 {
        super::super::effective_range(self.base_range, self.range_per_level, self.stacks)
    }
}

/// Marker component on pulse ring entities.
#[derive(Component)]
pub struct PulseRing;

/// The entity that spawned this ring (typically a bolt).
#[derive(Component)]
pub struct PulseSource(pub Entity);

/// Current expanding radius of the ring.
#[derive(Component)]
pub struct PulseRadius(pub f32);

/// Maximum radius before the ring despawns.
#[derive(Component)]
pub struct PulseMaxRadius(pub f32);

/// Expansion speed in world units per second.
#[derive(Component)]
pub struct PulseSpeed(pub f32);

/// Tracks which cells have been damaged by this specific ring.
#[derive(Component, Default)]
pub struct PulseDamaged(pub HashSet<Entity>);

pub fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    world: &mut World,
) {
    let emitter = PulseEmitter {
        base_range,
        range_per_level,
        stacks,
        speed,
        interval: 0.5,
        timer: 0.0,
    };
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(emitter);
    }
}

pub fn reverse(entity: Entity, world: &mut World) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.remove::<PulseEmitter>();
    }
}

/// Tick emitter timers and spawn pulse rings when interval elapses.
///
/// Uses manual `f32` timer accumulation. When the timer reaches the interval,
/// spawns a [`PulseRing`] entity at the emitter's current position.
pub fn tick_pulse_emitter(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut emitters: Query<(Entity, &mut PulseEmitter, &Transform)>,
) {
    let dt = time.timestep().as_secs_f32();
    for (entity, mut emitter, transform) in &mut emitters {
        emitter.timer += dt;
        if emitter.timer >= emitter.interval {
            emitter.timer -= emitter.interval;
            let effective_range = emitter.effective_max_radius();
            let speed = emitter.speed;
            commands.spawn((
                PulseRing,
                PulseSource(entity),
                PulseRadius(0.0),
                PulseMaxRadius(effective_range),
                PulseSpeed(speed),
                PulseDamaged(HashSet::new()),
                Transform::from_translation(transform.translation),
                CleanupOnNodeExit,
            ));
        }
    }
}

/// Expand pulse ring radius by speed * dt each tick.
pub fn tick_pulse_ring(
    time: Res<Time>,
    mut rings: Query<(&mut PulseRadius, &PulseSpeed), With<PulseRing>>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut rings {
        radius.0 += speed.0 * dt;
    }
}

/// Damage cells within each pulse ring radius.
///
/// For each ring, queries the quadtree for cells within the current radius
/// and sends [`DamageCell`] for any cell not already in the [`PulseDamaged`] set.
pub fn apply_pulse_damage(
    quadtree: Res<CollisionQuadtree>,
    mut rings: Query<(&Transform, &PulseRadius, &mut PulseDamaged), With<PulseRing>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (transform, radius, mut damaged) in &mut rings {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = transform.translation.truncate();
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {
                damage_writer.write(DamageCell {
                    cell,
                    damage: BASE_BOLT_DAMAGE,
                    source_chip: None,
                });
            }
        }
    }
}

/// Despawn pulse rings that have reached their maximum radius.
pub fn despawn_finished_pulse_ring(
    mut commands: Commands,
    rings: Query<(Entity, &PulseRadius, &PulseMaxRadius), With<PulseRing>>,
) {
    for (entity, radius, max_radius) in &rings {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_pulse_emitter,
            tick_pulse_ring,
            apply_pulse_damage,
            despawn_finished_pulse_ring,
        )
            .chain()
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
