//! Basic piercing behavior — non-piercing reflects, piercing passes through
//! destroyable cells, piercing reflects off tough cells.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::bolt::{
    components::PiercingRemaining, systems::bolt_cell_collision::tests::helpers::*,
    test_utils::piercing_stack,
};

#[test]
fn non_piercing_bolt_reflects_off_cell() {
    // Non-piercing bolt hitting a cell reflects (velocity.y < 0 after upward approach).
    // BoltImpactCell is sent. No PiercingRemaining component involved.
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    // CellHealth(30) — bolt deals base 10, survives.
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    // No PiercingRemaining or ActivePiercings component
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
        "non-piercing bolt should reflect downward off cell, got vy={}",
        vel.0.y
    );

    let hits = app.world().resource::<HitCells>();
    assert_eq!(hits.0.len(), 1, "BoltImpactCell should be sent");
}

/// Spec behavior 8: `bolt_cell_collision` uses `ActivePiercings` for pierce check.
/// Bolt with `piercing_stack(&[2])`, `PiercingRemaining(2)`, cell with `CellHealth(10.0)`.
/// Bolt pierces through, `PiercingRemaining` decremented to 1.
#[test]
fn piercing_bolt_passes_through_cell_it_would_destroy() {
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[2]), PiercingRemaining(2)));

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
        "BoltImpactCell should be sent for pierced cell"
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining should decrement from 2 to 1 after one pierce"
    );
}

#[test]
fn piercing_bolt_reflects_off_cell_it_would_not_destroy() {
    // Bolt with PiercingRemaining(1), no ActiveDamageBoosts.
    // Cell with CellHealth(30) — base damage 10, cell survives.
    // Bolt should reflect (velocity.y < 0). PiercingRemaining stays 1.
    let mut app = test_app();
    let bc = crate::bolt::systems::bolt_cell_collision::tests::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[1]), PiercingRemaining(1)));

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
