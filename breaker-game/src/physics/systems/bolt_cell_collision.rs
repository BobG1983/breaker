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
//! [`BoltHitCell`] messages. Wall hits reflect only (no message).

use bevy::prelude::*;

use crate::{
    bolt::filters::ActiveFilter,
    physics::{
        filters::{CollisionFilterCell, CollisionFilterWall},
        messages::BoltHitCell,
        queries::{CollisionQueryBolt, CollisionQueryCell},
    },
    shared::{
        BASE_BOLT_DAMAGE,
        math::{CCD_EPSILON, MAX_BOUNCES, ray_vs_aabb},
    },
    wall::components::WallSize,
};

/// Advances bolts along their velocity, reflecting off cells and walls via swept CCD.
///
/// For each bolt, traces a ray from its current position in the velocity
/// direction. If a cell or wall is hit, the bolt is placed just before the
/// impact point, the velocity is reflected off the hit face, and tracing
/// continues with the remaining movement distance. Sends [`BoltHitCell`]
/// messages for each cell hit. Wall hits reflect only.
pub(crate) fn bolt_cell_collision(
    time: Res<Time<Fixed>>,
    mut bolt_query: Query<CollisionQueryBolt, ActiveFilter>,
    cell_query: Query<CollisionQueryCell, CollisionFilterCell>,
    wall_query: Query<(Entity, &Transform, &WallSize), CollisionFilterWall>,
    mut hit_writer: MessageWriter<BoltHitCell>,
    mut pierced_this_frame: Local<Vec<Entity>>,
) {
    let dt = time.delta_secs();

    for (
        bolt_entity,
        mut bolt_tf,
        mut bolt_vel,
        _,
        bolt_radius,
        mut piercing_remaining,
        piercing,
        damage_boost,
    ) in &mut bolt_query
    {
        let r = bolt_radius.0;
        let mut position = bolt_tf.translation.truncate();
        let mut velocity = bolt_vel.value;
        let mut remaining = velocity.length() * dt;

        // Effective damage for pierce lookahead (compared against cell HP).
        // must match handle_cell_hit damage formula
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

            // Find the nearest hit among all cells and walls
            let mut best: Option<(Option<Entity>, crate::shared::math::RayHit)> = None;

            // Check cells
            for (cell_entity, cell_tf, cell_w, cell_h, _cell_health) in &cell_query {
                // Skip cells already pierced by this bolt this frame
                if pierced_this_frame.contains(&cell_entity) {
                    continue;
                }
                let cell_pos = cell_tf.translation.truncate();
                let cell_half_extents =
                    Vec2::new(cell_w.half_width() + r, cell_h.half_height() + r);
                if let Some(hit) =
                    ray_vs_aabb(position, direction, remaining, cell_pos, cell_half_extents)
                    && best.as_ref().is_none_or(|(_, b)| hit.distance < b.distance)
                {
                    best = Some((Some(cell_entity), hit));
                }
            }

            // Check walls
            for (_wall_entity, wall_tf, wall_size) in &wall_query {
                let wall_pos = wall_tf.translation.truncate();
                let wall_half_extents =
                    Vec2::new(wall_size.half_width + r, wall_size.half_height + r);
                if let Some(hit) =
                    ray_vs_aabb(position, direction, remaining, wall_pos, wall_half_extents)
                    && best.as_ref().is_none_or(|(_, b)| hit.distance < b.distance)
                {
                    best = Some((None, hit));
                }
            }

            let Some((hit_cell, hit)) = best else {
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
                let cell_hp = cell_query
                    .get(cell_entity)
                    .ok()
                    .and_then(|(_, _, _, _, health)| health)
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
            } else {
                // WALL HIT: reflect and reset PiercingRemaining
                velocity -= 2.0 * velocity.dot(hit.normal) * hit.normal;
                // Reset PiercingRemaining to Piercing.0
                if let (Some(pr), Some(p)) = (&mut piercing_remaining, piercing) {
                    pr.0 = p.0;
                }
            }
        }

        bolt_tf.translation = position.extend(bolt_tf.translation.z);
        bolt_vel.value = velocity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::{
            components::{Bolt, BoltBaseSpeed, BoltRadius, BoltServing, BoltVelocity},
            resources::BoltConfig,
        },
        cells::{
            components::{Cell, CellHealth, CellHeight, CellWidth},
            resources::CellConfig,
        },
        chips::components::{DamageBoost, Piercing, PiercingRemaining},
        wall::components::{Wall, WallSize},
    };

    // --- CCD system tests ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .add_systems(FixedUpdate, bolt_cell_collision);
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

    fn spawn_cell(app: &mut App, x: f32, y: f32) -> Entity {
        let (cw, ch) = default_cell_dims();
        app.world_mut()
            .spawn((Cell, cw, ch, Transform::from_xyz(x, y, 0.0)))
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
        let start_y = cell_y - cc.height / 2.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
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
        let cell_bottom = cell_y - cc.height / 2.0 - bc.radius;
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
        let start_x = cell_x - cc.width / 2.0 - bc.radius - 5.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
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
        let cell_bottom = cell_y - cc.height / 2.0 - bc.radius;
        let start_y = cell_bottom - 1.0;
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
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
            bolt_param_bundle(),
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
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        // 3×2 mini-grid at real spacing
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
        app.insert_resource(HitCells::default())
            .add_systems(FixedUpdate, collect_cell_hits.after(bolt_cell_collision));

        let cell_a = spawn_cell(&mut app, -100.0, 100.0);
        let cell_b = spawn_cell(&mut app, 100.0, 100.0);

        let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

        // Bolt A near cell A
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(-100.0, start_y, 0.0),
        ));
        // Bolt B near cell B
        app.world_mut().spawn((
            Bolt,
            bolt_param_bundle(),
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
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltHitCell>()
            .add_systems(FixedUpdate, bolt_cell_collision);

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                bolt_param_bundle(),
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
                BoltVelocity::new(0.0, 400.0),
                Transform::from_xyz(0.0, start_y, 0.0),
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
        app.world_mut().spawn((
            Wall,
            WallSize {
                half_width,
                half_height,
            },
            Transform::from_xyz(x, y, 0.0),
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
            BoltVelocity::new(400.0, 0.1),
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
            "bolt should reflect off wall, got vx={}",
            vel.value.x
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
            BoltVelocity::new(400.0, 0.1),
            Transform::from_xyz(start_x, 0.0, 0.0),
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
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, start_y, 0.0),
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
        app.world_mut()
            .spawn((
                Cell,
                cw,
                ch,
                CellHealth::new(hp),
                Transform::from_xyz(x, y, 0.0),
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
            BoltVelocity::new(0.0, 400.0),
            // No PiercingRemaining or Piercing component
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
            "non-piercing bolt should reflect downward off cell, got vy={}",
            vel.value.y
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
                BoltVelocity::new(0.0, 400.0),
                Piercing(2),
                PiercingRemaining(2),
                Transform::from_xyz(0.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y > 0.0,
            "piercing bolt should pass through cell it would destroy (velocity.y > 0), got vy={}",
            vel.value.y
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
                BoltVelocity::new(0.0, 400.0),
                Piercing(1),
                PiercingRemaining(1),
                Transform::from_xyz(0.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y < 0.0,
            "piercing bolt should reflect off cell it cannot destroy, got vy={}",
            vel.value.y
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
        // Effective damage = (10 * (1.0 + 0.5)).round() = 15 >= 12 → would destroy.
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
                BoltVelocity::new(0.0, 400.0),
                Piercing(1),
                PiercingRemaining(1),
                DamageBoost(0.5),
                Transform::from_xyz(0.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y > 0.0,
            "bolt with DamageBoost(0.5) should pierce 12-HP cell (boosted damage=15), got vy={}",
            vel.value.y
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
                BoltVelocity::new(0.0, 10000.0), // 10000/64 ≈ 156 units/frame — covers both cells
                Piercing(2),
                PiercingRemaining(2),
                Transform::from_xyz(0.0, start_y, 0.0),
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
                BoltVelocity::new(0.0, 400.0),
                Piercing(1),
                PiercingRemaining(1),
                Transform::from_xyz(-100.0, start_y, 0.0),
            ))
            .id();

        // Bolt B targets cell B (right side)
        let bolt_b = app
            .world_mut()
            .spawn((
                Bolt,
                bolt_param_bundle(),
                BoltVelocity::new(0.0, 400.0),
                Piercing(1),
                PiercingRemaining(1),
                Transform::from_xyz(100.0, start_y, 0.0),
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
                BoltVelocity::new(0.0, 400.0),
                Piercing(2),
                PiercingRemaining(0),
                Transform::from_xyz(0.0, start_y, 0.0),
            ))
            .id();

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.value.y < 0.0,
            "bolt with exhausted piercing should reflect (vy < 0), got vy={}",
            vel.value.y
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
            BoltVelocity::new(0.0, 10000.0), // very fast to cover both cells in one frame
            Piercing(2),
            PiercingRemaining(2),
            Transform::from_xyz(0.0, start_y, 0.0),
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
                BoltVelocity::new(400.0, 0.1),
                Piercing(2),
                PiercingRemaining(0),
                Transform::from_xyz(start_x, 0.0, 0.0),
            ))
            .id();

        tick(&mut app);

        // Verify wall hit occurred (velocity.x < 0)
        let vel = app.world().get::<BoltVelocity>(bolt_entity).unwrap();
        assert!(
            vel.value.x < 0.0,
            "bolt should have reflected off wall, got vx={}",
            vel.value.x
        );

        // PiercingRemaining should be reset to Piercing.0
        let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
        assert_eq!(
            pr.0, 2,
            "wall hit should reset PiercingRemaining to Piercing.0 (2), got {}",
            pr.0
        );
    }
}
