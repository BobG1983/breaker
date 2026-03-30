//! Tests for `bridge_impact_cell_wall`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::messages::CellImpactWall,
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

#[test]
fn bridge_impact_cell_wall_fires_impact_wall_globally() {
    let mut app = test_app_cell_wall();

    let cell = app.world_mut().spawn_empty().id();
    let wall = app.world_mut().spawn_empty().id();

    app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall { cell, wall })));

    app.world_mut().spawn((
        impact_wall_bound_effects(),
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
        "bridge_impact_cell_wall should fire Impact(Wall) globally on CellImpactWall"
    );
}

#[test]
fn bridge_impact_cell_wall_also_fires_impact_cell_globally() {
    let mut app = test_app_cell_wall();

    let cell = app.world_mut().spawn_empty().id();
    let wall = app.world_mut().spawn_empty().id();

    app.insert_resource(TestCellImpactWallMsg(Some(CellImpactWall { cell, wall })));

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
        "bridge_impact_cell_wall should also fire Impact(Cell) globally"
    );
}
