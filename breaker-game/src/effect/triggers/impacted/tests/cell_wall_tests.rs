//! Tests for `bridge_impacted_cell_wall`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::messages::CellImpactWall,
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

#[test]
fn bridge_impacted_cell_wall_fires_impacted_wall_on_cell() {
    let mut app = test_app_cell_wall();

    let cell = app
        .world_mut()
        .spawn((
            impacted_wall_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    let wall = app
        .world_mut()
        .spawn((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall { cell, wall })));

    tick(&mut app);

    let cell_active = app.world().get::<ActiveSpeedBoosts>(cell).unwrap();
    assert_eq!(
        cell_active.0.len(),
        1,
        "bridge_impacted_cell_wall should fire Impacted(Wall) on the cell entity"
    );
}

#[test]
fn bridge_impacted_cell_wall_fires_impacted_cell_on_wall() {
    let mut app = test_app_cell_wall();

    let cell = app
        .world_mut()
        .spawn((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    let wall = app
        .world_mut()
        .spawn((
            impacted_cell_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall { cell, wall })));

    tick(&mut app);

    let wall_active = app.world().get::<ActiveSpeedBoosts>(wall).unwrap();
    assert_eq!(
        wall_active.0.len(),
        1,
        "bridge_impacted_cell_wall should fire Impacted(Cell) on the wall entity"
    );
}
