//! Tests for piercing through multiple cells — stacked cells, independent bolts,
//! exhausted piercing, and grid-adjacent spacing.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{components::PiercingRemaining, systems::bolt_cell_collision::tests::helpers::*},
    effect::effects::piercing::ActivePiercings,
};

#[test]
fn two_stacked_cells_both_pierced_in_one_frame() {
    // Bolt with PiercingRemaining(2), high velocity (10000.0) to reach both cells
    // in one 64Hz frame (~156 units budget vs ~43 units needed).
    // Cell A at (0.0, 60.0), Cell B at (0.0, 90.0), both CellHealth(10).
    // Two BoltImpactCell messages. PiercingRemaining goes from 2 to 0.
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    app.insert_resource(FullHitMessages::default()).add_systems(
        FixedUpdate,
        collect_full_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    // Place bolt below both cells, moving upward at high speed
    let near_cell_y = 60.0;
    let far_cell_y = 90.0;
    spawn_cell_with_health(&mut app, 0.0, near_cell_y, 10.0);
    spawn_cell_with_health(&mut app, 0.0, far_cell_y, 10.0);

    let start_y = near_cell_y - bc.radius - 25.0; // well below cell A
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![2]), PiercingRemaining(2)));

    tick(&mut app);

    let hits = app.world().resource::<FullHitMessages>();
    assert_eq!(
        hits.0.len(),
        2,
        "both stacked cells should be pierced in one frame (two BoltImpactCell messages)"
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
    // Each bolt pierces its cell independently. Two BoltImpactCell messages total.
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(FullHitMessages::default()).add_systems(
        FixedUpdate,
        collect_full_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let left_cell_y = 100.0;
    let right_cell_y = 100.0;
    spawn_cell_with_health(&mut app, -100.0, left_cell_y, 10.0);
    spawn_cell_with_health(&mut app, 100.0, right_cell_y, 10.0);

    let start_y = left_cell_y - cc.height / 2.0 - bc.radius - 2.0;

    // Bolt A targets cell A (left side)
    let bolt_a = spawn_bolt(&mut app, -100.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_a)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));

    // Bolt B targets cell B (right side)
    let bolt_b = spawn_bolt(&mut app, 100.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_b)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));

    tick(&mut app);

    let hits = app.world().resource::<FullHitMessages>();
    assert_eq!(
        hits.0.len(),
        2,
        "both bolts should pierce their respective cells independently (two BoltImpactCell messages)"
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
    // Bolt has ActivePiercings(vec![2]) but PiercingRemaining(0) — all pierces used up.
    // It should reflect off a destroyable cell, not pierce through it.
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    // CellHealth(10) — base damage 10 would destroy it, but piercing is exhausted.
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![2]), PiercingRemaining(0)));

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
    // Bolt with ActivePiercings(vec![2]), PiercingRemaining(2) should pierce through
    // both grid-adjacent cells (spaced GRID_STEP_Y=28 apart) in one frame.
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let upper_cell_y = 100.0;
    let lower_cell_y = upper_cell_y - GRID_STEP_Y; // 72.0
    spawn_cell_with_health(&mut app, 0.0, upper_cell_y, 10.0);
    spawn_cell_with_health(&mut app, 0.0, lower_cell_y, 10.0);

    // Place bolt well below both cells, moving upward at high speed
    // to ensure it reaches both within one frame.
    let start_y = lower_cell_y - bc.radius - 30.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![2]), PiercingRemaining(2)));

    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(
        hits.0.len(),
        2,
        "piercing bolt should hit both grid-adjacent cells, got {} hits",
        hits.0.len()
    );
}
