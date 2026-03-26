//! Bolt-cell-wall collision detection via swept CCD (continuous collision detection).
//!
//! Instead of moving the bolt first and then checking for overlaps, this system
//! traces the bolt's path forward using ray-vs-expanded-AABB intersection.
//! On each hit, the bolt is placed just before the impact point, the velocity
//! is reflected, and the remaining movement continues. The bolt never overlaps
//! any cell or wall.
//!
//! A per-frame `MAX_BOUNCES` cap (4) prevents infinite bounce loops. Cell hits
//! are naturally bounded: after reflection, the bolt travels away from the hit
//! surface for the remainder of the frame budget.
//!
//! Piercing bolts (`PiercingRemaining > 0`) pass through cells without
//! reflecting, decrementing `PiercingRemaining` on each hit.
//!
//! Cell damage and destruction are handled by the cells domain via
//! [`BoltHitCell`] and [`DamageCell`] messages. Wall hits send
//! [`BoltHitWall`] messages for overclock triggers.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::{
        components::Bolt,
        filters::ActiveFilter,
        messages::{BoltHitCell, BoltHitWall},
        queries::CollisionQueryBolt,
    },
    cells::{
        components::{Cell, CellHealth},
        messages::DamageCell,
    },
    shared::{
        BASE_BOLT_DAMAGE, CELL_LAYER, WALL_LAYER,
        math::{CCD_EPSILON, MAX_BOUNCES, ray_vs_aabb},
    },
    wall::components::Wall,
};

/// Message writers used by the bolt-cell-wall collision system.
type CollisionWriters<'a> = (
    MessageWriter<'a, BoltHitCell>,
    MessageWriter<'a, DamageCell>,
    MessageWriter<'a, BoltHitWall>,
);

/// Query for looking up entity `Aabb2D` and `Position2D` by entity ID
/// after the quadtree broad-phase returns candidate entities.
///
/// Excludes bolts to avoid query conflicts with the mutable `bolt_query`.
type CandidateLookup<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static Aabb2D,
        Has<Cell>,
        Has<Wall>,
        Option<&'static CellHealth>,
    ),
    Without<Bolt>,
>;

/// Finds the nearest collision candidate from a set of quadtree results.
///
/// Returns `Some((cell_entity_or_none, candidate_entity, hit))` for the closest
/// hit, or `None` if no candidate was hit. Cell entities return
/// `Some(entity)` in the first field, walls return `None`. The second field
/// always holds the raw candidate entity (cell or wall).
fn find_nearest_candidate(
    candidates: &[Entity],
    candidate_lookup: &CandidateLookup,
    pierced_this_frame: &[Entity],
    position: Vec2,
    direction: Vec2,
    remaining: f32,
    bolt_radius: f32,
) -> Option<(Option<Entity>, Entity, crate::shared::math::RayHit)> {
    let mut best: Option<(Option<Entity>, Entity, crate::shared::math::RayHit)> = None;

    for candidate_entity in candidates {
        let Ok((_, candidate_pos, candidate_aabb, is_cell, _is_wall, _cell_health)) =
            candidate_lookup.get(*candidate_entity)
        else {
            continue;
        };

        if is_cell && pierced_this_frame.contains(candidate_entity) {
            continue;
        }

        let expanded_half_extents = candidate_aabb.half_extents + Vec2::splat(bolt_radius);
        if let Some(hit) = ray_vs_aabb(
            position,
            direction,
            remaining,
            candidate_pos.0,
            expanded_half_extents,
        ) && best
            .as_ref()
            .is_none_or(|(_, _, b)| hit.distance < b.distance)
        {
            let hit_entity = if is_cell {
                Some(*candidate_entity)
            } else {
                None
            };
            best = Some((hit_entity, *candidate_entity, hit));
        }
    }

    best
}

