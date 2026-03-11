//! Bolt-cell collision detection via swept CCD (continuous collision detection).
//!
//! Instead of moving the bolt first and then checking for overlaps, this system
//! traces the bolt's path forward using ray-vs-expanded-AABB intersection.
//! On each hit, the bolt is placed just before the impact point, the velocity
//! is reflected, and the remaining movement continues. The bolt never overlaps
//! any cell.
//!
//! Cell damage and destruction are handled by the cells domain via
//! [`BoltHitCell`] messages.

use bevy::prelude::*;

use crate::bolt::BoltConfig;
use crate::bolt::components::{ActiveBoltFilter, Bolt, BoltVelocity};
use crate::cells::CellConfig;
use crate::cells::components::Cell;
use crate::physics::messages::BoltHitCell;

/// Maximum number of cell bounces resolved per bolt per frame.
///
/// Prevents infinite loops in degenerate geometries. At max bolt speed (800)
/// and 64 Hz, the bolt travels ~12.5 units per frame — 4 bounces is generous.
const MAX_BOUNCES: u32 = 4;

/// Sub-pixel separation gap applied after each collision.
///
/// The bolt is placed this far outside the cell's expanded AABB to prevent
/// floating-point touching on the next sweep.
const CCD_EPSILON: f32 = 0.01;

/// Query filter for cell data.
type CellFilter = (With<Cell>, Without<Bolt>);

/// Result of a ray-vs-expanded-AABB intersection test.
struct RayHit {
    /// Distance along the ray to the entry point.
    distance: f32,
    /// Outward face normal at the entry point.
    normal: Vec2,
}

/// Casts a ray against an AABB and returns the entry distance and face normal.
///
/// The AABB should already be Minkowski-expanded by the bolt radius so that
/// a point-ray test is equivalent to a circle-vs-rectangle test.
///
/// Returns `None` if the ray misses, the origin is inside the AABB, or the
/// hit is beyond `max_dist`.
fn ray_expanded_aabb(
    origin: Vec2,
    direction: Vec2,
    max_dist: f32,
    aabb_center: Vec2,
    aabb_half_extents: Vec2,
) -> Option<RayHit> {
    let aabb_min = aabb_center - aabb_half_extents;
    let aabb_max = aabb_center + aabb_half_extents;

    let mut tmin = 0.0_f32;
    let mut tmax = max_dist;
    let mut normal = Vec2::ZERO;

    // X slab
    if direction.x.abs() < f32::EPSILON {
        if origin.x < aabb_min.x || origin.x > aabb_max.x {
            return None;
        }
    } else {
        let inv_d = direction.x.recip();
        let t1 = (aabb_min.x - origin.x) * inv_d;
        let t2 = (aabb_max.x - origin.x) * inv_d;
        let (t_near, t_far, near_normal) = if t1 < t2 {
            (t1, t2, Vec2::NEG_X)
        } else {
            (t2, t1, Vec2::X)
        };
        if t_near > tmin {
            tmin = t_near;
            normal = near_normal;
        }
        tmax = tmax.min(t_far);
        if tmin > tmax {
            return None;
        }
    }

    // Y slab
    if direction.y.abs() < f32::EPSILON {
        if origin.y < aabb_min.y || origin.y > aabb_max.y {
            return None;
        }
    } else {
        let inv_d = direction.y.recip();
        let t1 = (aabb_min.y - origin.y) * inv_d;
        let t2 = (aabb_max.y - origin.y) * inv_d;
        let (t_near, t_far, near_normal) = if t1 < t2 {
            (t1, t2, Vec2::NEG_Y)
        } else {
            (t2, t1, Vec2::Y)
        };
        if t_near > tmin {
            tmin = t_near;
            normal = near_normal;
        }
        tmax = tmax.min(t_far);
        if tmin > tmax {
            return None;
        }
    }

    // Origin inside AABB (tmin == 0 means the ray starts overlapping)
    if tmin <= 0.0 {
        return None;
    }

    Some(RayHit {
        distance: tmin,
        normal,
    })
}

