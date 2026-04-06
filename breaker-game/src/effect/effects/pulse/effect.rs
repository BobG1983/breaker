use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::{
    collision_layers::CollisionLayers, plugin::PhysicsSystems, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial};

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    cells::messages::DamageCell,
    effect::{core::EffectSourceChip, effects::damage_boost::ActiveDamageBoosts},
    shared::{CELL_LAYER, GameDrawLayer},
    state::types::NodeState,
};

/// Placeholder pulse ring color — HDR teal.
const PULSE_COLOR: Color = Color::linear_rgb(0.5, 3.0, 4.0);

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
        crate::effect::effects::effective_range(self.base_range, self.range_per_level, self.stacks)
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
/// `ActiveDamageBoosts` at ring-spawn time. Default `1.0`.
#[derive(Component)]
pub(crate) struct PulseRingDamageMultiplier(pub(crate) f32);

/// Base damage snapshotted from the emitter's bolt entity's `BoltBaseDamage` at
/// ring-spawn time. Falls back to `DEFAULT_BOLT_BASE_DAMAGE` if the bolt has no
/// `BoltBaseDamage`.
#[derive(Component)]
pub(crate) struct PulseRingBaseDamage(pub(crate) f32);

/// Query data for [`tick_pulse_emitter`].
type EmitterQuery = (
    Entity,
    &'static mut PulseEmitter,
    &'static Position2D,
    Option<&'static ActiveDamageBoosts>,
    Option<&'static EffectSourceChip>,
    Option<&'static BoltBaseDamage>,
);

/// Query data for [`apply_pulse_damage`].
type PulseDamageQuery = (
    &'static Position2D,
    &'static PulseRadius,
    &'static mut PulseDamaged,
    Option<&'static PulseRingDamageMultiplier>,
    Option<&'static PulseRingBaseDamage>,
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
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<ColorMaterial>>>,
) {
    let dt = time.timestep().as_secs_f32();
    for (_entity, mut emitter, position, active_boosts, esc, bolt_base_damage) in &mut emitters {
        emitter.timer += dt;
        if emitter.timer >= emitter.interval {
            emitter.timer -= emitter.interval;
            let effective_range = emitter.effective_max_radius();
            let speed = emitter.speed;
            let damage_multiplier = active_boosts.map_or(1.0, ActiveDamageBoosts::multiplier);
            let base_dmg = bolt_base_damage.map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
            let emitter_pos = position.0;
            let mut ring = commands.spawn((
                PulseRing,
                PulseSource,
                PulseRadius(0.0),
                PulseMaxRadius(effective_range),
                PulseSpeed(speed),
                PulseDamaged(HashSet::new()),
                PulseRingDamageMultiplier(damage_multiplier),
                PulseRingBaseDamage(base_dmg),
                Spatial::builder().at_position(emitter_pos).build(),
                Scale2D { x: 0.0, y: 0.0 },
                CleanupOnExit::<NodeState>::default(),
            ));
            if let (Some(m), Some(mat)) = (meshes.as_mut(), materials.as_mut()) {
                ring.insert((
                    Mesh2d(m.add(Circle::new(1.0))),
                    MeshMaterial2d(mat.add(ColorMaterial::from_color(PULSE_COLOR))),
                    GameDrawLayer::Fx,
                ));
            }
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
    for (position, radius, mut damaged, damage_mult, pulse_base_damage, esc) in &mut rings {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = position.0;
        let multiplier = damage_mult.map_or(1.0, |m| m.0);
        let base_damage = pulse_base_damage.map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
        let source_chip = esc.and_then(EffectSourceChip::source_chip);
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(center, radius.0, query_layers);
        for cell in candidates {
            if damaged.0.insert(cell) {
                damage_writer.write(DamageCell {
                    cell,
                    damage: base_damage * multiplier,
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

/// Syncs `Scale2D` to match `PulseRadius` each tick so the visual mesh
/// tracks the expanding pulse ring.
pub(crate) fn sync_pulse_visual(mut query: Query<(&PulseRadius, &mut Scale2D), With<PulseRing>>) {
    for (radius, mut scale) in &mut query {
        scale.x = radius.0;
        scale.y = radius.0;
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_pulse_emitter,
            tick_pulse_ring,
            sync_pulse_visual,
            apply_pulse_damage,
            despawn_finished_pulse_ring,
        )
            .chain()
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(NodeState::Playing)),
    );
}
