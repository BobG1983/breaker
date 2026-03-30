//! Tests for `bridge_impacted_bolt_wall` and `bridge_impacted_bolt_breaker`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactWall},
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

// =========================================================================
// bridge_impacted_bolt_wall
// =========================================================================

#[test]
fn bridge_impacted_bolt_wall_fires_impacted_wall_on_bolt() {
    let mut app = test_app_bolt_wall();

    let bolt = app
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

    app.insert_resource(TestBoltImpactWallMsg(Some(BoltImpactWall { bolt, wall })));

    tick(&mut app);

    let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert_eq!(
        bolt_active.0.len(),
        1,
        "bridge_impacted_bolt_wall should fire Impacted(Wall) on the bolt entity"
    );
}

// =========================================================================
// bridge_impacted_bolt_breaker
// =========================================================================

#[test]
fn bridge_impacted_bolt_breaker_fires_impacted_breaker_on_bolt() {
    let mut app = test_app_bolt_breaker();

    let bolt = app
        .world_mut()
        .spawn((
            impacted_breaker_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    let breaker = app
        .world_mut()
        .spawn((
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    app.insert_resource(TestBoltImpactBreakerMsg(Some(BoltImpactBreaker {
        bolt,
        breaker,
    })));

    tick(&mut app);

    let bolt_active = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert_eq!(
        bolt_active.0.len(),
        1,
        "bridge_impacted_bolt_breaker should fire Impacted(Breaker) on the bolt entity"
    );
}
