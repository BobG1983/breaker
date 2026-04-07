//! Fast-expanding beam rectangle in the bolt's velocity direction.

use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::PhysicsSystems,
    resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Rotation2D, Scale2D, Spatial, Velocity2D};

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    cells::messages::DamageCell,
    effect::{core::EffectSourceChip, effects::damage_boost::ActiveDamageBoosts},
    fx::EffectFlashTimer,
    shared::{CELL_LAYER, GameDrawLayer, PlayfieldConfig},
    state::types::NodeState,
};

/// Placeholder beam flash color — HDR white-yellow.
const BEAM_FLASH_COLOR: Color = Color::linear_rgb(5.0, 5.0, 2.0);
/// Placeholder beam flash duration in seconds.
const BEAM_FLASH_DURATION: f32 = 0.15;

/// Deferred request for piercing beam instant damage.
///
/// Spawned by `fire()` with pre-computed beam geometry,
/// consumed (and despawned) by `process_piercing_beam` in the same or next tick.
#[derive(Component)]
pub(crate) struct PiercingBeamRequest {
    /// Beam origin (entity position).
    pub origin: Vec2,
    /// Normalized beam direction.
    pub direction: Vec2,
    /// Beam length (distance to playfield boundary).
    pub length: f32,
    /// Beam half-width.
    pub half_width: f32,
    /// Pre-calculated damage per cell.
    pub damage: f32,
}

pub(crate) fn fire(
    entity: Entity,
    damage_mult: f32,
    width: f32,
    source_chip: &str,
    world: &mut World,
) {
    let pos = crate::effect::effects::entity_position(world, entity);

    let velocity = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);

    let mut dir = velocity.normalize_or_zero();
    if dir == Vec2::ZERO {
        dir = Vec2::Y;
    }
    let playfield = world.resource::<PlayfieldConfig>();

    // Compute distance to each playfield boundary along the beam direction.
    let mut min_t = f32::MAX;
    if dir.x > f32::EPSILON {
        min_t = min_t.min((playfield.right() - pos.x) / dir.x);
    } else if dir.x < -f32::EPSILON {
        min_t = min_t.min((playfield.left() - pos.x) / dir.x);
    }
    if dir.y > f32::EPSILON {
        min_t = min_t.min((playfield.top() - pos.y) / dir.y);
    } else if dir.y < -f32::EPSILON {
        min_t = min_t.min((playfield.bottom() - pos.y) / dir.y);
    }
    let beam_length = min_t.max(0.0);

    let edm = world
        .get::<ActiveDamageBoosts>(entity)
        .map_or(1.0, ActiveDamageBoosts::multiplier);

    let base_damage = world
        .get::<BoltBaseDamage>(entity)
        .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

    world.spawn((
        PiercingBeamRequest {
            origin: pos,
            direction: dir,
            length: beam_length,
            half_width: width / 2.0,
            damage: base_damage * damage_mult * edm,
        },
        EffectSourceChip::new(source_chip),
        CleanupOnExit::<NodeState>::default(),
    ));
}

pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Process all pending piercing beam requests: query quadtree, send damage, despawn request.
///
/// For each request, constructs the beam's bounding AABB, queries the quadtree
/// for candidate cells, performs narrow-phase filtering against the oriented beam
/// rectangle, sends [`DamageCell`] for each intersecting cell, then despawns the
/// request entity.
pub(crate) fn process_piercing_beam(
    mut commands: Commands,
    requests: Query<(Entity, &PiercingBeamRequest, Option<&EffectSourceChip>)>,
    quadtree: Res<CollisionQuadtree>,
    positions: Query<&GlobalPosition2D>,
    mut damage_writer: MessageWriter<DamageCell>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<ColorMaterial>>>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);

    for (entity, request, esc) in &requests {
        let dir = request.direction;
        let origin = request.origin;
        let length = request.length;
        let hw = request.half_width;

        // Compute the beam's oriented bounding box as an AABB for broad-phase.
        let end = origin + dir * length;
        let perp = Vec2::new(-dir.y, dir.x);

        let corners = [
            origin + perp * hw,
            origin - perp * hw,
            end + perp * hw,
            end - perp * hw,
        ];

        let min_x = corners.iter().map(|c| c.x).fold(f32::INFINITY, f32::min);
        let max_x = corners
            .iter()
            .map(|c| c.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = corners.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
        let max_y = corners
            .iter()
            .map(|c| c.y)
            .fold(f32::NEG_INFINITY, f32::max);

        let bounding_aabb = Aabb2D::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));

        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&bounding_aabb, query_layers);

        // Narrow-phase: check each candidate against the oriented beam rectangle.
        for cell in candidates {
            let cell_pos = positions.get(cell).map_or(Vec2::ZERO, |p| p.0);

            let to_cell = cell_pos - origin;
            let along = to_cell.dot(dir);
            if along < 0.0 || along > length {
                continue;
            }
            let perp_dist = (to_cell - dir * along).length();
            if perp_dist > hw {
                continue;
            }

            damage_writer.write(DamageCell {
                cell,
                damage: request.damage,
                source_chip: esc.and_then(EffectSourceChip::source_chip),
            });
        }

        // Spawn flash visual entity for beams with non-trivial length.
        if length > f32::EPSILON
            && let (Some(meshes), Some(materials)) = (meshes.as_mut(), materials.as_mut())
        {
            let beam_midpoint = origin + dir * (length / 2.0);
            let beam_width = hw * 2.0;
            let beam_angle = dir.y.atan2(dir.x);

            commands.spawn((
                Spatial::builder().at_position(beam_midpoint).build(),
                Scale2D {
                    x: length,
                    y: beam_width,
                },
                Rotation2D::from_radians(beam_angle),
                GameDrawLayer::Fx,
                Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(BEAM_FLASH_COLOR))),
                EffectFlashTimer(BEAM_FLASH_DURATION),
                CleanupOnExit::<NodeState>::default(),
            ));
        }

        commands.entity(entity).despawn();
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        process_piercing_beam
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(NodeState::Playing)),
    );
}
