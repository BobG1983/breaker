use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::*;
use crate::{
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
    chips::components::DamageBoost,
    effect::{
        definition::{Effect, EffectNode, ImpactTarget, Trigger},
        effect_nodes::until::system::*,
        effects::{damage_boost::ActiveDamageBoosts, piercing::ActivePiercings},
    },
};

// --- Behavior 8: Until removal with OnImpact(Cell) replaces OneShotDamageBoost ---

#[test]
fn check_until_triggers_removes_damage_boost_on_cell_impact() {
    use crate::bolt::messages::BoltHitCell;

    #[derive(Resource)]
    struct SendMsg(Option<BoltHitCell>);

    fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitCell>();
    app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            DamageBoost(2.0),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                children: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
            }]),
        ))
        .id();

    let cell = app.world_mut().spawn_empty().id();
    app.insert_resource(SendMsg(Some(BoltHitCell { cell, bolt })));

    tick(&mut app);

    // DamageBoost component should be removed
    assert!(
        app.world().get::<DamageBoost>(bolt).is_none(),
        "DamageBoost should be removed after OnImpact(Cell) trigger"
    );

    // UntilTriggers entry should be removed
    let triggers = app.world().get::<UntilTriggers>(bolt);
    assert!(
        triggers.is_none() || triggers.unwrap().0.is_empty(),
        "UntilTriggers entry should be removed after trigger match"
    );
}

// --- Behavior 7: Until removal with OnImpact(Breaker) ---

#[test]
fn check_until_triggers_reverses_speed_boost_on_breaker_impact() {
    use crate::bolt::messages::BoltHitBreaker;

    #[derive(Resource)]
    struct SendMsg(Option<BoltHitBreaker>);

    fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitBreaker>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitBreaker>();
    app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 600.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
            }]),
        ))
        .id();

    app.insert_resource(SendMsg(Some(BoltHitBreaker { bolt })));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Reversal: 600.0 / 1.5 = 400.0
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "velocity should be reversed to 400.0, got {}",
        vel.0.y
    );
}

// --- Behavior 7 edge case: OnImpact(Cell) does NOT trigger removal of OnImpact(Breaker) ---

#[test]
fn check_until_triggers_cell_impact_does_not_remove_breaker_until() {
    use crate::bolt::messages::BoltHitCell;

    #[derive(Resource)]
    struct SendMsg(Option<BoltHitCell>);

    fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitCell>();
    app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 600.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
            }]),
        ))
        .id();

    let cell = app.world_mut().spawn_empty().id();
    app.insert_resource(SendMsg(Some(BoltHitCell { cell, bolt })));

    tick(&mut app);

    // UntilTriggers should still have the breaker entry
    let triggers = app.world().get::<UntilTriggers>(bolt).unwrap();
    assert_eq!(
        triggers.0.len(),
        1,
        "OnImpact(Cell) should NOT trigger removal of OnImpact(Breaker) entry"
    );

    // Velocity should be unchanged
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.y - 600.0).abs() < f32::EPSILON,
        "velocity should be unchanged, got {}",
        vel.0.y
    );
}

// =========================================================================
// BLOCKING: Behavior 11 — check_until_triggers reads all message types
// =========================================================================

// --- check_until_triggers removes on wall impact ---

#[test]
fn check_until_triggers_removes_on_wall_impact() {
    use crate::bolt::messages::BoltHitWall;

    #[derive(Resource)]
    struct SendMsg(Option<BoltHitWall>);

    fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitWall>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitWall>();
    app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 520.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Wall),
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.3 })],
            }]),
        ))
        .id();

    app.insert_resource(SendMsg(Some(BoltHitWall {
        bolt,
        wall: Entity::PLACEHOLDER,
    })));

    tick(&mut app);

    // UntilTriggers entry should be removed
    let triggers = app.world().get::<UntilTriggers>(bolt);
    assert!(
        triggers.is_none() || triggers.unwrap().0.is_empty(),
        "UntilTriggers entry should be removed after OnImpact(Wall) trigger"
    );

    // Velocity should be divided by multiplier: 520.0 / 1.3 = 400.0
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "velocity should be reversed to 400.0, got {}",
        vel.0.y
    );
}

// --- check_until_triggers removes on cell destroyed ---

