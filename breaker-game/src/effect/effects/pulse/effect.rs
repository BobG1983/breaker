use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    effect::{EffectiveDamageMultiplier, core::EffectSourceChip},
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Emitter component attached to a bolt entity. Drives periodic ring emission.
#[derive(Component)]
pub(crate) struct PulseEmitter {
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
    pub(crate) fn effective_max_radius(&self) -> f32 {
        super::super::effective_range(self.base_range, self.range_per_level, self.stacks)
    }
}

/// Marker component on pulse ring entities.
#[derive(Component)]
pub struct PulseRing;

/// Marker for pulse rings, indicating they were spawned by a pulse emitter.
#[derive(Component)]
pub(crate) struct PulseSource;

/// Current expanding radius of the ring.
#[derive(Component)]
pub(crate) struct PulseRadius(pub(crate) f32);

/// Maximum radius before the ring despawns.
#[derive(Component)]
pub(crate) struct PulseMaxRadius(pub(crate) f32);

/// Expansion speed in world units per second.
#[derive(Component)]
pub(crate) struct PulseSpeed(pub(crate) f32);

/// Tracks which cells have been damaged by this specific ring.
#[derive(Component, Default)]
pub(crate) struct PulseDamaged(pub(crate) HashSet<Entity>);

/// Damage multiplier snapshotted from the emitter's captured
/// `EffectiveDamageMultiplier` at ring-spawn time. Default `1.0`.
#[derive(Component)]
pub(crate) struct PulseRingDamageMultiplier(pub(crate) f32);

/// Query data for [`tick_pulse_emitter`].
type EmitterQuery = (
    Entity,
    &'static mut PulseEmitter,
    &'static Position2D,
    Option<&'static EffectiveDamageMultiplier>,
    Option<&'static EffectSourceChip>,
);

/// Query data for [`apply_pulse_damage`].
type PulseDamageQuery = (
    &'static Position2D,
    &'static PulseRadius,
    &'static mut PulseDamaged,
    Option<&'static PulseRingDamageMultiplier>,
    Option<&'static EffectSourceChip>,
);

pub(crate) fn fire(entity: Entity, emitter: PulseEmitter, source_chip: &str, world: &mut World) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert((emitter, EffectSourceChip::new(source_chip)));
    }
}

pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.remove::<PulseEmitter>();
    }
}

/// Tick emitter timers and spawn pulse rings when interval elapses.
///
/// Uses manual `f32` timer accumulation. When the timer reaches the interval,
/// spawns a [`PulseRing`] entity at the emitter's current position.
pub(crate) fn tick_pulse_emitter(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut emitters: Query<EmitterQuery>,
) {
    let dt = time.timestep().as_secs_f32();
    for (_entity, mut emitter, position, edm, esc) in &mut emitters {
        emitter.timer += dt;
        if emitter.timer >= emitter.interval {
            emitter.timer -= emitter.interval;
            let effective_range = emitter.effective_max_radius();
            let speed = emitter.speed;
            let damage_multiplier = edm.map_or(1.0, |e| e.0);
            let mut ring = commands.spawn((
                PulseRing,
                PulseSource,
                PulseRadius(0.0),
                PulseMaxRadius(effective_range),
                PulseSpeed(speed),
                PulseDamaged(HashSet::new()),
                PulseRingDamageMultiplier(damage_multiplier),
                Position2D(position.0),
                CleanupOnNodeExit,
            ));
            ring.insert(esc.cloned().unwrap_or_default());
        }
    }
}

/// Expand pulse ring radius by speed * dt each tick.
pub(crate) fn tick_pulse_ring(
    time: Res<Time>,
    mut rings: Query<(&mut PulseRadius, &PulseSpeed), With<PulseRing>>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut rings {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Damage cells within each pulse ring radius.
///
/// For each ring, queries the quadtree for cells within the current radius
/// and sends [`DamageCell`] for any cell not already in the [`PulseDamaged`] set.
pub(crate) fn apply_pulse_damage(
    quadtree: Res<CollisionQuadtree>,
    mut rings: Query<PulseDamageQuery, With<PulseRing>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (position, radius, mut damaged, damage_mult, esc) in &mut rings {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = position.0;
        let multiplier = damage_mult.map_or(1.0, |m| m.0);
        let source_chip = esc.and_then(EffectSourceChip::source_chip);
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {
                damage_writer.write(DamageCell {
                    cell,
                    damage: BASE_BOLT_DAMAGE * multiplier,
                    source_chip: source_chip.clone(),
                });
            }
        }
    }
}

/// Despawn pulse rings that have reached their maximum radius.
pub(crate) fn despawn_finished_pulse_ring(
    mut commands: Commands,
    rings: Query<(Entity, &PulseRadius, &PulseMaxRadius), With<PulseRing>>,
) {
    for (entity, radius, max_radius) in &rings {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn register(app: &mut App) {
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
