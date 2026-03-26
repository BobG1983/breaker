use bevy::prelude::*;

use super::{super::test_helpers::*, bridge::*};
use crate::{
    bolt::messages::{BoltHitBreaker, BoltHitCell, BoltHitWall},
    effect::{
        definition::{Effect, EffectChains, EffectNode, ImpactTarget, Trigger},
        typed_events::*,
    },
};

// --- Test infrastructure ---

#[derive(Resource, Default)]
struct CapturedSpeedBoostFired(Vec<SpeedBoostFired>);

fn capture_speed_boost_fired(
    trigger: On<SpeedBoostFired>,
    mut captured: ResMut<CapturedSpeedBoostFired>,
) {
    captured.0.push(trigger.event().clone());
}

#[derive(Resource)]
struct SendBoltHitCell(Option<BoltHitCell>);

fn send_bolt_hit_cell(msg: Res<SendBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
    if let Some(m) = msg.0.clone() {
        writer.write(m);
    }
}

#[derive(Resource)]
struct SendBoltHitWall(Option<BoltHitWall>);

fn send_bolt_hit_wall(msg: Res<SendBoltHitWall>, mut writer: MessageWriter<BoltHitWall>) {
    if let Some(m) = msg.0.clone() {
        writer.write(m);
    }
}

#[derive(Resource)]
struct SendBoltHitBreaker(Option<BoltHitBreaker>);

fn send_bolt_hit_breaker(msg: Res<SendBoltHitBreaker>, mut writer: MessageWriter<BoltHitBreaker>) {
    if let Some(m) = msg.0.clone() {
        writer.write(m);
    }
}

fn cell_impacted_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltHitCell>()
        .insert_resource(SendBoltHitCell(None))
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(
            FixedUpdate,
            (send_bolt_hit_cell, bridge_cell_impacted).chain(),
        );
    app
}

fn wall_impacted_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltHitWall>()
        .insert_resource(SendBoltHitWall(None))
        .init_resource::<CapturedSpeedBoostFired>()
        .add_observer(capture_speed_boost_fired)
        .add_systems(
            FixedUpdate,
            (send_bolt_hit_wall, bridge_wall_impacted).chain(),
        );
    app
}

fn breaker_impacted_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltHitBreaker>()
        .insert_resource(SendBoltHitBreaker(None))
        .init_resource::<CapturedShockwaveFired>()
        .add_observer(capture_shockwave_fired)
        .add_systems(
            FixedUpdate,
            (send_bolt_hit_breaker, bridge_breaker_impacted).chain(),
        );
    app
}

// --- M6: bridge_breaker_impacted does NOT evaluate breaker entity ---

/// M6: Breaker entity with When(Impacted(Breaker)) chains is not reachable
/// from `BoltHitBreaker` — only the bolt entity is checked. Bolt has no chains,
/// so nothing fires.
#[test]
fn bridge_breaker_impacted_does_not_evaluate_breaker_entity() {
    use crate::breaker::components::Breaker;

    let mut app = breaker_impacted_test_app();

    // Breaker entity with Impacted(Breaker) chain — should NOT fire
    app.world_mut().spawn((
        Breaker,
        EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impacted(ImpactTarget::Breaker),
                Effect::test_shockwave(64.0),
            ),
        )]),
    ));

    // Bolt entity with NO EffectChains
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert!(
        captured.0.is_empty(),
        "bridge_breaker_impacted should NOT evaluate breaker entity — only bolt is checked. Got {}",
        captured.0.len()
    );
}

// --- Cell impacted bridge tests ---

