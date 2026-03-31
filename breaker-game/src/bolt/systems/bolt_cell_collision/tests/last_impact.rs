//! `LastImpact` stamping tests for bolt-cell collision.
//!
//! Verifies that `LastImpact { position, side }` is inserted/updated on the
//! bolt entity when the bolt rebounds off a cell surface, and NOT inserted
//! when the bolt pierces through a cell.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::components::{Bolt, ImpactSide, LastImpact, PiercingRemaining},
    effect::EffectivePiercing,
};

// ── Behavior 3: bottom face rebound stamps ImpactSide::Bottom ──

#[test]
fn cell_bottom_rebound_stamps_last_impact_with_bottom_side() {
    // Given: Bolt at (0.0, 78.0) with velocity (0.0, 400.0) moving upward,
    //        cell at (0.0, 100.0) with CellHealth(30.0).
    //        start_y = 100.0 - 12.0 - 8.0 - 2.0 = 78.0
    //        Bolt starts OUTSIDE the cell's expanded AABB so CCD fires.
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Bottom
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
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after cell bottom rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Bottom,
        "cell bottom rebound should stamp ImpactSide::Bottom, got {:?}",
        last_impact.side
    );
    // Contact point should be approximately at the cell's bottom face
    let cell_bottom = cell_y - cc.height / 2.0;
    assert!(
        (last_impact.position.y - cell_bottom).abs() < 10.0,
        "LastImpact position.y should be near cell bottom ({cell_bottom}), got {}",
        last_impact.position.y
    );
}

// ── Behavior 4: top face rebound stamps ImpactSide::Top ──

#[test]
fn cell_top_rebound_stamps_last_impact_with_top_side() {
    // Given: Bolt at (0.0, 122.0) with velocity (0.0, -400.0) moving downward,
    //        cell at (0.0, 100.0) with CellHealth(30.0).
    //        start_y = 100.0 + 12.0 + 8.0 + 2.0 = 122.0
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Top
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y + cc.height / 2.0 + bc.radius + 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, -400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after cell top rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "cell top rebound should stamp ImpactSide::Top, got {:?}",
        last_impact.side
    );
}

#[test]
fn cell_top_rebound_side_determined_by_normal_not_velocity() {
    // Edge case: Velocity nearly horizontal (399.0, -20.0) but hitting cell top.
    // Side should still be determined by the hit normal, not by velocity direction.
    // NOTE: The bolt barely moves vertically per tick (~0.31 units at 64 Hz),
    // so it must start very close to the expanded AABB top edge.
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y + cc.height / 2.0 + bc.radius + 0.1;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(399.0, -20.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after cell rebound with nearly horizontal velocity");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "side should be determined by collision normal (Top), not velocity direction, got {:?}",
        last_impact.side
    );
}

// ── Behavior 5: left face rebound stamps ImpactSide::Left ──

#[test]
fn cell_left_rebound_stamps_last_impact_with_left_side() {
    // Given: Bolt at (55.0, 100.0) with velocity (400.0, 0.0) moving rightward,
    //        cell at (100.0, 100.0) with CellHealth(30.0).
    //        start_x = 100.0 - 35.0 - 8.0 - 2.0 = 55.0
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Left
    let mut app = test_app();

    let cell_x = 100.0;
    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, cell_x, cell_y, 30.0);

    let cc = crate::cells::resources::CellConfig::default();
    let bc = crate::bolt::resources::BoltConfig::default();
    let start_x = cell_x - cc.width / 2.0 - bc.radius - 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.0)),
            Position2D(Vec2::new(start_x, cell_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after cell left rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Left,
        "cell left rebound should stamp ImpactSide::Left, got {:?}",
        last_impact.side
    );
}

// ── Behavior 6: right face rebound stamps ImpactSide::Right ──

