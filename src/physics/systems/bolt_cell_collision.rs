//! Bolt-cell collision detection and reflection.
//!
//! Physics only detects the collision and reflects the bolt. Cell damage
//! and destruction are handled by the cells domain via [`BoltHitCell`] messages.

use bevy::prelude::*;

use crate::bolt::BoltConfig;
use crate::bolt::components::{Bolt, BoltVelocity};
use crate::cells::CellConfig;
use crate::cells::components::Cell;
use crate::physics::messages::BoltHitCell;

/// Minimum physics frame rate assumed for swept collision detection.
///
/// Determines how far back the bolt's path is traced to catch tunneling
/// at high speeds.
const MIN_PHYSICS_FPS: f32 = 30.0;

/// When X and Y penetration depths are within this ratio, the hit is
/// ambiguous (corner clip). In that case, the bolt's velocity direction
/// is used as a tiebreaker instead of raw penetration depth.
const CORNER_DEPTH_RATIO: f32 = 1.5;

/// Query filter for cell data.
type CellQueryFilter = (With<Cell>, Without<Bolt>);

/// Returns the distance along a ray to the first intersection with an AABB,
/// or `None` if no intersection within `max_dist`.
fn ray_aabb_distance(
    origin: Vec2,
    direction: Vec2,
    max_dist: f32,
    aabb_center: Vec2,
    aabb_half_extents: Vec2,
) -> Option<f32> {
    let aabb_min = aabb_center - aabb_half_extents;
    let aabb_max = aabb_center + aabb_half_extents;

    let mut tmin = 0.0_f32;
    let mut tmax = max_dist;

    // X slab
    if direction.x.abs() < f32::EPSILON {
        if origin.x < aabb_min.x || origin.x > aabb_max.x {
            return None;
        }
    } else {
        let inv_d = direction.x.recip();
        let t1 = (aabb_min.x - origin.x) * inv_d;
        let t2 = (aabb_max.x - origin.x) * inv_d;
        let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        tmin = tmin.max(t_near);
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
        let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        tmin = tmin.max(t_near);
        tmax = tmax.min(t_far);
        if tmin > tmax {
            return None;
        }
    }

    Some(tmin)
}

