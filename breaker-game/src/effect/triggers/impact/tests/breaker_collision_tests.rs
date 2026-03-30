//! Tests for `bridge_impact_breaker_cell` and `bridge_impact_breaker_wall`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

// =========================================================================
// bridge_impact_breaker_cell
// =========================================================================

#[test]
fn bridge_impact_breaker_cell_fires_impact_cell_globally() {
    let mut app = test_app_breaker_cell();

    let breaker = app.world_mut().spawn_empty().id();
    let cell = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBreakerImpactCellMsg(Some(BreakerImpactCell {
        breaker,
        cell,
    })));

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
        "bridge_impact_breaker_cell should fire Impact(Cell) globally on BreakerImpactCell"
    );
}

// =========================================================================
// bridge_impact_breaker_wall
// =========================================================================

#[test]
fn bridge_impact_breaker_wall_fires_impact_wall_globally() {
    let mut app = test_app_breaker_wall();

    let breaker = app.world_mut().spawn_empty().id();
    let wall = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBreakerImpactWallMsg(Some(BreakerImpactWall {
        breaker,
        wall,
    })));

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
        "bridge_impact_breaker_wall should fire Impact(Wall) globally on BreakerImpactWall"
    );
}
