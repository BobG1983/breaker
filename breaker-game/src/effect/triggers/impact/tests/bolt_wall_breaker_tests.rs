//! Tests for `bridge_impact_bolt_wall` and `bridge_impact_bolt_breaker`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactWall},
    effect::{core::*, effects::speed_boost::ActiveSpeedBoosts},
};

// =========================================================================
// bridge_impact_bolt_wall
// =========================================================================

#[test]
fn bridge_impact_bolt_wall_fires_impact_wall_globally() {
    let mut app = test_app_bolt_wall();

    let bolt = app.world_mut().spawn_empty().id();
    let wall = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBoltImpactWallMsg(Some(BoltImpactWall { bolt, wall })));

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
        "bridge_impact_bolt_wall should fire Impact(Wall) globally on BoltImpactWall"
    );
}

// =========================================================================
// bridge_impact_bolt_breaker
// =========================================================================

#[test]
fn bridge_impact_bolt_breaker_fires_impact_breaker_globally() {
    let mut app = test_app_bolt_breaker();

    let bolt = app.world_mut().spawn_empty().id();
    let breaker = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBoltImpactBreakerMsg(Some(BoltImpactBreaker {
        bolt,
        breaker,
    })));

    app.world_mut().spawn((
        impact_breaker_bound_effects(),
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
        "bridge_impact_bolt_breaker should fire Impact(Breaker) globally on BoltImpactBreaker"
    );
}
