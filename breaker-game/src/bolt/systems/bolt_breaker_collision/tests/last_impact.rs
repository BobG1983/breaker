//! `LastImpact` stamping tests for bolt-breaker collision.
//!
//! Verifies that `LastImpact { position, side }` is inserted/updated on the
//! bolt entity when the bolt rebounds off the breaker surface.

use bevy::prelude::*;

use super::helpers::*;
use crate::bolt::components::{ImpactSide, LastImpact};

// ── Behavior 1: top-surface rebound stamps LastImpact with ImpactSide::Top ──

#[test]
fn breaker_top_rebound_stamps_last_impact_with_top_side() {
    // Given: Bolt at (0.0, -229.0) with velocity (0.0, -400.0),
    //        breaker at (0.0, -250.0) with default dimensions (width 120, height 20).
    //        start_y = -250.0 + 10.0 + 8.0 + 3.0 = -229.0
    //        No pre-existing LastImpact on the bolt.
    // When: bolt_breaker_collision runs for one fixed tick
    // Then: Bolt entity has LastImpact with side: ImpactSide::Top
    //       and position.y approximately at breaker top surface (-250.0 + 10.0 = -240.0)
    let mut app = test_app();
    let breaker_y = -250.0;
    let hh = default_breaker_height();
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    let start_y = breaker_y + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after breaker top rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "breaker top rebound should stamp ImpactSide::Top, got {:?}",
        last_impact.side
    );
    let breaker_top_y = breaker_y + hh.half_height();
    assert!(
        (last_impact.position.y - breaker_top_y).abs() < 5.0,
        "LastImpact position.y should be near breaker top surface ({breaker_top_y}), got {}",
        last_impact.position.y
    );
}

#[test]
fn breaker_top_rebound_overwrites_existing_last_impact() {
    // Edge case: Bolt already has a LastImpact from a previous wall hit.
    // The breaker rebound should overwrite it completely.
    let mut app = test_app();
    let breaker_y = -250.0;
    let hh = default_breaker_height();
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    let start_y = breaker_y + hh.half_height() + default_bolt_radius().0 + 3.0;
    // Spawn bolt with a pre-existing LastImpact
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut().entity_mut(bolt_entity).insert(LastImpact {
        position: Vec2::new(999.0, 999.0),
        side:     ImpactSide::Left,
    });

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after breaker rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "breaker top rebound should overwrite old ImpactSide::Left with ImpactSide::Top"
    );
    assert!(
        last_impact.position.x.abs() < 50.0,
        "LastImpact position should be overwritten from (999, 999), got {:?}",
        last_impact.position
    );
}

// ── Behavior 2: side-surface rebound stamps LastImpact with Left or Right ──

#[test]
fn breaker_left_side_rebound_stamps_last_impact_with_left_side() {
    // Given: Bolt at (-71.0, -250.0) with velocity (400.0, -50.0),
    //        breaker at (0.0, -250.0) with default dimensions.
    //        The bolt starts OUTSIDE the breaker's expanded AABB
    //        (breaker_x - half_w - bolt_radius - 3.0 = 0 - 60 - 8 - 3 = -71),
    //        so CCD sweeps the bolt into the left side face.
    // When: bolt_breaker_collision runs for one fixed tick
    // Then: Bolt entity has LastImpact with side: ImpactSide::Left
    let mut app = test_app();
    let breaker_y = -250.0;
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    let bolt_entity = spawn_bolt(&mut app, -71.0, breaker_y, 400.0, -50.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after breaker side rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Left,
        "bolt hitting breaker left face should stamp ImpactSide::Left, got {:?}",
        last_impact.side
    );
}

#[test]
fn breaker_right_side_rebound_stamps_last_impact_with_right_side() {
    // Edge case: Bolt approaching from the right side should produce ImpactSide::Right
    let mut app = test_app();
    let breaker_y = -250.0;
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    // Mirror of the left-side test: bolt starts to the right of breaker
    let bolt_entity = spawn_bolt(&mut app, 71.0, breaker_y, -400.0, -50.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after breaker right side rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Right,
        "bolt hitting breaker right face should stamp ImpactSide::Right, got {:?}",
        last_impact.side
    );
}

// ── Behavior 15: overlap resolution stamps LastImpact ──

#[test]
fn overlap_resolution_downward_bolt_stamps_last_impact_top() {
    // Given: Bolt positioned inside the breaker's expanded AABB due to breaker movement.
    //        Velocity (0.0, -100.0) (moving downward).
    //        Breaker at (0.0, -250.0) with default dimensions.
    // When: bolt_breaker_collision runs for one fixed tick (triggers overlap resolution)
    // Then: Bolt entity has LastImpact with side: ImpactSide::Top
    let mut app = test_app();
    let breaker_y = -250.0;
    let hh = default_breaker_height();
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    // Position bolt inside the breaker's expanded AABB
    let inside_y = breaker_y + hh.half_height() + default_bolt_radius().0 - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, inside_y, 0.0, -100.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("overlap resolution should stamp LastImpact on downward bolt");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "overlap resolution should stamp ImpactSide::Top, got {:?}",
        last_impact.side
    );
    let breaker_top_y = breaker_y + hh.half_height();
    assert!(
        (last_impact.position.y - breaker_top_y).abs() < 5.0,
        "LastImpact position.y should be near breaker top surface ({breaker_top_y}), got {}",
        last_impact.position.y
    );
}

#[test]
fn overlap_resolution_upward_bolt_does_not_stamp_last_impact() {
    // Edge case: Bolt inside breaker AABB but moving upward (velocity.y > 0).
    //            Bolt is pushed above but NOT reflected, so LastImpact should NOT be stamped.
    let mut app = test_app();
    let breaker_y = -250.0;
    let hh = default_breaker_height();
    spawn_breaker_at(&mut app, 0.0, breaker_y);

    // Position bolt inside the breaker's expanded AABB
    let inside_y = breaker_y + hh.half_height() + default_bolt_radius().0 - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, inside_y, 0.0, 100.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "upward bolt in overlap should NOT get LastImpact (no rebound), but got {last_impact:?}"
    );
}
