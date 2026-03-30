use bevy::prelude::*;

use super::system::*;
use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::CellImpactWall,
    effect::{
        core::*,
        effects::speed_boost::ActiveSpeedBoosts,
    },
};

// -- BoltImpactCell helper --

#[derive(Resource)]
struct TestBoltImpactCellMsg(Option<BoltImpactCell>);

fn enqueue_bolt_impact_cell(
    msg_res: Res<TestBoltImpactCellMsg>,
    mut writer: MessageWriter<BoltImpactCell>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app_bolt_cell() -> App {
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
struct TestBoltImpactWallMsg(Option<BoltImpactWall>);

fn enqueue_bolt_impact_wall(
    msg_res: Res<TestBoltImpactWallMsg>,
    mut writer: MessageWriter<BoltImpactWall>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app_bolt_wall() -> App {
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
struct TestBoltImpactBreakerMsg(Option<BoltImpactBreaker>);

fn enqueue_bolt_impact_breaker(
    msg_res: Res<TestBoltImpactBreakerMsg>,
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app_bolt_breaker() -> App {
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
struct TestBreakerImpactCellMsg(Option<BreakerImpactCell>);

fn enqueue_breaker_impact_cell(
    msg_res: Res<TestBreakerImpactCellMsg>,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app_breaker_cell() -> App {
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
struct TestBreakerImpactWallMsg(Option<BreakerImpactWall>);

fn enqueue_breaker_impact_wall(
    msg_res: Res<TestBreakerImpactWallMsg>,
    mut writer: MessageWriter<BreakerImpactWall>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app_breaker_wall() -> App {
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
struct TestCellImpactWallMsg(Option<CellImpactWall>);

fn enqueue_cell_impact_wall(
    msg_res: Res<TestCellImpactWallMsg>,
    mut writer: MessageWriter<CellImpactWall>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

fn test_app_cell_wall() -> App {
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

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn impact_cell_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

fn impact_bolt_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

fn impact_wall_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Wall),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

fn impact_breaker_bound_effects() -> BoundEffects {
    BoundEffects(vec![(
        "test".into(),
        EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Breaker),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
    )])
}

// =========================================================================
// bridge_impact_bolt_cell
// =========================================================================

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

// =========================================================================
// bridge_impact_cell_wall
// =========================================================================

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
