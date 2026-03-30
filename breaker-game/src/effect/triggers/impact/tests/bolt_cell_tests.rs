//! Tests for `bridge_impact_bolt_cell` -- fires globally on both `Impact(Cell)` and `Impact(Bolt)`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::BoltImpactCell,
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

#[test]
fn bridge_impact_bolt_cell_fires_impact_cell_globally() {
    let mut app = test_app_bolt_cell();

    let bolt = app.world_mut().spawn_empty().id();
    let cell = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

    // Entity listening for Impact(Cell) -- should fire
    app.world_mut().spawn((
        impact_cell_bound_effects(),
        StagedEffects::default(),
        ActiveSpeedBoosts(vec![]),
    ));

    tick(&mut app);

    let active = app
        .world_mut()
        .query::<&ActiveSpeedBoosts>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        active.0.len(),
        1,
        "bridge_impact_bolt_cell should fire Impact(Cell) globally on BoltImpactCell"
    );
}

#[test]
fn bridge_impact_bolt_cell_also_fires_impact_bolt_globally() {
    let mut app = test_app_bolt_cell();

    let bolt = app.world_mut().spawn_empty().id();
    let cell = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

    // Entity listening for Impact(Bolt) -- should also fire
    app.world_mut().spawn((
        impact_bolt_bound_effects(),
        StagedEffects::default(),
        ActiveSpeedBoosts(vec![]),
    ));

    tick(&mut app);

    let active = app
        .world_mut()
        .query::<&ActiveSpeedBoosts>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        active.0.len(),
        1,
        "bridge_impact_bolt_cell should also fire Impact(Bolt) globally"
    );
}

#[test]
fn bridge_impact_bolt_cell_no_message_no_fire() {
    let mut app = test_app_bolt_cell();

    app.insert_resource(TestBoltImpactCellMsg(None));

    app.world_mut().spawn((
        impact_cell_bound_effects(),
        StagedEffects::default(),
        ActiveSpeedBoosts(vec![]),
    ));

    tick(&mut app);

    let active = app
        .world_mut()
        .query::<&ActiveSpeedBoosts>()
        .single(app.world())
        .unwrap();
    assert_eq!(
        active.0.len(),
        0,
        "No BoltImpactCell message means no Impact trigger should fire"
    );
}
