use bevy::{ecs::world::CommandQueue, prelude::*};

use super::helpers::*;
use crate::prelude::*;

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
    spawn_bolt(&mut app, 0.0, start_y, 0.0, speed);

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
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    // Place bolt below the cell's expanded AABB, moving upward
    let start_y = cell_y - cc.height / 2.0 - bc.radius - 5.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

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
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_x = 100.0;
    spawn_cell(&mut app, cell_x, 0.0);

    // Place bolt left of cell, moving right
    let start_x = cell_x - cc.width / 2.0 - bc.radius - 5.0;
    spawn_bolt(&mut app, start_x, 0.0, 400.0, 0.1);

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
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    // Place bolt just 1 unit below the expanded AABB bottom, moving up fast.
    // It will hit quickly and have most of its movement remaining.
    let cell_bottom = cell_y - cc.height / 2.0 - bc.radius;
    let start_y = cell_bottom - 1.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

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
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    // Two cells vertically, bolt path crosses both
    let near_y = 50.0;
    let far_y = 100.0;
    let near_cell = spawn_cell(&mut app, 0.0, near_y);
    spawn_cell(&mut app, 0.0, far_y);

    let start_y = near_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    // Only the nearer cell should be hit (bolt reflects before reaching the far one)
    assert_eq!(hits.0.len(), 1, "should hit exactly one cell");
    assert_eq!(hits.0[0], near_cell, "should hit the nearer cell");
}

#[test]
fn bolt_hit_cell_message_sent() {
    let mut app = test_app();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

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

    spawn_bolt(&mut app, 0.0, -100.0, 0.0, 300.0);

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "bolt should still move upward");
}

#[test]
fn max_bounces_cap() {
    let mut app = test_app();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    // Two cells very close together creating a narrow channel.
    // Bolt bouncing between them could loop forever without the cap.
    let gap = bc.radius.mul_add(2.0, 2.0); // just wider than bolt diameter
    spawn_cell(&mut app, 0.0, gap / 2.0 + cc.height / 2.0 + bc.radius);
    spawn_cell(&mut app, 0.0, -(gap / 2.0 + cc.height / 2.0 + bc.radius));

    // Bolt in the channel, moving up very fast
    spawn_bolt(&mut app, 0.0, 0.0, 0.1, 800.0);

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
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_a = spawn_cell(&mut app, -100.0, 100.0);
    let cell_b = spawn_cell(&mut app, 100.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

    // Bolt A near cell A
    spawn_bolt(&mut app, -100.0, start_y, 0.0, 400.0);
    // Bolt B near cell B
    spawn_bolt(&mut app, 100.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(hits.0.len(), 2, "both bolts should register hits");
    assert!(hits.0.contains(&cell_a), "cell A should be hit");
    assert!(hits.0.contains(&cell_b), "cell B should be hit");
}

#[test]
fn serving_bolt_is_not_advanced() {
    let mut app = test_app();

    let entity = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&super::helpers::test_bolt_definition())
                .serving()
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };

    tick(&mut app);

    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        pos.0.y.abs() < f32::EPSILON,
        "serving bolt should not be moved by CCD, got y={}",
        pos.0.y
    );
}

// --- BoltImpactCell bolt entity tests ---

#[test]
fn bolt_cell_collision_populates_bolt_entity_in_message() {
    // This test verifies that BoltImpactCell.bolt is set to the actual bolt entity,
    // not Entity::PLACEHOLDER. It will FAIL until the production code is fixed
    // to capture the bolt entity from the query binding.
    let mut app = test_app();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(FullHitMessages::default()).add_systems(
        FixedUpdate,
        collect_full_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hits = app.world().resource::<FullHitMessages>();
    assert_eq!(
        hits.0.len(),
        1,
        "should send exactly one BoltImpactCell message"
    );
    assert_ne!(
        hits.0[0].bolt,
        Entity::PLACEHOLDER,
        "BoltImpactCell.bolt should not be Entity::PLACEHOLDER — it should be the real bolt entity"
    );
    assert_eq!(
        hits.0[0].bolt, bolt_entity,
        "BoltImpactCell.bolt should equal the bolt entity that caused the collision"
    );
}