/// Advances bolts along their velocity, reflecting off cells via swept CCD.
///
/// For each bolt, traces a ray from its current position in the velocity
/// direction. If a cell is hit, the bolt is placed just before the impact
/// point, the velocity is reflected off the hit face, and tracing continues
/// with the remaining movement distance. Sends [`BoltHitCell`] messages for
/// each cell hit.
pub fn bolt_cell_collision(
    time: Res<Time<Fixed>>,
    bolt_config: Res<BoltConfig>,
    cell_config: Res<CellConfig>,
    mut bolt_query: Query<(Entity, &mut Transform, &mut BoltVelocity), ActiveBoltFilter>,
    cell_query: Query<(Entity, &Transform), CellFilter>,
    mut hit_writer: MessageWriter<BoltHitCell>,
) {
    let dt = time.delta_secs();
    let half_extents = Vec2::new(
        cell_config.half_width + bolt_config.radius,
        cell_config.half_height + bolt_config.radius,
    );

    for (bolt_entity, mut bolt_tf, mut bolt_vel) in &mut bolt_query {
        let mut position = bolt_tf.translation.truncate();
        let mut velocity = bolt_vel.value;
        let mut remaining = velocity.length() * dt;

        for _ in 0..MAX_BOUNCES {
            if remaining <= CCD_EPSILON {
                break;
            }

            let direction = velocity.normalize_or_zero();
            if direction == Vec2::ZERO {
                break;
            }

            // Find the nearest cell hit along the ray
            let mut best: Option<(Entity, RayHit)> = None;

            for (cell_entity, cell_tf) in &cell_query {
                let cell_pos = cell_tf.translation.truncate();
                if let Some(hit) =
                    ray_expanded_aabb(position, direction, remaining, cell_pos, half_extents)
                    && best.as_ref().is_none_or(|(_, b)| hit.distance < b.distance)
                {
                    best = Some((cell_entity, hit));
                }
            }

            let Some((cell_entity, hit)) = best else {
                // No cell in path — move the full remaining distance
                position += direction * remaining;
                break;
            };

            // Move to just before the impact point
            let advance = (hit.distance - CCD_EPSILON).max(0.0);
            position += direction * advance;
            remaining -= advance;

            // Reflect velocity off the hit face
            velocity -= 2.0 * velocity.dot(hit.normal) * hit.normal;

            hit_writer.write(BoltHitCell {
                bolt: bolt_entity,
                cell: cell_entity,
            });
        }

        bolt_tf.translation = position.extend(bolt_tf.translation.z);
        bolt_vel.value = velocity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltServing};

    // --- ray_expanded_aabb unit tests ---

    #[test]
    fn ray_hit_from_below() {
        let hit = ray_expanded_aabb(
            Vec2::new(0.0, -30.0),
            Vec2::Y,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        )
        .expect("should hit");

        assert!(
            (hit.distance - 10.0).abs() < 0.01,
            "distance={}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_Y);
    }

    #[test]
    fn ray_hit_from_side() {
        let hit = ray_expanded_aabb(
            Vec2::new(-60.0, 0.0),
            Vec2::X,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        )
        .expect("should hit");

        assert!(
            (hit.distance - 17.0).abs() < 0.01,
            "distance={}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_X);
    }

    #[test]
    fn ray_miss_parallel() {
        let result = ray_expanded_aabb(
            Vec2::new(0.0, -30.0),
            Vec2::X, // parallel to Y extent
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        );
        assert!(result.is_none(), "parallel ray should miss");
    }

    #[test]
    fn ray_miss_beyond_max_dist() {
        let result = ray_expanded_aabb(
            Vec2::new(0.0, -200.0),
            Vec2::Y,
            10.0, // too short to reach
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        );
        assert!(result.is_none(), "ray should not reach cell");
    }

    #[test]
    fn ray_origin_inside_returns_none() {
        let result = ray_expanded_aabb(
            Vec2::ZERO, // inside the AABB
            Vec2::Y,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        );
        assert!(result.is_none(), "origin inside AABB should return None");
    }

    // --- CCD system tests ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<CellConfig>();
        app.add_message::<BoltHitCell>();
        app.add_systems(Update, bolt_cell_collision);
        app
    }

    /// Advances `Time<Fixed>` by one default timestep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(timestep);
        app.update();
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut()
            .spawn((Cell, Transform::from_xyz(x, y, 0.0)))
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
            BoltVelocity::new(0.0, speed),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));

        tick(&mut app);

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = speed.mul_add(dt, start_y);
        assert!(
            (tf.translation.y - expected).abs() < 0.1,
            "bolt should move full distance: expected {expected}, got {}",
            tf.translation.y
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
        let start_y = cell_y - cc.half_height - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt should reflect downward, got vy={}",
            vel.value.y
        );

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        let cell_bottom = cell_y - cc.half_height - bc.radius;
        assert!(
            tf.translation.y < cell_bottom,
            "bolt should be below cell: y={:.2}, cell_bottom={cell_bottom:.2}",
            tf.translation.y
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
        let start_x = cell_x - cc.half_width - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(400.0, 0.1), // mostly horizontal
            Transform::from_xyz(start_x, 0.0, 0.0),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.x < 0.0,
            "bolt should reflect leftward, got vx={}",
            vel.value.x
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
        let cell_bottom = cell_y - cc.half_height - bc.radius;
        let start_y = cell_bottom - 1.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));

        tick(&mut app);

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();

        // Bolt should NOT be sitting right at the impact point — it should have
        // continued downward with the remaining distance after reflection
        assert!(
            tf.translation.y < start_y,
            "bolt should have moved past the impact point in reflected direction, \
             got y={:.2}, start={start_y:.2}",
            tf.translation.y
        );
    }

    #[test]
    fn bolt_hits_only_nearest_cell() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // Two cells vertically, bolt path crosses both
        let near_y = 50.0;
        let far_y = 100.0;
        let near_cell = spawn_cell(&mut app, 0.0, near_y);
        spawn_cell(&mut app, 0.0, far_y);

        let start_y = near_y - cc.half_height - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
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

    #[test]
    fn bolt_hit_cell_message_sent() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        let cell_y = 100.0;
        let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

        let start_y = cell_y - cc.half_height - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
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
            BoltVelocity::new(0.0, 300.0),
            Transform::from_xyz(0.0, -100.0, 0.0),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y > 0.0, "bolt should still move upward");
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
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        let upper_y = 100.0;
        let lower_y = upper_y - GRID_STEP_Y;
        spawn_cell(&mut app, 0.0, upper_y);
        spawn_cell(&mut app, 0.0, lower_y);

        // Bolt below the upper cell, moving up
        let start_y = upper_y - cc.half_height - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));

        // Two frames — CCD should prevent cascade
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
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        let left_x = 0.0;
        let right_x = left_x + GRID_STEP_X;
        let cell_y = 100.0;
        spawn_cell(&mut app, left_x, cell_y);
        spawn_cell(&mut app, right_x, cell_y);

        // Bolt left of right cell, moving right
        let start_x = right_x - cc.half_width - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(400.0, 10.0),
            Transform::from_xyz(start_x, cell_y, 0.0),
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
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // 3×2 mini-grid at real spacing
        let base_y = 100.0;
        for row in 0..2 {
            for col in 0..3 {
                #[allow(clippy::cast_precision_loss)]
                let x = (col as f32 - 1.0) * GRID_STEP_X;
                #[allow(clippy::cast_precision_loss)]
                let y = (row as f32).mul_add(GRID_STEP_Y, base_y);
                spawn_cell(&mut app, x, y);
            }
        }

        let start_y = base_y - cc.half_height - bc.radius - 2.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(30.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
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
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // Two cells very close together creating a narrow channel.
        // Bolt bouncing between them could loop forever without the cap.
        let gap = bc.radius.mul_add(2.0, 2.0); // just wider than bolt diameter
        spawn_cell(&mut app, 0.0, gap / 2.0 + cc.half_height + bc.radius);
        spawn_cell(&mut app, 0.0, -(gap / 2.0 + cc.half_height + bc.radius));

        // Bolt in the channel, moving up very fast
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.1, 800.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
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
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        let cell_a = spawn_cell(&mut app, -100.0, 100.0);
        let cell_b = spawn_cell(&mut app, 100.0, 100.0);

        let start_y = 100.0 - cc.half_height - bc.radius - 2.0;

        // Bolt A near cell A
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(-100.0, start_y, 0.0),
        ));
        // Bolt B near cell B
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(100.0, start_y, 0.0),
        ));

        tick(&mut app);

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 2, "both bolts should register hits");
        assert!(hits.0.contains(&cell_a), "cell A should be hit");
        assert!(hits.0.contains(&cell_b), "cell B should be hit");
    }

    #[test]
    fn serving_bolt_is_not_advanced() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<CellConfig>();
        app.add_message::<BoltHitCell>();
        app.add_systems(Update, bolt_cell_collision);

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                BoltVelocity::new(0.0, 400.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert!(
            tf.translation.y.abs() < f32::EPSILON,
            "serving bolt should not be moved by CCD, got y={}",
            tf.translation.y
        );
    }
}
