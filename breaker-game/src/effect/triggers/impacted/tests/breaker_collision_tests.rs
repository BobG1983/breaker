//! Tests for `bridge_impacted_breaker_cell` and `bridge_impacted_breaker_wall`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

// =========================================================================
// bridge_impacted_breaker_cell
// =========================================================================

#[test]
fn bridge_impacted_breaker_cell_fires_impacted_cell_on_breaker() {
    let mut app = test_app_breaker_cell();

    let breaker = app
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

    app.insert_resource(TestBreakerImpactCellMsg(Some(BreakerImpactCell {
        breaker,
        cell,
    })));

    tick(&mut app);

    let breaker_active = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
    assert_eq!(
        breaker_active.0.len(),
        1,
        "bridge_impacted_breaker_cell should fire Impacted(Cell) on the breaker entity"
    );
}

// =========================================================================
// bridge_impacted_breaker_wall
// =========================================================================

#[test]
fn bridge_impacted_breaker_wall_fires_impacted_wall_on_breaker() {
    let mut app = test_app_breaker_wall();

    let breaker = app
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

    app.insert_resource(TestBreakerImpactWallMsg(Some(BreakerImpactWall {
        breaker,
        wall,
    })));

    tick(&mut app);

    let breaker_active = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
    assert_eq!(
        breaker_active.0.len(),
        1,
        "bridge_impacted_breaker_wall should fire Impacted(Wall) on the breaker entity"
    );
}