/// Bolt has `When(Impacted(Cell))`, Cell has `When(Impacted(Cell))`.
/// Both fire on `BoltHitCell`. An unrelated third entity does NOT fire.
#[test]
fn bridge_cell_impacted_evaluates_both_bolt_and_cell() {
    let mut app = cell_impacted_test_app();

    // Bolt with Impacted(Cell) chain
    let bolt = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impacted(ImpactTarget::Cell),
                Effect::test_shockwave(64.0),
            ),
        )]))
        .id();

    // Cell with Impacted(Cell) chain
    let cell = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impacted(ImpactTarget::Cell),
                Effect::test_shockwave(32.0),
            ),
        )]))
        .id();

    // Unrelated entity with Impacted(Cell) — should NOT fire
    app.world_mut().spawn(EffectChains(vec![(
        None,
        EffectNode::trigger_leaf(
            Trigger::Impacted(ImpactTarget::Cell),
            Effect::test_shockwave(16.0),
        ),
    )]));

    app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell { cell, bolt });
    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        2,
        "both bolt and cell should fire Impacted(Cell) — unrelated entity should NOT. Got {}",
        captured.0.len()
    );
}

// --- Wall impacted bridge tests ---

/// Bolt has `When(Impacted(Wall))` — fires on `BoltHitWall`.
#[test]
fn bridge_wall_impacted_evaluates_bolt_and_wall() {
    let mut app = wall_impacted_test_app();

    // Bolt with Impacted(Wall) chain
    let bolt = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impacted(ImpactTarget::Wall),
                Effect::test_speed_boost(1.5),
            ),
        )]))
        .id();

    // Wall with Impacted(Wall) chain
    let wall = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impacted(ImpactTarget::Wall),
                Effect::test_speed_boost(1.3),
            ),
        )]))
        .id();

    app.world_mut().resource_mut::<SendBoltHitWall>().0 = Some(BoltHitWall { bolt, wall });
    tick(&mut app);

    let captured = app.world().resource::<CapturedSpeedBoostFired>();
    assert_eq!(
        captured.0.len(),
        2,
        "both bolt and wall should fire Impacted(Wall) — got {}",
        captured.0.len()
    );
}

// --- Breaker impacted bridge tests ---

/// Bolt has `When(Impacted(Breaker))` — fires on `BoltHitBreaker`.
#[test]
fn bridge_breaker_impacted_evaluates_bolt_and_breaker() {
    let mut app = breaker_impacted_test_app();

    // Bolt with Impacted(Breaker) chain
    let bolt = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::trigger_leaf(
                Trigger::Impacted(ImpactTarget::Breaker),
                Effect::test_shockwave(64.0),
            ),
        )]))
        .id();

    app.world_mut().resource_mut::<SendBoltHitBreaker>().0 = Some(BoltHitBreaker { bolt });
    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        1,
        "bolt should fire Impacted(Breaker) — got {}",
        captured.0.len()
    );
    assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
}

// =========================================================================
// H2: Once(When) consumption through bridge_cell_impacted
// =========================================================================

/// Bolt has `Once([When(Impacted(Cell), [Do(Shockwave(64.0))])])`.
/// First `BoltHitCell` fires the shockwave and consumes the Once.
/// Second `BoltHitCell` produces no additional shockwave.
#[test]
fn once_when_consumed_through_bridge_cell_impacted() {
    let mut app = cell_impacted_test_app();

    // Bolt with Once([When(Impacted(Cell), [Do(Shockwave(64.0))])])
    let bolt = app
        .world_mut()
        .spawn(EffectChains(vec![(
            None,
            EffectNode::Once(vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }]),
        )]))
        .id();

    let cell = app.world_mut().spawn_empty().id();

    // First BoltHitCell — should fire and consume
    app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell { cell, bolt });
    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        1,
        "first BoltHitCell should fire ShockwaveFired(64.0) — got {}",
        captured.0.len()
    );
    assert!(
        (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
        "should fire shockwave with base_range 64.0"
    );

    // EffectChains should be empty (Once consumed)
    let chains = app.world().get::<EffectChains>(bolt).unwrap();
    assert!(
        chains.0.is_empty(),
        "Once node should be consumed after first match — got {} entries",
        chains.0.len()
    );

    // Second BoltHitCell — should NOT fire again
    app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell { cell, bolt });
    tick(&mut app);

    let captured = app.world().resource::<CapturedShockwaveFired>();
    assert_eq!(
        captured.0.len(),
        1,
        "second BoltHitCell should NOT fire — Once already consumed — got {}",
        captured.0.len()
    );
}
