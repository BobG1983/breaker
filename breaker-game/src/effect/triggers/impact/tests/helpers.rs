//! Shared test infrastructure for `impact` bridge system tests.

use bevy::prelude::*;

use super::super::system::*;
use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::CellImpactWall,
    effect::core::*,
};

// -- BoltImpactCell helper --

#[derive(Resource)]
pub(super) struct TestBoltImpactCellMsg(pub(super) Option<BoltImpactCell>);

pub(super) fn enqueue_bolt_impact_cell(
    msg_res: Res<TestBoltImpactCellMsg>,
    mut writer: MessageWriter<BoltImpactCell>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn test_app_bolt_cell() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactCell>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_bolt_impact_cell.before(bridge_impact_bolt_cell),
                bridge_impact_bolt_cell,
            ),
        );
    app
}

// -- BoltImpactWall helper --

#[derive(Resource)]
pub(super) struct TestBoltImpactWallMsg(pub(super) Option<BoltImpactWall>);

pub(super) fn enqueue_bolt_impact_wall(
    msg_res: Res<TestBoltImpactWallMsg>,
    mut writer: MessageWriter<BoltImpactWall>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn test_app_bolt_wall() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactWall>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_bolt_impact_wall.before(bridge_impact_bolt_wall),
                bridge_impact_bolt_wall,
            ),
        );
    app
}

// -- BoltImpactBreaker helper --

#[derive(Resource)]
pub(super) struct TestBoltImpactBreakerMsg(pub(super) Option<BoltImpactBreaker>);

pub(super) fn enqueue_bolt_impact_breaker(
    msg_res: Res<TestBoltImpactBreakerMsg>,
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn test_app_bolt_breaker() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactBreaker>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_bolt_impact_breaker.before(bridge_impact_bolt_breaker),
                bridge_impact_bolt_breaker,
            ),
        );
    app
}

// -- BreakerImpactCell helper --

#[derive(Resource)]
pub(super) struct TestBreakerImpactCellMsg(pub(super) Option<BreakerImpactCell>);

pub(super) fn enqueue_breaker_impact_cell(
    msg_res: Res<TestBreakerImpactCellMsg>,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn test_app_breaker_cell() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerImpactCell>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_breaker_impact_cell.before(bridge_impact_breaker_cell),
                bridge_impact_breaker_cell,
            ),
        );
    app
}

// -- BreakerImpactWall helper --

#[derive(Resource)]
pub(super) struct TestBreakerImpactWallMsg(pub(super) Option<BreakerImpactWall>);

pub(super) fn enqueue_breaker_impact_wall(
    msg_res: Res<TestBreakerImpactWallMsg>,
    mut writer: MessageWriter<BreakerImpactWall>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn test_app_breaker_wall() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerImpactWall>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_breaker_impact_wall.before(bridge_impact_breaker_wall),
                bridge_impact_breaker_wall,
            ),
        );
    app
}

// -- CellImpactWall helper --

#[derive(Resource)]
pub(super) struct TestCellImpactWallMsg(pub(super) Option<CellImpactWall>);

pub(super) fn enqueue_cell_impact_wall(
    msg_res: Res<TestCellImpactWallMsg>,
    mut writer: MessageWriter<CellImpactWall>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn test_app_cell_wall() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellImpactWall>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_cell_impact_wall.before(bridge_impact_cell_wall),
                bridge_impact_cell_wall,
            ),
        );
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn impact_cell_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

pub(super) fn impact_bolt_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

pub(super) fn impact_wall_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Wall),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

pub(super) fn impact_breaker_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Breaker),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}