#[test]
fn check_until_triggers_removes_on_cell_destroyed() {
    use crate::cells::messages::CellDestroyedAt;

    #[derive(Resource)]
    struct SendMsg(Option<CellDestroyedAt>);

    fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<CellDestroyedAt>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<CellDestroyedAt>();
    app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 600.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::CellDestroyed,
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
            }]),
        ))
        .id();

    app.insert_resource(SendMsg(Some(CellDestroyedAt {
        position: Vec2::ZERO,
        was_required_to_clear: true,
    })));

    tick(&mut app);

    // UntilTriggers entry should be removed
    let triggers = app.world().get::<UntilTriggers>(bolt);
    assert!(
        triggers.is_none() || triggers.unwrap().0.is_empty(),
        "UntilTriggers entry should be removed after OnCellDestroyed trigger"
    );

    // Velocity should be divided by multiplier: 600.0 / 1.5 = 400.0
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "velocity should be reversed to 400.0, got {}",
        vel.0.y
    );
}

// --- Behavior: Trigger match removes entry with nested When child ---

#[test]
fn check_until_triggers_nested_when_removed_on_trigger() {
    use crate::bolt::messages::BoltHitBreaker;

    #[derive(Resource)]
    struct SendMsg(Option<BoltHitBreaker>);

    fn enqueue(msg: Res<SendMsg>, mut writer: MessageWriter<BoltHitBreaker>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitBreaker>();
    app.add_systems(FixedUpdate, (enqueue, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Breaker),
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
                }],
            }]),
        ))
        .id();

    app.insert_resource(SendMsg(Some(BoltHitBreaker { bolt })));

    tick(&mut app);

    // UntilTriggers entry should be removed — the nested When is gone.
    let triggers = app.world().get::<UntilTriggers>(bolt);
    assert!(
        triggers.is_none() || triggers.unwrap().0.is_empty(),
        "UntilTriggers entry should be removed after OnImpact(Breaker) trigger"
    );
}

// =========================================================================
// Vec-based Until reversal — ActiveDamageBoosts, ActivePiercings
// =========================================================================

// --- Test 12: Trigger removes damage boost entry from vec ---

#[test]
fn check_until_triggers_removes_damage_boost_from_vec_on_trigger() {
    use crate::bolt::messages::BoltHitCell;

    #[derive(Resource)]
    struct SendCellMsg(Option<BoltHitCell>);

    fn enqueue_cell_msg(msg: Res<SendCellMsg>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitCell>();
    app.add_systems(
        FixedUpdate,
        (enqueue_cell_msg, check_until_triggers).chain(),
    );

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            ActiveDamageBoosts(vec![2.0, 1.5]),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                children: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
            }]),
        ))
        .id();

    let cell = app.world_mut().spawn_empty().id();
    app.insert_resource(SendCellMsg(Some(BoltHitCell { cell, bolt })));

    tick(&mut app);

    // The 2.0 entry should be removed, leaving [1.5]
    let boosts = app
        .world()
        .get::<ActiveDamageBoosts>(bolt)
        .expect("bolt should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![1.5],
        "ActiveDamageBoosts should be [1.5] after 2.0 is removed, got {:?}",
        boosts.0
    );
}

// --- M11: Trigger-based Until reversal for Piercing via check_until_triggers ---

/// M11: Bolt with `ActivePiercings`(vec![2, 1]) and `UntilTriggers` entry
/// { trigger: Impact(Cell), children: [Do(Piercing(2))] }. On `BoltHitCell`,
/// `check_until_triggers` removes the matching entry (value 2) from `ActivePiercings`.
#[test]
fn check_until_triggers_removes_piercing_from_active_piercings() {
    use crate::bolt::messages::BoltHitCell;

    #[derive(Resource)]
    struct SendCellMsg(Option<BoltHitCell>);

    fn enqueue_cell(msg: Res<SendCellMsg>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    let mut app = test_app();
    app.add_message::<BoltHitCell>();
    app.add_systems(FixedUpdate, (enqueue_cell, check_until_triggers).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            ActivePiercings(vec![2, 1]),
            UntilTriggers(vec![UntilTriggerEntry {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                children: vec![EffectNode::Do(Effect::Piercing(2))],
            }]),
        ))
        .id();

    let cell = app.world_mut().spawn_empty().id();
    app.insert_resource(SendCellMsg(Some(BoltHitCell { cell, bolt })));

    tick(&mut app);

    // The entry matching value 2 should be removed, leaving [1]
    let piercings = app
        .world()
        .get::<ActivePiercings>(bolt)
        .expect("bolt should have ActivePiercings");
    assert_eq!(
        piercings.0,
        vec![1],
        "ActivePiercings should be [1] after removing matching entry 2, got {:?}",
        piercings.0
    );

    // UntilTriggers entry should be consumed
    let triggers = app.world().get::<UntilTriggers>(bolt);
    assert!(
        triggers.is_none() || triggers.unwrap().0.is_empty(),
        "UntilTriggers entry should be removed after trigger match"
    );
}