/// Advances bolts along their velocity, reflecting off cells and walls via swept CCD.
///
/// For each bolt, traces a ray from its current position in the velocity
/// direction. If a cell or wall is hit, the bolt is placed just before the
/// impact point, the velocity is reflected off the hit face, and tracing
/// continues with the remaining movement distance. Sends [`BoltHitCell`]
/// and [`DamageCell`] messages for each cell hit. Sends [`BoltHitWall`]
/// messages for each wall hit.
pub(crate) fn bolt_cell_collision(
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    mut bolt_query: Query<CollisionQueryBolt, ActiveFilter>,
    candidate_lookup: CandidateLookup,
    mut writers: CollisionWriters,
    mut pierced_this_frame: Local<Vec<Entity>>,
) {
    let (ref mut hit_writer, ref mut damage_writer, ref mut wall_hit_writer) = writers;
    let dt = time.delta_secs();

    for (
        bolt_entity,
        mut bolt_position,
        mut bolt_vel,
        _,
        bolt_radius,
        mut piercing_remaining,
        piercing,
        damage_boost,
        bolt_entity_scale,
        spawned_by_evo,
    ) in &mut bolt_query
    {
        let bolt_scale = bolt_entity_scale.map_or(1.0, |s| s.0);
        let r = bolt_radius.0 * bolt_scale;
        let mut position = bolt_position.0;
        let mut velocity = bolt_vel.0;
        let mut remaining = velocity.length() * dt;

        // Effective damage for pierce lookahead (compared against cell HP).
        // must match `handle_cell_hit` damage formula
        let effective_damage = BASE_BOLT_DAMAGE * (1.0 + damage_boost.map_or(0.0, |b| b.0));

        // Clear per-bolt pierce skip set
        pierced_this_frame.clear();

        for _ in 0..MAX_BOUNCES {
            if remaining <= CCD_EPSILON {
                break;
            }

            let direction = velocity.normalize_or_zero();
            if direction == Vec2::ZERO {
                break;
            }

            // Compute swept AABB for this bounce step
            let end_pos = position + direction * remaining;
            let sweep_min = position.min(end_pos) - Vec2::splat(r);
            let sweep_max = position.max(end_pos) + Vec2::splat(r);
            let swept_aabb = Aabb2D::from_min_max(sweep_min, sweep_max);

            // Query quadtree for candidate cells and walls
            let candidates = quadtree.quadtree.query_aabb_filtered(
                &swept_aabb,
                CollisionLayers::new(0, CELL_LAYER | WALL_LAYER),
            );

            let best = find_nearest_candidate(
                &candidates,
                &candidate_lookup,
                &pierced_this_frame,
                position,
                direction,
                remaining,
                r,
            );

            let Some((hit_cell, candidate_entity, hit)) = best else {
                // No target in path — move the full remaining distance
                position += direction * remaining;
                break;
            };

            // Advance to the impact point — shared by all hit outcomes
            let advance = (hit.distance - CCD_EPSILON).max(0.0);
            position += direction * advance;
            remaining -= advance;

            if let Some(cell_entity) = hit_cell {
                // Check if this bolt can pierce this cell
                let can_pierce = piercing_remaining.as_deref().is_some_and(|pr| pr.0 > 0);
                let cell_hp = candidate_lookup
                    .get(cell_entity)
                    .ok()
                    .and_then(|(_, _, _, _, _, health)| health)
                    .map(|h| h.current);
                let would_destroy = cell_hp.is_some_and(|hp| hp <= effective_damage);

                if can_pierce && would_destroy {
                    // PIERCE: do NOT reflect; decrement remaining pierces
                    if let Some(ref mut pr) = piercing_remaining {
                        pr.0 = pr.0.saturating_sub(1);
                    }
                    pierced_this_frame.push(cell_entity);
                    // Continue CCD loop — velocity unchanged, direction unchanged
                } else {
                    // NORMAL: reflect
                    velocity -= 2.0 * velocity.dot(hit.normal) * hit.normal;
                }
                hit_writer.write(BoltHitCell {
                    cell: cell_entity,
                    bolt: bolt_entity,
                });
                damage_writer.write(DamageCell {
                    cell: cell_entity,
                    damage: effective_damage,
                    source_bolt: Some(bolt_entity),
                    source_chip: spawned_by_evo.map(|s| s.0.clone()),
                });
            } else {
                // WALL HIT: reflect and reset `PiercingRemaining`
                velocity -= 2.0 * velocity.dot(hit.normal) * hit.normal;
                // Reset `PiercingRemaining` to `Piercing.0`
                if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
                    pr.0 = p.0;
                }
                wall_hit_writer.write(BoltHitWall {
                    bolt: bolt_entity,
                    wall: candidate_entity,
                });
            }
        }

        bolt_position.0 = position;
        bolt_vel.0 = velocity;
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::{
            components::{Bolt, BoltBaseSpeed, BoltRadius, BoltServing, SpawnedByEvolution},
            messages::BoltHitWall,
            resources::BoltConfig,
        },
        cells::{
            components::{Cell, CellHealth, CellHeight, CellWidth},
            messages::DamageCell,
            resources::CellConfig,
        },
        chips::components::{DamageBoost, Piercing, PiercingRemaining},
        shared::{
            BOLT_LAYER, CELL_LAYER, EntityScale, GameDrawLayer, WALL_LAYER, math::MAX_BOUNCES,
        },
        wall::components::{Wall, WallSize},
    };

    // --- CCD system tests ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RantzPhysics2dPlugin)
            .add_message::<BoltHitCell>()
            .add_message::<DamageCell>()
            .add_message::<BoltHitWall>()
            .add_systems(
                FixedUpdate,
                bolt_cell_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            );
        app
    }

    fn bolt_param_bundle() -> (BoltBaseSpeed, BoltRadius) {
        let bc = BoltConfig::default();
        (BoltBaseSpeed(bc.base_speed), BoltRadius(bc.radius))
    }

    fn default_cell_dims() -> (CellWidth, CellHeight) {
        let cc = CellConfig::default();
        (CellWidth(cc.width), CellHeight(cc.height))
    }

    /// Accumulates one fixed timestep of overstep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Cell entities use `Position2D` as canonical position.
    fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
        let (cw, ch) = default_cell_dims();
        let half_extents = Vec2::new(cw.half_width(), ch.half_height());
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                cw,
                ch,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    #[test]
    fn bolt_moves_full_distance_no_cells() {
        let mut app = test_app();
        let dt = app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f32();

        let start_y = 0.0;
        let speed = 400.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, speed)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let pos = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = speed.mul_add(dt, start_y);
        assert!(
            (pos.0.y - expected).abs() < 0.1,
            "bolt should move full distance: expected {expected}, got {}",
            pos.0.y
        );
    }

    #[test]
    fn bolt_reflects_off_cell_bottom() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        // Place bolt below the cell's expanded AABB, moving upward
        let start_y = cell_y - cc.height / 2.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y < 0.0,
            "bolt should reflect downward, got vy={}",
            vel.0.y
        );

        let pos = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        let cell_bottom = cell_y - cc.height / 2.0 - bc.radius;
        assert!(
            pos.0.y < cell_bottom,
            "bolt should be below cell: y={:.2}, cell_bottom={cell_bottom:.2}",
            pos.0.y
        );
    }

    #[test]
    fn bolt_reflects_off_cell_side() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_x = 100.0;
        spawn_cell(&mut app, cell_x, 0.0);

        // Place bolt left of cell, moving right
        let start_x = cell_x - cc.width / 2.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)), // mostly horizontal
            Position2D(Vec2::new(start_x, 0.0)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.x < 0.0,
            "bolt should reflect leftward, got vx={}",
            vel.0.x
        );
    }

    #[test]
    fn bolt_uses_remaining_distance_after_bounce() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        // Place bolt just 1 unit below the expanded AABB bottom, moving up fast.
        // It will hit quickly and have most of its movement remaining.
        let cell_bottom = cell_y - cc.height / 2.0 - bc.radius;
        let start_y = cell_bottom - 1.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let pos = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();

        // Bolt should NOT be sitting right at the impact point -- it should have
        // continued downward with the remaining distance after reflection
        assert!(
            pos.0.y < start_y,
            "bolt should have moved past the impact point in reflected direction, \
             got y={:.2}, start={start_y:.2}",
            pos.0.y
        );
    }

    #[test]
    fn bolt_hits_only_nearest_cell() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        // Two cells vertically, bolt path crosses both
        let near_y = 50.0;
        let far_y = 100.0;
        let near_cell = spawn_cell(&mut app, 0.0, near_y);
        spawn_cell(&mut app, 0.0, far_y);

        let start_y = near_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        // Only the nearer cell should be hit (bolt reflects before reaching the far one)
        assert_eq!(hits.0.len(), 1, "should hit exactly one cell");
        assert_eq!(hits.0[0], near_cell, "should hit the nearer cell");
    }

    /// Collects `BoltHitCell` messages into a resource for test assertions.
    #[derive(Resource, Default)]
    struct HitCells(Vec<Entity>);

    fn collect_cell_hits(mut reader: MessageReader<BoltHitCell>, mut hits: ResMut<HitCells>) {
        for msg in reader.read() {
            hits.0.push(msg.cell);
        }
    }

    /// Collects full `BoltHitCell` messages (including the bolt field) for assertion.
    #[derive(Resource, Default)]
    struct FullHitMessages(Vec<BoltHitCell>);

    fn collect_full_hits(
        mut reader: MessageReader<BoltHitCell>,
        mut hits: ResMut<FullHitMessages>,
    ) {
        for msg in reader.read() {
            hits.0.push(msg.clone());
        }
    }

    #[test]
    fn bolt_hit_cell_message_sent() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let cell_y = 100.0;
        let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 1, "should send exactly one hit message");
        assert_eq!(
            hits.0[0], cell_entity,
            "hit message should reference the correct cell"
        );
    }

    #[test]
    fn no_collision_when_far_away() {
        let mut app = test_app();

        spawn_cell(&mut app, 0.0, 200.0);

        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 300.0)),
            Position2D(Vec2::new(0.0, -100.0)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.0.y > 0.0, "bolt should still move upward");
    }

    // --- Cascade prevention tests ---

    /// Real grid vertical spacing: `cell_height` (24) + padding (4) = 28
    const GRID_STEP_Y: f32 = 28.0;
    /// Real grid horizontal spacing: `cell_width` (70) + padding (4) = 74
    const GRID_STEP_X: f32 = 74.0;

    #[test]
    fn vertical_adjacent_cells_no_cascade() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let upper_y = 100.0;
        let lower_y = upper_y - GRID_STEP_Y;
        spawn_cell(&mut app, 0.0, upper_y);
        spawn_cell(&mut app, 0.0, lower_y);

        // Bolt below the upper cell, moving up
        let start_y = upper_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        // Two frames -- CCD should prevent cascade
        tick(&mut app);
        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "bolt should hit only one cell across two frames, not cascade (got {} hits)",
            hits.0.len()
        );
    }

    #[test]
    fn horizontal_adjacent_cells_no_cascade() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let left_x = 0.0;
        let right_x = left_x + GRID_STEP_X;
        let cell_y = 100.0;
        spawn_cell(&mut app, left_x, cell_y);
        spawn_cell(&mut app, right_x, cell_y);

        // Bolt left of right cell, moving right
        let start_x = right_x - cc.width / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 10.0)),
            Position2D(Vec2::new(start_x, cell_y)),
        ));

        tick(&mut app);
        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "bolt should hit only one cell across two frames, not cascade (got {} hits)",
            hits.0.len()
        );
    }

    #[test]
    fn grid_entry_from_below_hits_one_cell() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        // 3x2 mini-grid at real spacing
        let base_y = 100.0;
        for row in 0..2 {
            for col in 0..3 {
                let x = (f32::from(i16::try_from(col).unwrap_or(0)) - 1.0) * GRID_STEP_X;
                let y = f32::from(i16::try_from(row).unwrap_or(0)).mul_add(GRID_STEP_Y, base_y);
                spawn_cell(&mut app, x, y);
            }
        }

        let start_y = base_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(30.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);
        tick(&mut app);
        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "bolt entering grid should hit exactly 1 cell across 3 frames, not cascade (got {})",
            hits.0.len()
        );
    }

    // --- Edge case tests ---

    #[test]
    fn max_bounces_cap() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        // Two cells very close together creating a narrow channel.
        // Bolt bouncing between them could loop forever without the cap.
        let gap = bc.radius.mul_add(2.0, 2.0); // just wider than bolt diameter
        spawn_cell(&mut app, 0.0, gap / 2.0 + cc.height / 2.0 + bc.radius);
        spawn_cell(&mut app, 0.0, -(gap / 2.0 + cc.height / 2.0 + bc.radius));

        // Bolt in the channel, moving up very fast
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.1, 800.0)),
            Position2D(Vec2::new(0.0, 0.0)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert!(
            u32::try_from(hits.0.len()).unwrap_or(0) <= MAX_BOUNCES,
            "should not exceed MAX_BOUNCES ({MAX_BOUNCES}), got {} hits",
            hits.0.len()
        );
    }

    #[test]
    fn multiple_bolts_each_hit_different_cells() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let cell_a = spawn_cell(&mut app, -100.0, 100.0);
        let cell_b = spawn_cell(&mut app, 100.0, 100.0);

        let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

        // Bolt A near cell A
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(-100.0, start_y)),
        ));
        // Bolt B near cell B
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(100.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 2, "both bolts should register hits");
        assert!(hits.0.contains(&cell_a), "cell A should be hit");
        assert!(hits.0.contains(&cell_b), "cell B should be hit");
    }

    #[test]
    fn serving_bolt_is_not_advanced() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(0.0, 0.0)),
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert!(
            pos.0.y.abs() < f32::EPSILON,
            "serving bolt should not be moved by CCD, got y={}",
            pos.0.y
        );
    }

    // --- BoltHitCell bolt entity tests ---

    #[test]
    fn bolt_cell_collision_populates_bolt_entity_in_message() {
        // This test verifies that BoltHitCell.bolt is set to the actual bolt entity,
        // not Entity::PLACEHOLDER. It will FAIL until the production code is fixed
        // to capture the bolt entity from the query binding.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(FullHitMessages::default())
            .add_systems(FixedUpdate, collect_full_hits.after(bolt_cell_collision));

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let hits = app.world().resource::<FullHitMessages>();
        assert_eq!(
            hits.0.len(),
            1,
            "should send exactly one BoltHitCell message"
        );
        assert_ne!(
            hits.0[0].bolt,
            Entity::PLACEHOLDER,
            "BoltHitCell.bolt should not be Entity::PLACEHOLDER — it should be the real bolt entity"
        );
        assert_eq!(
            hits.0[0].bolt, bolt_entity,
            "BoltHitCell.bolt should equal the bolt entity that caused the collision"
        );
    }

    // --- Wall collision tests ---

    fn spawn_wall(app: &mut App, x: f32, y: f32, half_width: f32, half_height: f32) {
        let pos = Vec2::new(x, y);
        app.world_mut().spawn((
            Wall,
            WallSize {
                half_width,
                half_height,
            },
            Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Wall,
        ));
    }

    #[test]
    fn bolt_reflects_off_wall() {
        let mut app = test_app();
        let bc = BoltConfig::default();

        // Place a wall to the right
        spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

        // Bolt moving right toward the wall
        let start_x = 200.0 - 50.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)),
            Position2D(Vec2::new(start_x, 0.0)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.x < 0.0,
            "bolt should reflect off wall, got vx={}",
            vel.0.x
        );
    }

    #[test]
    fn wall_hit_does_not_emit_cell_message() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

        let start_x = 200.0 - 50.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)),
            Position2D(Vec2::new(start_x, 0.0)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert!(
            hits.0.is_empty(),
            "wall hit should not emit BoltHitCell message"
        );
    }

    #[test]
    fn cell_hit_preferred_over_farther_wall() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        // Cell closer than wall
        let cell_y = 50.0;
        let cell_entity = spawn_cell(&mut app, 0.0, cell_y);
        spawn_wall(&mut app, 0.0, 200.0, 400.0, 50.0);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 1);
        assert_eq!(hits.0[0], cell_entity, "should hit cell, not wall");
    }

    // --- Piercing chip effect tests ---

    /// Spawns a cell with explicit [`CellHealth`] for piercing lookahead tests.
    fn spawn_cell_with_health(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        let (cw, ch) = default_cell_dims();
        let half_extents = Vec2::new(cw.half_width(), ch.half_height());
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                cw,
                ch,
                CellHealth::new(hp),
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    #[test]
    fn non_piercing_bolt_reflects_off_cell() {
        // Non-piercing bolt hitting a cell reflects (velocity.y < 0 after upward approach).
        // BoltHitCell is sent. No PiercingRemaining component involved.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let cell_y = 100.0;
        // CellHealth(30) — bolt deals base 10, survives.
        spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            // No PiercingRemaining or Piercing component
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y < 0.0,
            "non-piercing bolt should reflect downward off cell, got vy={}",
            vel.0.y
        );

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 1, "BoltHitCell should be sent");
    }

    #[test]
    fn piercing_bolt_passes_through_cell_it_would_destroy() {
        // Bolt with PiercingRemaining(2), no DamageBoost.
        // Cell with CellHealth(10) — base damage 10 would destroy it.
        // Bolt should NOT reflect (velocity.y > 0 after upward approach).
        // BoltHitCell is sent. PiercingRemaining decremented to 1.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let cell_y = 100.0;
        spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Piercing(2),
                PiercingRemaining(2),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y > 0.0,
            "piercing bolt should pass through cell it would destroy (velocity.y > 0), got vy={}",
            vel.0.y
        );

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "BoltHitCell should be sent for pierced cell"
        );

        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 1,
            "PiercingRemaining should decrement from 2 to 1 after one pierce"
        );
    }

    #[test]
    fn piercing_bolt_reflects_off_cell_it_would_not_destroy() {
        // Bolt with PiercingRemaining(1), no DamageBoost.
        // Cell with CellHealth(30) — base damage 10, cell survives.
        // Bolt should reflect (velocity.y < 0). PiercingRemaining stays 1.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Piercing(1),
                PiercingRemaining(1),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y < 0.0,
            "piercing bolt should reflect off cell it cannot destroy, got vy={}",
            vel.0.y
        );

        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 1,
            "PiercingRemaining should stay at 1 when cell survives the hit"
        );
    }

    #[test]
    fn piercing_with_damage_boost_uses_boosted_damage_for_lookahead() {
        // Bolt with PiercingRemaining(1), DamageBoost(0.5).
        // Cell with CellHealth(12).
        // Effective damage = (10 * (1.0 + 0.5)).round() = 15 >= 12 -> would destroy.
        // Bolt should pierce (velocity.y > 0).
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell_with_health(&mut app, 0.0, cell_y, 12.0);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Piercing(1),
                PiercingRemaining(1),
                DamageBoost(0.5),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y > 0.0,
            "bolt with DamageBoost(0.5) should pierce 12-HP cell (boosted damage=15), got vy={}",
            vel.0.y
        );

        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 0,
            "PiercingRemaining should decrement from 1 to 0 after piercing"
        );
    }

    #[test]
    fn two_stacked_cells_both_pierced_in_one_frame() {
        // Bolt with PiercingRemaining(2), high velocity (10000.0) to reach both cells
        // in one 64Hz frame (~156 units budget vs ~43 units needed).
        // Cell A at (0.0, 60.0), Cell B at (0.0, 90.0), both CellHealth(10).
        // Two BoltHitCell messages. PiercingRemaining goes from 2 to 0.
        let mut app = test_app();
        let bc = BoltConfig::default();
        app.insert_resource(FullHitMessages::default())
            .add_systems(FixedUpdate, collect_full_hits.after(bolt_cell_collision));

        // Place bolt below both cells, moving upward at high speed
        let near_cell_y = 60.0;
        let far_cell_y = 90.0;
        spawn_cell_with_health(&mut app, 0.0, near_cell_y, 10.0);
        spawn_cell_with_health(&mut app, 0.0, far_cell_y, 10.0);

        let start_y = near_cell_y - bc.radius - 25.0; // well below cell A
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 10000.0)), // 10000/64 ~ 156 units/frame -- covers both cells
                Piercing(2),
                PiercingRemaining(2),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let hits = app.world().resource::<FullHitMessages>();
        assert_eq!(
            hits.0.len(),
            2,
            "both stacked cells should be pierced in one frame (two BoltHitCell messages)"
        );

        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 0,
            "PiercingRemaining should go from 2 to 0 after piercing both cells"
        );
    }

    #[test]
    fn skip_set_is_per_bolt_two_bolts_pierce_independently() {
        // Two bolts each with PiercingRemaining(1), one cell in each bolt's path.
        // Each bolt pierces its cell independently. Two BoltHitCell messages total.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(FullHitMessages::default())
            .add_systems(FixedUpdate, collect_full_hits.after(bolt_cell_collision));

        let left_cell_y = 100.0;
        let right_cell_y = 100.0;
        spawn_cell_with_health(&mut app, -100.0, left_cell_y, 10.0);
        spawn_cell_with_health(&mut app, 100.0, right_cell_y, 10.0);

        let start_y = left_cell_y - cc.height / 2.0 - bc.radius - 2.0;

        // Bolt A targets cell A (left side)
        let bolt_a = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Piercing(1),
                PiercingRemaining(1),
                Position2D(Vec2::new(-100.0, start_y)),
            ))
            .id();

        // Bolt B targets cell B (right side)
        let bolt_b = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Piercing(1),
                PiercingRemaining(1),
                Position2D(Vec2::new(100.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let hits = app.world().resource::<FullHitMessages>();
        assert_eq!(
            hits.0.len(),
            2,
            "both bolts should pierce their respective cells independently (two BoltHitCell messages)"
        );

        // Both bolts should still be moving upward (they pierced, not reflected)
        let pr_a = app.world().get::<PiercingRemaining>(bolt_a).unwrap();
        let pr_b = app.world().get::<PiercingRemaining>(bolt_b).unwrap();
        assert_eq!(
            pr_a.0, 0,
            "bolt A PiercingRemaining should be 0 after pierce"
        );
        assert_eq!(
            pr_b.0, 0,
            "bolt B PiercingRemaining should be 0 after pierce"
        );
    }

    #[test]
    fn bolt_with_exhausted_piercing_reflects_normally() {
        // Bolt has Piercing(2) but PiercingRemaining(0) — all pierces used up.
        // It should reflect off a destroyable cell, not pierce through it.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        // CellHealth(10) — base damage 10 would destroy it, but piercing is exhausted.
        spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Piercing(2),
                PiercingRemaining(0),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y < 0.0,
            "bolt with exhausted piercing should reflect (vy < 0), got vy={}",
            vel.0.y
        );

        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 0,
            "PiercingRemaining should stay at 0 (exhausted), got {}",
            pr.0
        );
    }

    #[test]
    fn piercing_bolt_hits_grid_adjacent_cells() {
        // Bolt with Piercing(2), PiercingRemaining(2) should pierce through
        // both grid-adjacent cells (spaced GRID_STEP_Y=28 apart) in one frame.
        let mut app = test_app();
        let bc = BoltConfig::default();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let upper_cell_y = 100.0;
        let lower_cell_y = upper_cell_y - GRID_STEP_Y; // 72.0
        spawn_cell_with_health(&mut app, 0.0, upper_cell_y, 10.0);
        spawn_cell_with_health(&mut app, 0.0, lower_cell_y, 10.0);

        // Place bolt well below both cells, moving upward at high speed
        // to ensure it reaches both within one frame.
        let start_y = lower_cell_y - bc.radius - 30.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 10000.0)), // very fast to cover both cells in one frame
            Piercing(2),
            PiercingRemaining(2),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            2,
            "piercing bolt should hit both grid-adjacent cells, got {} hits",
            hits.0.len()
        );
    }

    #[test]
    fn wall_hit_resets_piercing_remaining() {
        // Bolt with Piercing(2), PiercingRemaining(0). Bolt hits wall.
        // PiercingRemaining should reset to Piercing.0 = 2.
        let mut app = test_app();
        let bc = BoltConfig::default();

        // Place wall to the right
        spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

        let start_x = 200.0 - 50.0 - bc.radius - 5.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(400.0, 0.1)),
                Piercing(2),
                PiercingRemaining(0),
                Position2D(Vec2::new(start_x, 0.0)),
            ))
            .id();

        tick(&mut app);

        // Verify wall hit occurred (velocity.x < 0)
        let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
        assert!(
            vel.0.x < 0.0,
            "bolt should have reflected off wall, got vx={}",
            vel.0.x
        );

        // PiercingRemaining should be reset to Piercing.0
        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 2,
            "wall hit should reset PiercingRemaining to Piercing.0 (2), got {}",
            pr.0
        );
    }

    // --- EntityScale collision tests ---

    #[test]
    fn scaled_bolt_effective_radius_changes_cell_collision_boundary() {
        let mut app = test_app();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = 81.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 50.0)),
                EntityScale(0.5),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
        assert!(
            vel.0.y > 0.0,
            "scaled bolt (effective_radius=4) at y=81 should NOT reach cell (expanded bottom=84), \
             got vy={:.1} (if negative, full radius expansion was used instead of scaled)",
            vel.0.y
        );
    }

    #[test]
    fn bolt_without_entity_scale_in_cell_collision_is_backward_compatible() {
        // Same as bolt_reflects_off_cell_bottom but explicitly no EntityScale.
        // Bolt should use full radius (8.0) and reflect normally.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            // No EntityScale component
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y < 0.0,
            "bolt without EntityScale should reflect normally, got vy={:.1}",
            vel.0.y
        );
    }

    // --- DamageCell emission tests ---

    /// Collects [`DamageCell`] messages into a resource for test assertions.
    #[derive(Resource, Default)]
    struct DamageCellMessages(Vec<DamageCell>);

    fn collect_damage_cells(
        mut reader: MessageReader<DamageCell>,
        mut msgs: ResMut<DamageCellMessages>,
    ) {
        for msg in reader.read() {
            msgs.0.push(msg.clone());
        }
    }

    /// Collects [`BoltHitWall`] messages into a resource for test assertions.
    #[derive(Resource, Default)]
    struct WallHitMessages(Vec<BoltHitWall>);

    fn collect_wall_hits(
        mut reader: MessageReader<BoltHitWall>,
        mut msgs: ResMut<WallHitMessages>,
    ) {
        for msg in reader.read() {
            msgs.0.push(msg.clone());
        }
    }

    /// Creates a test app with `DamageCell` and `BoltHitWall` message capture
    /// in addition to the standard `BoltHitCell`.
    fn test_app_with_damage_and_wall_messages() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RantzPhysics2dPlugin)
            .add_message::<BoltHitCell>()
            .add_message::<DamageCell>()
            .add_message::<BoltHitWall>()
            .insert_resource(DamageCellMessages::default())
            .insert_resource(WallHitMessages::default())
            .insert_resource(FullHitMessages::default())
            .add_systems(
                FixedUpdate,
                bolt_cell_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            )
            .add_systems(
                FixedUpdate,
                (collect_damage_cells, collect_wall_hits, collect_full_hits)
                    .after(bolt_cell_collision),
            );
        app
    }

    #[test]
    fn cell_collision_emits_damage_cell_with_base_damage() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "should emit exactly one DamageCell message on cell hit"
        );
        assert_eq!(
            msgs.0[0].cell, cell_entity,
            "DamageCell.cell should match the hit cell entity"
        );
        assert!(
            (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
            "DamageCell.damage should be BASE_BOLT_DAMAGE (10.0), got {}",
            msgs.0[0].damage
        );
        assert_eq!(
            msgs.0[0].source_bolt,
            Some(bolt_entity),
            "DamageCell.source_bolt should match the bolt entity"
        );
    }

    #[test]
    fn cell_collision_emits_damage_cell_with_zero_damage_boost() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            DamageBoost(0.0),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "DamageBoost(0.0) bolt should emit one DamageCell"
        );
        assert!(
            (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
            "DamageBoost(0.0) should produce damage == 10.0, got {}",
            msgs.0[0].damage
        );
    }

    #[test]
    fn cell_collision_emits_damage_cell_with_boosted_damage() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                DamageBoost(0.5),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(msgs.0.len(), 1, "boosted bolt should emit one DamageCell");
        assert!(
            (msgs.0[0].damage - 15.0).abs() < f32::EPSILON,
            "DamageCell.damage with DamageBoost(0.5) should be 15.0, got {}",
            msgs.0[0].damage
        );
        assert_eq!(
            msgs.0[0].source_bolt,
            Some(bolt_entity),
            "DamageCell.source_bolt should match bolt entity"
        );
    }

    #[test]
    fn two_bolts_emit_damage_cell_with_correct_source_bolt() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_a = spawn_cell(&mut app, -100.0, 100.0);
        let cell_b = spawn_cell(&mut app, 100.0, 100.0);

        let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

        let bolt_a = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(-100.0, start_y)),
            ))
            .id();
        let bolt_b = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(100.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            2,
            "two bolts hitting two cells should produce two DamageCell messages"
        );

        // Find the message for each cell and verify source_bolt
        let msg_a = msgs.0.iter().find(|m| m.cell == cell_a);
        let msg_b = msgs.0.iter().find(|m| m.cell == cell_b);
        assert!(msg_a.is_some(), "DamageCell for cell A should exist");
        assert!(msg_b.is_some(), "DamageCell for cell B should exist");
        assert_eq!(
            msg_a.unwrap().source_bolt,
            Some(bolt_a),
            "DamageCell for cell A should have source_bolt == bolt A"
        );
        assert_eq!(
            msg_b.unwrap().source_bolt,
            Some(bolt_b),
            "DamageCell for cell B should have source_bolt == bolt B"
        );
    }

    #[test]
    fn wall_hit_does_not_emit_damage_cell() {
        // A bolt hitting only a wall should produce zero DamageCell messages.
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();

        spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

        let start_x = 200.0 - 50.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)),
            Position2D(Vec2::new(start_x, 0.0)),
        ));

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert!(
            msgs.0.is_empty(),
            "wall hit should NOT emit DamageCell, got {} messages",
            msgs.0.len()
        );
    }

    #[test]
    fn piercing_bolt_emits_damage_cell_for_each_pierced_cell() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();

        let near_cell_y = 60.0;
        let far_cell_y = 90.0;
        let cell_a = spawn_cell_with_health(&mut app, 0.0, near_cell_y, 10.0);
        let cell_b = spawn_cell_with_health(&mut app, 0.0, far_cell_y, 10.0);

        let start_y = near_cell_y - bc.radius - 25.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 10000.0)),
                Piercing(2),
                PiercingRemaining(2),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            2,
            "piercing bolt should emit DamageCell for each pierced cell, got {}",
            msgs.0.len()
        );

        for msg in &msgs.0 {
            assert!(
                (msg.damage - 10.0).abs() < f32::EPSILON,
                "each DamageCell.damage should be 10.0, got {}",
                msg.damage
            );
            assert_eq!(
                msg.source_bolt,
                Some(bolt_entity),
                "each DamageCell.source_bolt should match the bolt entity"
            );
        }

        let cells_hit: Vec<Entity> = msgs.0.iter().map(|m| m.cell).collect();
        assert!(
            cells_hit.contains(&cell_a),
            "DamageCell for near cell should exist"
        );
        assert!(
            cells_hit.contains(&cell_b),
            "DamageCell for far cell should exist"
        );
    }

    #[test]
    fn cell_hit_emits_both_bolt_hit_cell_and_damage_cell() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(0.0, start_y)),
            ))
            .id();

        tick(&mut app);

        let hit_msgs = app.world().resource::<FullHitMessages>();
        assert_eq!(hit_msgs.0.len(), 1, "should emit exactly one BoltHitCell");
        assert_eq!(hit_msgs.0[0].cell, cell_entity);
        assert_eq!(hit_msgs.0[0].bolt, bolt_entity);

        let dmg_msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            dmg_msgs.0.len(),
            1,
            "should emit exactly one DamageCell alongside BoltHitCell"
        );
        assert_eq!(dmg_msgs.0[0].cell, cell_entity);
        assert_eq!(dmg_msgs.0[0].source_bolt, Some(bolt_entity));
    }

    // --- BoltHitWall emission tests ---

    #[test]
    fn wall_hit_emits_bolt_hit_wall_with_correct_bolt_entity() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();

        spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

        let start_x = 200.0 - 50.0 - bc.radius - 5.0;
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(400.0, 0.1)),
                Position2D(Vec2::new(start_x, 0.0)),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<WallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "wall hit should emit exactly one BoltHitWall message"
        );
        assert_eq!(
            msgs.0[0].bolt, bolt_entity,
            "BoltHitWall.bolt should match the bolt entity that hit the wall"
        );
    }

    #[test]
    fn cell_hit_does_not_emit_bolt_hit_wall() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        // BoltHitCell should be sent (existing behavior)
        let hit_msgs = app.world().resource::<FullHitMessages>();
        assert_eq!(
            hit_msgs.0.len(),
            1,
            "BoltHitCell should be sent for cell hit"
        );

        // BoltHitWall should NOT be sent
        let wall_msgs = app.world().resource::<WallHitMessages>();
        assert!(
            wall_msgs.0.is_empty(),
            "cell hit should NOT emit BoltHitWall, got {} messages",
            wall_msgs.0.len()
        );
    }

    #[test]
    fn bolt_hit_wall_identifies_correct_bolt_among_two() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();

        spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

        // Bolt A: moving right toward wall
        let start_x_a = 200.0 - 50.0 - bc.radius - 5.0;
        let bolt_a = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(400.0, 0.1)),
                Position2D(Vec2::new(start_x_a, 0.0)),
            ))
            .id();

        // Bolt B: moving upward, far from wall -- will not hit it
        let _bolt_b = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(-100.0, 0.0)),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<WallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "only bolt A should hit the wall, got {} BoltHitWall messages",
            msgs.0.len()
        );
        assert_eq!(
            msgs.0[0].bolt, bolt_a,
            "BoltHitWall.bolt should be bolt A (the one that hit the wall)"
        );
    }

    // --- Quadtree broad-phase collision tests ---
    //
    // These tests verify that the CCD system reads collision extents from
    // `Aabb2D.half_extents` (populated by the quadtree broad phase) rather
    // than from the legacy `CellWidth`/`CellHeight` or `WallSize` components.

    /// Spawns a cell with explicit `Aabb2D` `half_extents` that differ from the
    /// legacy `CellWidth`/`CellHeight` dimensions. Used to test which source
    /// the collision system reads for Minkowski expansion.
    fn spawn_cell_with_custom_aabb(
        app: &mut App,
        x: f32,
        y: f32,
        aabb_half_extents: Vec2,
    ) -> Entity {
        let (cw, ch) = default_cell_dims();
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                cw,
                ch,
                Aabb2D::new(Vec2::ZERO, aabb_half_extents),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    #[test]
    fn ccd_reads_cell_half_extents_from_aabb2d_not_cell_dimensions() {
        // Cell at (0, 100) with standard CellWidth(70)/CellHeight(24)
        // (half_extents 35.0, 12.0) but Aabb2D half_extents set to (5.0, 5.0).
        //
        // Bolt at (20, start_y) moving upward. x=20 is:
        //  - INSIDE the CellWidth-based expanded AABB (35 + 8 = 43)
        //  - OUTSIDE the Aabb2D-based expanded AABB (5 + 8 = 13)
        //
        // If the system reads from Aabb2D, the bolt misses (no reflection).
        // If the system reads from CellWidth/CellHeight, the bolt hits and reflects.
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        let _cell = spawn_cell_with_custom_aabb(
            &mut app,
            0.0,
            cell_y,
            Vec2::new(5.0, 5.0), // tiny AABB
        );

        // Bolt at x=20, well within CellWidth range (half=35) but outside
        // Aabb2D range (half=5). Place below the cell's bottom.
        let expanded_bottom = cell_y - cc.height / 2.0 - bc.radius;
        let start_y = expanded_bottom - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(20.0, start_y)),
        ));

        // Run one tick to populate quadtree, then another for collision
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.y > 0.0,
            "bolt at x=20 should miss the cell when CCD reads Aabb2D(5,5) \
             instead of CellWidth(70)/CellHeight(24) — got vy={:.1} \
             (negative means it reflected off the cell using legacy dimensions)",
            vel.0.y
        );
    }

    #[test]
    fn ccd_reads_wall_half_extents_from_aabb2d_not_wall_size() {
        // Wall at (200, 0) with WallSize half_width=50, half_height=300
        // but Aabb2D half_extents set to (5.0, 5.0).
        //
        // Bolt at (137, 50) moving right at (400, 0.1). y=50 is:
        //  - INSIDE the WallSize-based expanded Y range (-308 to 308)
        //  - OUTSIDE the Aabb2D-based expanded Y range (-13 to 13)
        //
        // If the system reads from WallSize, the bolt hits (y=50 within range).
        // If the system reads from Aabb2D, the bolt misses (y=50 outside range).
        let mut app = test_app();
        let bc = BoltConfig::default();

        // Spawn wall with large WallSize but tiny Aabb2D
        app.world_mut().spawn((
            Wall,
            WallSize {
                half_width: 50.0,
                half_height: 300.0,
            },
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
            Position2D(Vec2::new(200.0, 0.0)),
            GlobalPosition2D(Vec2::new(200.0, 0.0)),
            Spatial2D,
            GameDrawLayer::Wall,
        ));

        // Bolt outside the expanded AABB on the left (x=137 < 142=200-50-8)
        // but at y=50 which is inside WallSize range but outside Aabb2D range.
        let start_x = 200.0 - 50.0 - bc.radius - 5.0; // 137.0
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)),
            Position2D(Vec2::new(start_x, 50.0)),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&Velocity2D>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.0.x > 0.0,
            "bolt at y=50 should miss the wall when CCD reads Aabb2D(5,5) \
             instead of WallSize(50,300) — got vx={:.1} \
             (negative means it reflected off the wall using legacy WallSize)",
            vel.0.x
        );
    }

    #[test]
    fn ccd_uses_aabb2d_larger_than_cell_dimensions_to_detect_hit() {
        // Inverse test: cell has small CellWidth(70)/CellHeight(24) but
        // large Aabb2D half_extents (100.0, 50.0).
        //
        // Bolt at (60, start_y) moving upward. x=60 is:
        //  - OUTSIDE the CellWidth-based expanded AABB (35 + 8 = 43)
        //  - INSIDE the Aabb2D-based expanded AABB (100 + 8 = 108)
        //
        // If the system reads from Aabb2D, the bolt hits and reflects.
        // If the system reads from CellWidth/CellHeight, the bolt misses.
        let mut app = test_app();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let cell_y = 100.0;
        let cell_entity = spawn_cell_with_custom_aabb(
            &mut app,
            0.0,
            cell_y,
            Vec2::new(100.0, 50.0), // large AABB
        );

        // Place bolt at x=60, outside CellWidth range but inside Aabb2D range
        // Aabb2D expanded bottom: 100 - 50 - 8 = 42
        let start_y = 42.0 - 2.0; // just below the Aabb2D expanded bottom
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(60.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "bolt at x=60 should hit the cell when CCD uses Aabb2D(100,50) — \
             got {} hits (0 means it used legacy CellWidth/CellHeight instead)",
            hits.0.len()
        );
        assert_eq!(
            hits.0[0], cell_entity,
            "the hit entity should be the cell with the large Aabb2D"
        );
    }

    #[test]
    fn cell_with_aabb2d_but_no_cell_dimensions_is_collision_candidate() {
        // A cell entity with `Cell`, `Aabb2D`, `CollisionLayers`, and
        // `Position2D` but WITHOUT `CellWidth`/`CellHeight` components.
        //
        // The refactored system reads collision extents from `Aabb2D`, so
        // this cell IS a collision candidate even without the legacy
        // dimension components.
        //
        // The current system uses `Query<CollisionQueryCell>` which requires
        // `CellWidth`/`CellHeight` — so this cell is invisible to it.
        let mut app = test_app();
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let bc = BoltConfig::default();

        // Spawn a cell with ONLY Aabb2D (no CellWidth/CellHeight)
        let cell_y = 100.0;
        let half_extents = Vec2::new(35.0, 12.0);
        let cell_entity = app
            .world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(Vec2::new(0.0, cell_y)),
                GlobalPosition2D(Vec2::new(0.0, cell_y)),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id();

        // Bolt approaching from below
        let expanded_bottom = cell_y - half_extents.y - bc.radius;
        let start_y = expanded_bottom - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "cell with Aabb2D but no CellWidth/CellHeight should still be a collision \
             candidate when the system reads from Aabb2D — got {} hits \
             (0 means the system still requires CellWidth/CellHeight)",
            hits.0.len()
        );
        assert_eq!(
            hits.0[0], cell_entity,
            "the hit should be the cell with only Aabb2D"
        );
    }

    // ── SpawnedByEvolution → DamageCell.source_chip attribution tests ──

    #[test]
    fn damage_cell_carries_source_chip_from_bolt_spawned_by_evolution() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        let _bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(0.0, start_y)),
                SpawnedByEvolution("chain_lightning".to_owned()),
            ))
            .id();

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "should emit exactly one DamageCell message on cell hit"
        );
        assert_eq!(
            msgs.0[0].cell, cell_entity,
            "DamageCell.cell should match the hit cell entity"
        );
        assert_eq!(
            msgs.0[0].source_chip,
            Some("chain_lightning".to_owned()),
            "DamageCell.source_chip should carry the bolt's SpawnedByEvolution name"
        );
    }

    #[test]
    fn damage_cell_carries_source_chip_none_when_bolt_has_no_spawned_by_evolution() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_y = 100.0;
        spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ));

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "should emit exactly one DamageCell message on cell hit"
        );
        assert_eq!(
            msgs.0[0].source_chip, None,
            "DamageCell.source_chip should be None when bolt has no SpawnedByEvolution"
        );
    }

    #[test]
    fn multiple_bolts_with_different_attributions_produce_correctly_attributed_damage_cells() {
        let mut app = test_app_with_damage_and_wall_messages();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        let cell_a = spawn_cell(&mut app, -200.0, 100.0);
        let cell_b = spawn_cell(&mut app, 200.0, 100.0);

        let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

        // Bolt A: attributed to "alpha"
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(-200.0, start_y)),
            SpawnedByEvolution("alpha".to_owned()),
        ));

        // Bolt B: no attribution
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(200.0, start_y)),
        ));

        tick(&mut app);

        let msgs = app.world().resource::<DamageCellMessages>();
        assert_eq!(
            msgs.0.len(),
            2,
            "two bolts hitting two cells should produce two DamageCell messages"
        );

        let msg_a = msgs.0.iter().find(|m| m.cell == cell_a);
        let msg_b = msgs.0.iter().find(|m| m.cell == cell_b);
        assert!(msg_a.is_some(), "DamageCell for cell A should exist");
        assert!(msg_b.is_some(), "DamageCell for cell B should exist");
        assert_eq!(
            msg_a.unwrap().source_chip,
            Some("alpha".to_owned()),
            "DamageCell for cell A should have source_chip Some(\"alpha\") from bolt's SpawnedByEvolution"
        );
        assert_eq!(
            msg_b.unwrap().source_chip,
            None,
            "DamageCell for cell B should have source_chip None (bolt has no SpawnedByEvolution)"
        );
    }
}
