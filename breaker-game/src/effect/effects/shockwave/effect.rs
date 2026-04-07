use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;
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

/// Marker component for shockwave entities.
/// Placeholder shockwave color — HDR orange.
const SHOCKWAVE_COLOR: Color = Color::linear_rgb(4.0, 1.5, 0.2);

#[derive(Component)]
pub(crate) struct ShockwaveSource;

/// Current radius of the expanding shockwave.
#[derive(Component)]
pub(crate) struct ShockwaveRadius(pub(crate) f32);

/// Maximum radius the shockwave expands to before despawning.
#[derive(Component)]
pub(crate) struct ShockwaveMaxRadius(pub(crate) f32);

/// Expansion speed of the shockwave in world units per second.
#[derive(Component)]
pub(crate) struct ShockwaveSpeed(pub(crate) f32);

/// Tracks which cell entities have already been damaged by this specific
/// shockwave instance to enforce at-most-once damage.
#[derive(Component, Default)]
pub(crate) struct ShockwaveDamaged(pub(crate) HashSet<Entity>);

/// Damage multiplier snapshotted from the source entity's
/// `ActiveDamageBoosts` at fire-time. Default `1.0`.
#[derive(Component)]
pub(crate) struct ShockwaveDamageMultiplier(pub(crate) f32);

/// Base damage snapshotted from the source entity's `BoltBaseDamage` at fire-time.
/// Falls back to `DEFAULT_BOLT_BASE_DAMAGE` if the source has no `BoltBaseDamage`.
#[derive(Component)]
pub(crate) struct ShockwaveBaseDamage(pub(crate) f32);

/// Query data for [`apply_shockwave_damage`].
type ShockwaveDamageQuery = (
    &'static Position2D,
    &'static ShockwaveRadius,
    &'static mut ShockwaveDamaged,
    Option<&'static ShockwaveDamageMultiplier>,
    Option<&'static ShockwaveBaseDamage>,
    Option<&'static EffectSourceChip>,
);

pub(crate) fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    source_chip: &str,
    world: &mut World,
) {
    let effective_range =
        crate::effect::effects::effective_range(base_range, range_per_level, stacks);

    let position = crate::effect::effects::entity_position(world, entity);

    let edm = world
        .get::<ActiveDamageBoosts>(entity)
        .map_or(1.0, ActiveDamageBoosts::multiplier);

    let base_damage = world
        .get::<BoltBaseDamage>(entity)
        .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

    let visual = {
        let mesh = world
            .get_resource_mut::<Assets<Mesh>>()
            .map(|mut m| m.add(Circle::new(1.0)));
        let mat = world
            .get_resource_mut::<Assets<ColorMaterial>>()
            .map(|mut m| m.add(ColorMaterial::from_color(SHOCKWAVE_COLOR)));
        mesh.zip(mat)
    };

    let mut entity = world.spawn((
        ShockwaveSource,
        ShockwaveRadius(0.0),
        ShockwaveMaxRadius(effective_range),
        ShockwaveSpeed(speed),
        ShockwaveDamaged::default(),
        ShockwaveDamageMultiplier(edm),
        ShockwaveBaseDamage(base_damage),
        EffectSourceChip::new(source_chip),
        Spatial::builder().at_position(position).build(),
        Scale2D { x: 0.0, y: 0.0 },
        GameDrawLayer::Fx,
        CleanupOnExit::<NodeState>::default(),
    ));
    if let Some((mesh, mat)) = visual {
        entity.insert((Mesh2d(mesh), MeshMaterial2d(mat)));
    }
}

pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Expand shockwave radius by speed * delta time each fixed tick.
pub(crate) fn tick_shockwave(
    time: Res<Time>,
    mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>,
) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Despawn shockwaves that have reached their maximum radius.
pub(crate) fn despawn_finished_shockwave(
    mut commands: Commands,
    query: Query<(Entity, &ShockwaveRadius, &ShockwaveMaxRadius)>,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Damage cells within the expanding shockwave ring.
///
/// For each shockwave, queries the quadtree for cells within the current radius
/// and sends [`DamageCell`] for any cell not already in the [`ShockwaveDamaged`] set.
pub(crate) fn apply_shockwave_damage(
    quadtree: Res<CollisionQuadtree>,
    mut shockwaves: Query<ShockwaveDamageQuery, With<ShockwaveSource>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    for (position, radius, mut damaged, damage_mult, shockwave_base_damage, esc) in &mut shockwaves
    {
        if radius.0 <= 0.0 {
            continue;
        }
        let center = position.0;
        let multiplier = damage_mult.map_or(1.0, |m| m.0);
        let base_damage = shockwave_base_damage.map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
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

/// Syncs `Scale2D` to match `ShockwaveRadius` each tick so the visual mesh
/// tracks the expanding shockwave.
pub(crate) fn sync_shockwave_visual(
    mut query: Query<(&ShockwaveRadius, &mut Scale2D), With<ShockwaveSource>>,
) {
    for (radius, mut scale) in &mut query {
        scale.x = radius.0;
        scale.y = radius.0;
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            tick_shockwave,
            sync_shockwave_visual,
            apply_shockwave_damage,
            despawn_finished_shockwave,
        )
            .chain()
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(NodeState::Playing)),
    );
}