#[test]
fn cell_right_rebound_stamps_last_impact_with_right_side() {
    // Given: Bolt at (145.0, 100.0) with velocity (-400.0, 0.0) moving leftward,
    //        cell at (100.0, 100.0) with CellHealth(30.0).
    //        start_x = 100.0 + 35.0 + 8.0 + 2.0 = 145.0
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Right
    let mut app = test_app();

    let cell_x = 100.0;
    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, cell_x, cell_y, 30.0);

    let cc = crate::cells::resources::CellConfig::default();
    let bc = crate::bolt::resources::BoltConfig::default();
    let start_x = cell_x + cc.width / 2.0 + bc.radius + 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(-400.0, 0.0)),
            Position2D(Vec2::new(start_x, cell_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after cell right rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Right,
        "cell right rebound should stamp ImpactSide::Right, got {:?}",
        last_impact.side
    );
}

// ── Behavior 10: piercing bolt passing through destroyable cell does NOT stamp ──

#[test]
fn piercing_bolt_through_destroyable_cell_does_not_stamp_last_impact() {
    // Given: Bolt with EffectivePiercing(2), PiercingRemaining(2).
    //        Cell at (0.0, 100.0) with CellHealth(10.0) — bolt base damage 10.0 would destroy it.
    //        No pre-existing LastImpact on the bolt.
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt entity does NOT have a LastImpact component
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            EffectivePiercing(2),
            PiercingRemaining(2),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "piercing bolt passing through destroyable cell should NOT get LastImpact, got {last_impact:?}"
    );
}

#[test]
fn piercing_bolt_through_cell_preserves_existing_last_impact() {
    // Edge case: Bolt has a pre-existing LastImpact from a previous wall hit.
    //            Pierce-through must leave it unchanged.
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            EffectivePiercing(2),
            PiercingRemaining(2),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();
    // Insert a pre-existing LastImpact
    let original = LastImpact {
        position: Vec2::new(50.0, 300.0),
        side: ImpactSide::Top,
    };
    app.world_mut().entity_mut(bolt_entity).insert(original);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("pre-existing LastImpact should still be present after pierce-through");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "pre-existing LastImpact side should be unchanged after pierce-through"
    );
    assert!(
        (last_impact.position - Vec2::new(50.0, 300.0)).length() < 0.01,
        "pre-existing LastImpact position should be unchanged after pierce-through, got {:?}",
        last_impact.position
    );
}

// ── Behavior 11: piercing bolt reflecting off tough cell DOES stamp ──

#[test]
fn piercing_bolt_reflecting_off_tough_cell_stamps_last_impact() {
    // Given: Bolt with EffectivePiercing(1), PiercingRemaining(1).
    //        Cell at (0.0, 100.0) with CellHealth(30.0) — bolt base damage 10.0, cell survives.
    //        Cannot pierce because cell would survive.
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Bottom (reflected off bottom face)
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
            EffectivePiercing(1),
            PiercingRemaining(1),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("piercing bolt reflecting off tough cell should have LastImpact");
    assert_eq!(
        last_impact.side,
        ImpactSide::Bottom,
        "should stamp ImpactSide::Bottom when reflecting off tough cell's bottom face, got {:?}",
        last_impact.side
    );
}

// ── Behavior 12: exhausted piercing reflects and stamps ──

#[test]
fn exhausted_piercing_bolt_reflecting_off_destroyable_cell_stamps_last_impact() {
    // Given: Bolt with EffectivePiercing(1), PiercingRemaining(0).
    //        Cell at (0.0, 100.0) with CellHealth(10.0) — bolt base damage 10.0 would destroy it,
    //        but piercing is exhausted so it cannot pierce through.
    // When: bolt_cell_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Bottom (reflected)
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            EffectivePiercing(1),
            PiercingRemaining(0),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("exhausted piercing bolt should have LastImpact after reflect");
    assert_eq!(
        last_impact.side,
        ImpactSide::Bottom,
        "exhausted piercing bolt reflecting off cell bottom should stamp ImpactSide::Bottom, got {:?}",
        last_impact.side
    );
}

#[test]
fn exhausted_piercing_zero_effective_also_reflects_and_stamps() {
    // Edge case: PiercingRemaining(0) with EffectivePiercing(0) — same behavior.
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            EffectivePiercing(0),
            PiercingRemaining(0),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt with EP(0)/PR(0) should have LastImpact after reflect");
    assert_eq!(
        last_impact.side,
        ImpactSide::Bottom,
        "bolt with EP(0)/PR(0) reflecting off cell bottom should stamp ImpactSide::Bottom, got {:?}",
        last_impact.side
    );
}
