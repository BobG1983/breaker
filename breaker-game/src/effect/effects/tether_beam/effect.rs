//! Two free-moving bolts connected by a crackling neon beam that damages intersected cells.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, ccd::ray_vs_aabb, collision_layers::CollisionLayers, plugin::PhysicsSystems,
    resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D};

use crate::{
    bolt::{BASE_BOLT_DAMAGE, components::Bolt, resources::BoltConfig},
    cells::{components::Cell, messages::DamageCell},
    effect::{
        EffectiveDamageMultiplier,
        core::{EffectSourceChip, chip_attribution},
    },
    shared::{CELL_LAYER, CleanupOnNodeExit, playing_state::PlayingState},
};

/// Marker on a tether bolt entity, pointing to its beam entity.
#[derive(Component)]
pub struct TetherBoltMarker(pub Entity);

/// The beam entity linking two tether bolts.
#[derive(Component)]
pub struct TetherBeamComponent {
    /// First tether bolt entity.
    pub bolt_a: Entity,
    /// Second tether bolt entity.
    pub bolt_b: Entity,
    /// Damage multiplier applied to `BASE_BOLT_DAMAGE`.
    pub damage_mult: f32,
    /// Effective damage multiplier snapshotted from the source entity's
    /// `EffectiveDamageMultiplier` at fire-time. Default `1.0`.
    pub effective_damage_multiplier: f32,
}

/// Spawns two tethered bolts with a damaging beam between them.
///
/// Evolution of `ChainBolt`. The beam is a line segment between the two bolt
/// positions — cells intersecting the beam take damage each tick.
pub(crate) fn fire(entity: Entity, damage_mult: f32, source_chip: &str, world: &mut World) {
    let spawn_pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

    let bolt_a = super::super::spawn_extra_bolt(world, spawn_pos);
    let bolt_b = super::super::spawn_extra_bolt(world, spawn_pos);

    let edm = world
        .get::<EffectiveDamageMultiplier>(entity)
        .map_or(1.0, |e| e.0);

    // Spawn the beam entity linking both bolts
    let beam = world
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult,
                effective_damage_multiplier: edm,
            },
            EffectSourceChip(chip_attribution(source_chip)),
            CleanupOnNodeExit,
        ))
        .id();

    // Add TetherBoltMarker to each bolt, pointing to the beam
    world.entity_mut(bolt_a).insert(TetherBoltMarker(beam));
    world.entity_mut(bolt_b).insert(TetherBoltMarker(beam));
}

/// No-op — tether bolts have their own lifecycle.
pub(crate) fn reverse(_entity: Entity, _damage_mult: f32, _source_chip: &str, _world: &mut World) {}

/// Tick system: damages cells whose AABB intersects each tether beam segment.
///
/// For each beam, looks up the positions of `bolt_a` and `bolt_b`. If either bolt
/// is missing, despawns the beam. Otherwise, computes broadphase via quadtree
/// AABB query and narrowphase via ray-vs-AABB intersection, sending `DamageCell`
/// for each cell hit by the beam segment.
///
/// The beam has an effective half-width equal to the bolt radius (from
/// `BoltConfig`), so cells whose AABBs are within the bolt radius of the beam
/// line segment are considered intersecting.
pub fn tick_tether_beam(
    mut commands: Commands,
    beams: Query<(Entity, &TetherBeamComponent, Option<&EffectSourceChip>)>,
    bolt_positions: Query<&Position2D, With<Bolt>>,
    quadtree: Res<CollisionQuadtree>,
    cell_aabbs: Query<(&Aabb2D, &GlobalPosition2D), With<Cell>>,
    bolt_config: Option<Res<BoltConfig>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    let beam_half_width = bolt_config
        .as_ref()
        .map_or(BoltConfig::default().radius, |c| c.radius);

    for (beam_entity, component, esc) in &beams {
        // Look up both bolt positions; despawn beam if either is missing
        let pos_a = if let Ok(p) = bolt_positions.get(component.bolt_a) {
            p.0
        } else {
            commands.entity(beam_entity).despawn();
            continue;
        };
        let pos_b = if let Ok(p) = bolt_positions.get(component.bolt_b) {
            p.0
        } else {
            commands.entity(beam_entity).despawn();
            continue;
        };

        // Broadphase: compute beam bounding box expanded by beam half-width and
        // query quadtree. The expansion ensures cells near (but not exactly on)
        // the beam line are included as candidates.
        let beam_aabb =
            Aabb2D::from_min_max(pos_a.min(pos_b), pos_a.max(pos_b)).expand_by(beam_half_width);
        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&beam_aabb, query_layers);

        // Narrowphase: test line-segment vs cell AABB intersection
        let beam_vec = pos_b - pos_a;
        let max_dist = beam_vec.length();
        let direction = beam_vec.normalize_or_zero();
        let damage =
            BASE_BOLT_DAMAGE * component.damage_mult * component.effective_damage_multiplier;

        let mut damaged_this_tick: HashSet<Entity> = HashSet::new();

        for cell in candidates {
            if damaged_this_tick.contains(&cell) {
                continue;
            }

            let Ok((local_aabb, global_pos)) = cell_aabbs.get(cell) else {
                continue;
            };

            // Compute world-space AABB for the cell, expanded by the beam
            // half-width (Minkowski sum) so a point-ray test is equivalent to a
            // thick-beam-vs-AABB test.
            let world_aabb = Aabb2D::new(global_pos.0 + local_aabb.center, local_aabb.half_extents)
                .expand_by(beam_half_width);

            // Check ray intersection OR origin-inside-AABB
            let ray_hit = ray_vs_aabb(pos_a, direction, max_dist, &world_aabb);
            let origin_inside = world_aabb.contains_point(pos_a);

            if ray_hit.is_some() || origin_inside {
                damaged_this_tick.insert(cell);
                damage_writer.write(DamageCell {
                    cell,
                    damage,
                    source_chip: esc.and_then(|e| e.0.clone()),
                });
            }
        }
    }
}

/// Registers systems for `TetherBeam` effect.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_tether_beam
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}
