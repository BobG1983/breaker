//! Tests for `bridge_impacted_bolt_cell` -- targeted on both participants.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::BoltImpactCell,
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

#[test]
fn bridge_impacted_bolt_cell_fires_impacted_cell_on_bolt_entity() {
    let mut app = test_app_bolt_cell();

    let bolt = app
        .world_mut()
        .spawn((
            impacted_cell_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

    tick(&mut app);

    let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert_eq!(
        bolt_active.0.len(),
        1,
        "bridge_impacted_bolt_cell should fire Impacted(Cell) on the bolt entity"
    );
    assert!(
        (bolt_active.0[0] - 1.5).abs() < f32::EPSILON,
        "SpeedBoost multiplier should be 1.5"
    );

    let cell_active = app.world().get::<ActiveSpeedBoosts>(cell).unwrap();
    assert_eq!(
        cell_active.0.len(),
        0,
        "Cell entity has no Impacted(Cell) chain and should not be affected"
    );
}

#[test]
fn bridge_impacted_bolt_cell_fires_impacted_bolt_on_cell_entity() {
    let mut app = test_app_bolt_cell();

    let bolt = app
        .world_mut()
        .spawn((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn((
            impacted_bolt_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

    tick(&mut app);

    let cell_active = app.world().get::<ActiveSpeedBoosts>(cell).unwrap();
    assert_eq!(
        cell_active.0.len(),
        1,
        "bridge_impacted_bolt_cell should fire Impacted(Bolt) on the cell entity"
    );

    let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert_eq!(
        bolt_active.0.len(),
        0,
        "Bolt entity has no Impacted(Bolt) chain and should not be affected"
    );
}

#[test]
fn bridge_impacted_bolt_cell_does_not_fire_on_uninvolved_entity() {
    let mut app = test_app_bolt_cell();

    let bolt = app
        .world_mut()
        .spawn((
            impacted_cell_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn((
            impacted_bolt_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    // Third entity -- not involved in the collision, should NOT fire
    app.world_mut().spawn((
        impacted_cell_bound_effects(),
        StagedEffects::default(),
        ActiveSpeedBoosts(vec![]),
    ));

    app.insert_resource(TestBoltImpactCellMsg(Some(BoltImpactCell { cell, bolt })));

    tick(&mut app);

    // Count entities with non-empty ActiveSpeedBoosts
    let mut affected_count = 0;
    for active in app
        .world_mut()
        .query::<&ActiveSpeedBoosts>()
        .iter(app.world())
    {
        if !active.0.is_empty() {
            affected_count += 1;
        }
    }
    assert_eq!(
        affected_count, 2,
        "Only the bolt and cell from the message should be affected (targeted, not global)"
    );
}
