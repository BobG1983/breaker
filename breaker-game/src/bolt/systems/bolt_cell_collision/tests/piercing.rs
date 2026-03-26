use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    chips::components::{DamageBoost, Piercing, PiercingRemaining},
};

#[test]
fn non_piercing_bolt_reflects_off_cell() {
    // Non-piercing bolt hitting a cell reflects (velocity.y < 0 after upward approach).
    // BoltHitCell is sent. No PiercingRemaining component involved.
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

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
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

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
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

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
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

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
    let bc = crate::bolt::resources::BoltConfig::default();
    app.insert_resource(FullHitMessages::default()).add_systems(
        FixedUpdate,
        collect_full_hits.after(super::super::system::bolt_cell_collision),
    );

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
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(FullHitMessages::default()).add_systems(
        FixedUpdate,
        collect_full_hits.after(super::super::system::bolt_cell_collision),
    );

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
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

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
    let bc = crate::bolt::resources::BoltConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

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
    let bc = crate::bolt::resources::BoltConfig::default();

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