/// Detects bolt-cell collisions and reflects the bolt.
///
/// Uses AABB overlap for collision, with swept ray-casting to catch
/// high-speed tunneling. Among overlapping cells, the one with the
/// smallest penetration depth (nearest surface) is chosen. Reflects the
/// bolt based on which face of the cell was hit. Sends [`BoltHitCell`]
/// messages for the cells domain to handle damage and destruction.
pub fn bolt_cell_collision(
    bolt_config: Res<BoltConfig>,
    cell_config: Res<CellConfig>,
    mut bolt_query: Query<(Entity, &mut Transform, &mut BoltVelocity), With<Bolt>>,
    cell_query: Query<(Entity, &Transform), CellQueryFilter>,
    mut hit_writer: MessageWriter<BoltHitCell>,
) {
    let cell_half_width = cell_config.half_width;
    let cell_half_height = cell_config.half_height;
    let half_extents = Vec2::new(
        cell_half_width + bolt_config.radius,
        cell_half_height + bolt_config.radius,
    );

    for (bolt_entity, mut bolt_transform, mut bolt_velocity) in &mut bolt_query {
        let bolt_pos = bolt_transform.translation;

        // Phase 1: Find best direct overlap (smallest penetration = nearest surface)
        let mut best_direct: Option<(Entity, Vec3, f32, bool)> = None;

        for (cell_entity, cell_transform) in &cell_query {
            let cell_pos = cell_transform.translation;
            let dx = (bolt_pos.x - cell_pos.x).abs();
            let dy = (bolt_pos.y - cell_pos.y).abs();

            if dx < half_extents.x && dy < half_extents.y {
                let depth_x = half_extents.x - dx;
                let depth_y = half_extents.y - dy;
                let min_depth = depth_x.min(depth_y);

                // Determine side vs top/bottom hit. When depths are
                // similar (corner clip), use velocity direction as a
                // tiebreaker so an upward bolt reflects Y, not X.
                let ratio = depth_x.max(depth_y) / depth_x.min(depth_y).max(f32::EPSILON);
                let is_side = if ratio < CORNER_DEPTH_RATIO {
                    bolt_velocity.value.x.abs() > bolt_velocity.value.y.abs()
                } else {
                    depth_x < depth_y
                };

                if best_direct.is_none_or(|(_, _, d, _)| min_depth < d) {
                    best_direct = Some((cell_entity, cell_pos, min_depth, is_side));
                }
            }
        }

        // Phase 2: If no direct hit, try swept collision for tunneling detection
        let hit = best_direct.map_or_else(
            || {
                let speed = bolt_velocity.speed();
                if speed > f32::EPSILON {
                    let vel_dir = bolt_velocity.direction();
                    let sweep_dist = speed / MIN_PHYSICS_FPS;
                    let bolt_pos_2d = bolt_pos.truncate();

                    let mut best_swept: Option<(Entity, Vec3, f32)> = None;

                    for (cell_entity, cell_transform) in &cell_query {
                        let cell_pos = cell_transform.translation;
                        if let Some(t) = ray_aabb_distance(
                            bolt_pos_2d,
                            -vel_dir,
                            sweep_dist,
                            cell_pos.truncate(),
                            half_extents,
                        ) && best_swept.is_none_or(|(_, _, d)| t < d)
                        {
                            best_swept = Some((cell_entity, cell_pos, t));
                        }
                    }

                    best_swept.map(|(entity, cell_pos, t)| {
                        let hit_point = bolt_pos_2d - vel_dir * t;
                        let rel_x = (hit_point.x - cell_pos.x).abs() / half_extents.x;
                        let rel_y = (hit_point.y - cell_pos.y).abs() / half_extents.y;
                        let is_side = rel_x > rel_y;
                        (entity, cell_pos, t, is_side)
                    })
                } else {
                    None
                }
            },
            Some,
        );

        // Apply the hit
        if let Some((cell_entity, cell_pos, _, is_side)) = hit {
            if is_side {
                bolt_velocity.value.x = -bolt_velocity.value.x;
                // Push in the direction the bolt is NOW traveling (post-reflection).
                // Using position-based sign fails when the bolt overshoots cell center.
                let sign = bolt_velocity.value.x.signum();
                bolt_transform.translation.x =
                    sign.mul_add(cell_half_width + bolt_config.radius, cell_pos.x);
            } else {
                bolt_velocity.value.y = -bolt_velocity.value.y;
                let sign = bolt_velocity.value.y.signum();
                bolt_transform.translation.y =
                    sign.mul_add(cell_half_height + bolt_config.radius, cell_pos.y);
            }

            hit_writer.write(BoltHitCell {
                bolt: bolt_entity,
                cell: cell_entity,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltVelocity};
    use crate::cells::components::Cell;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<CellConfig>();
        app.add_message::<BoltHitCell>();
        app.add_systems(Update, bolt_cell_collision);
        app
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut()
            .spawn((Cell, Transform::from_xyz(x, y, 0.0)))
            .id()
    }

    #[test]
    fn bolt_reflects_off_cell_top() {
        let mut app = test_app();
        let cc = CellConfig::default();
        spawn_cell(&mut app, 0.0, 100.0);

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 300.0),
            Transform::from_xyz(0.0, 100.0 - cc.half_height, 0.0),
        ));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y < 0.0, "bolt should reflect downward");
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
    fn fast_bolt_does_not_tunnel_through_cell() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // Cell at y=100
        spawn_cell(&mut app, 0.0, 100.0);

        // Bolt already past the cell (as if move_bolt teleported it through at high speed).
        // Bolt is above the cell, moving upward — it "jumped over" the cell this tick.
        let past_y = 100.0 + cc.half_height + bc.radius + 50.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 2000.0),
            Transform::from_xyz(0.0, past_y, 0.0),
        ));
        app.update();

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "tunneling bolt should still detect the cell hit"
        );
    }

    #[test]
    fn only_one_cell_hit_per_tick() {
        let mut app = test_app();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // Two adjacent cells, both overlapping the bolt
        spawn_cell(&mut app, 0.0, 100.0);
        spawn_cell(&mut app, cc.half_width.mul_add(2.0, -5.0), 100.0);

        // Bolt overlaps both cells
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 300.0),
            Transform::from_xyz(cc.half_width - 2.0, 100.0 - cc.half_height, 0.0),
        ));
        app.update();

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 1, "only one cell should be hit per tick");
    }

    #[test]
    fn nearest_cell_is_hit_when_multiple_overlap() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // "Far" cell: centered on the bolt (large penetration depth)
        let far_cell = spawn_cell(&mut app, 0.0, 100.0);

        // "Near" cell: edge just touching the bolt (small penetration depth)
        let near_x = cc.half_width + bc.radius - 1.0;
        let near_cell = spawn_cell(&mut app, near_x, 100.0);

        // Bolt positioned at origin of far cell — deep inside far, barely inside near
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 300.0),
            Transform::from_xyz(0.0, 100.0 - cc.half_height + 2.0, 0.0),
        ));
        app.update();

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 1, "should hit exactly one cell");
        assert_eq!(
            hits.0[0], near_cell,
            "should hit the nearer cell (smaller penetration), not the far cell ({far_cell:?})"
        );
    }

    #[test]
    fn multiple_bolts_each_hit_different_cells() {
        let mut app = test_app();
        let cc = CellConfig::default();
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        let cell_a = spawn_cell(&mut app, -100.0, 100.0);
        let cell_b = spawn_cell(&mut app, 100.0, 100.0);

        // Bolt A overlaps cell A
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 300.0),
            Transform::from_xyz(-100.0, 100.0 - cc.half_height, 0.0),
        ));
        // Bolt B overlaps cell B
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 300.0),
            Transform::from_xyz(100.0, 100.0 - cc.half_height, 0.0),
        ));
        app.update();

        let hits = app.world().resource::<HitCells>();
        assert_eq!(hits.0.len(), 2, "both bolts should register hits");
        assert!(hits.0.contains(&cell_a), "cell A should be in the hit list");
        assert!(hits.0.contains(&cell_b), "cell B should be in the hit list");
    }

    #[test]
    fn fast_bolt_does_not_ping_pong_inside_cell() {
        let mut app = test_app();
        app.insert_resource(HitCells::default());
        app.add_systems(Update, collect_cell_hits.after(bolt_cell_collision));

        // Cell at y=100. Bolt moving fast upward — will overshoot cell
        // center on the first tick, landing above it.
        spawn_cell(&mut app, 0.0, 100.0);

        // Position the bolt so move_bolt (not running here) would place it
        // past cell center. We simulate the post-move position directly.
        // Bolt is ABOVE cell center, velocity upward — should be pushed below.
        let bolt_y = 100.0 + 2.0; // past cell center
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(50.0, 600.0),
            Transform::from_xyz(0.0, bolt_y, 0.0),
        ));

        app.update();

        let hits = app.world().resource::<HitCells>();
        assert_eq!(
            hits.0.len(),
            1,
            "bolt should hit cell exactly once, not ping-pong (got {} hits)",
            hits.0.len()
        );

        // Verify bolt was pushed BELOW the cell (in approach direction),
        // not above (wrong side when bolt overshoots cell center)
        let bolt_tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            bolt_tf.translation.y < 100.0,
            "bolt should be pushed below cell, not above (y={:.1})",
            bolt_tf.translation.y
        );
    }

    #[test]
    fn upward_bolt_clipping_corner_reflects_y_not_x() {
        let mut app = test_app();
        let bc = BoltConfig::default();
        let cc = CellConfig::default();

        // Cell at (0, 100)
        spawn_cell(&mut app, 0.0, 100.0);

        // Bolt approaching from below, slightly off-center — clips the corner.
        // With pure penetration-depth logic, depth_x ≈ depth_y at corners,
        // which can misidentify this as a side hit (reflecting X while Y
        // continues upward). The bolt should reflect Y (bounce down).
        let bolt_x = cc.half_width + bc.radius - 2.5; // just inside the right edge
        let bolt_y = 100.0 - cc.half_height - bc.radius + 3.0; // just inside the bottom edge
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(50.0, 350.0), // mostly upward with slight rightward drift
            Transform::from_xyz(bolt_x, bolt_y, 0.0),
        ));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt moving mostly upward should reflect Y on corner hit, got vy={:.1}",
            vel.value.y
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
        app.update();

        // Bolt velocity should be unchanged (still upward)
        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(vel.value.y > 0.0, "bolt should still move upward");
    }
}
