use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{cells::messages::DamageCell, effect::effects::chain_lightning::tests::helpers::*};

// ── Behavior 3: fire() with arcs=0 does nothing ──

#[test]
fn fire_with_arcs_zero_does_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 0, 50.0, 1.0, 200.0, "", app.world_mut());

    // No ChainLightningChain entity
    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=0 should not spawn any chain entity"
    );

    // No DamageCell message
    let messages = app.world().resource::<Messages<DamageCell>>();
    assert!(
        messages.iter_current_update_messages().next().is_none(),
        "arcs=0 should not write any DamageCell message"
    );
}

#[test]
fn fire_with_arcs_zero_and_multiple_cells_does_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell_a = spawn_test_cell(&mut app, 10.0, 0.0);
    let _cell_b = spawn_test_cell(&mut app, 15.0, 0.0);
    let _cell_c = spawn_test_cell(&mut app, 20.0, 0.0);

    tick(&mut app);

    fire(entity, 0, 50.0, 1.0, 200.0, "", app.world_mut());

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=0 with multiple cells should still do nothing"
    );
}

// ── Behavior 4: fire() with arcs=1 damages first target, spawns no chain entity ──

#[test]
fn fire_with_arcs_one_damages_first_target_and_spawns_no_chain() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    // DamageCell should be written
    let messages = app.world().resource::<Messages<DamageCell>>();
    let written: Vec<&DamageCell> = messages.iter_current_update_messages().collect();
    assert_eq!(written.len(), 1, "arcs=1 should damage the first target");
    assert_eq!(written[0].cell, cell);

    // No chain entity (remaining_jumps would be 0)
    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=1 should not spawn a chain entity (remaining_jumps=0)"
    );
}

#[test]
fn fire_with_arcs_one_and_no_cells_in_range_does_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    // Cell far away
    let _cell = spawn_test_cell(&mut app, 500.0, 0.0);

    tick(&mut app);

    fire(entity, 1, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    assert!(
        messages.iter_current_update_messages().next().is_none(),
        "arcs=1 with no cells in range should not damage anything"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "arcs=1 with no cells in range should not spawn a chain"
    );
}

// ── Behavior 5: fire() with no valid targets in range ──

#[test]
fn fire_with_no_targets_in_range_damages_nothing_and_spawns_no_chain() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell = spawn_test_cell(&mut app, 500.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    assert!(
        messages.iter_current_update_messages().next().is_none(),
        "no targets in range should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "no targets in range should not spawn a chain"
    );
}

#[test]
fn fire_with_empty_quadtree_damages_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    // No cells at all
    tick(&mut app);

    fire(entity, 3, 50.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    assert!(
        messages.iter_current_update_messages().next().is_none(),
        "empty quadtree should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "empty quadtree should not spawn a chain"
    );
}

// ── Behavior 6: fire() with range=0 or negative range ──

#[test]
fn fire_with_zero_range_damages_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell = spawn_test_cell(&mut app, 0.0, 0.0);

    tick(&mut app);

    fire(entity, 3, 0.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    assert!(
        messages.iter_current_update_messages().next().is_none(),
        "range=0.0 should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "range=0.0 should not spawn a chain"
    );
}

#[test]
fn fire_with_negative_range_damages_nothing() {
    let mut app = chain_lightning_test_app();

    let entity = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let _cell = spawn_test_cell(&mut app, 0.0, 0.0);

    tick(&mut app);

    fire(entity, 3, -5.0, 1.0, 200.0, "", app.world_mut());

    let messages = app.world().resource::<Messages<DamageCell>>();
    assert!(
        messages.iter_current_update_messages().next().is_none(),
        "negative range should produce no DamageCell"
    );

    let mut chain_query = app.world_mut().query::<&ChainLightningChain>();
    assert!(
        chain_query.iter(app.world()).next().is_none(),
        "negative range should not spawn a chain"
    );
}
