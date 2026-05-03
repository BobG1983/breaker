//! Phantom-breaker gating tests for the `*_bump_occurred` bridges
//! (`on_bump_occurred`, `on_perfect_bump_occurred`, `on_early_bump_occurred`,
//! `on_late_bump_occurred`, `on_no_bump_occurred`).
//!
//! Each bridge must skip its chip walk when `msg.breaker` is a phantom entity.
//! Today none of the bridges have this gate, so all phantom-gate assertions
//! fail (RED). Regression-guard assertions (real breaker) pass today.

use bevy::prelude::*;

use super::super::{
    super::system::{
        on_bump_occurred, on_early_bump_occurred, on_late_bump_occurred, on_perfect_bump_occurred,
    },
    helpers::*,
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed, NoBump},
    effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, storage::BoundEffects, types::Trigger,
    },
};

// ── Behavior #5: on_bump_occurred skips phantom breakers ─────────────────────

#[test]
fn on_bump_occurred_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::BumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "on_bump_occurred must NOT walk global chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_bump_occurred_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let real_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::BumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
        .expect("on_bump_occurred should walk global chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: Late grade on phantom — gate still applies (`on_bump_occurred` fires for any grade).
#[test]
fn on_bump_occurred_skips_phantom_breaker_msg_any_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_bump_occurred),
        on_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::BumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "on_bump_occurred phantom gate must apply for any grade"
    );
}

// ── Behavior #6: on_perfect_bump_occurred skips phantom breakers ─────────────

#[test]
fn on_perfect_bump_occurred_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "on_perfect_bump_occurred must NOT walk global chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_perfect_bump_occurred_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let real_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
        .expect("on_perfect_bump_occurred should walk global chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: non-Perfect grade on `on_perfect_bump_occurred` — `None` due to
/// grade filter, not the phantom gate.
#[test]
fn on_perfect_bump_occurred_skips_non_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bump_occurred),
        on_perfect_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::PerfectBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "EffectStack must be None — grade filter rejects non-Perfect grade before phantom gate"
    );
}

// ── Behavior #7: on_early_bump_occurred skips phantom breakers ───────────────

#[test]
fn on_early_bump_occurred_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bump_occurred),
        on_early_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::EarlyBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "on_early_bump_occurred must NOT walk global chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_early_bump_occurred_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bump_occurred),
        on_early_bump_occurred,
    ));

    let real_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::EarlyBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
        .expect("on_early_bump_occurred should walk global chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: non-Early grade on `on_early_bump_occurred` — `None` due to
/// grade filter, not the phantom gate.
#[test]
fn on_early_bump_occurred_skips_non_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bump_occurred),
        on_early_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::EarlyBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "EffectStack must be None — grade filter rejects non-Early grade before phantom gate"
    );
}

// ── Behavior #8: on_late_bump_occurred skips phantom breakers ────────────────

#[test]
fn on_late_bump_occurred_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_late_bump_occurred),
        on_late_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::LateBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "on_late_bump_occurred must NOT walk global chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_late_bump_occurred_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_late_bump_occurred),
        on_late_bump_occurred,
    ));

    let real_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::LateBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
        .expect("on_late_bump_occurred should walk global chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: non-Late grade on `on_late_bump_occurred` — `None` due to
/// grade filter, not the phantom gate.
#[test]
fn on_late_bump_occurred_skips_non_late_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_late_bump_occurred),
        on_late_bump_occurred,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::LateBumpOccurred,
            1.5,
        )]))
        .id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "EffectStack must be None — grade filter rejects non-Late grade before phantom gate"
    );
}

// ── Behavior #9: on_no_bump_occurred skips phantom breakers ──────────────────

#[test]
fn on_no_bump_occurred_skips_phantom_breaker_msg() {
    let mut app = bridge_test_app();

    let phantom_entity = spawn_phantom_marker(&mut app);
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip", 1.5)]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
            .is_none(),
        "on_no_bump_occurred must NOT walk global chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_no_bump_occurred_walks_real_breaker_msg() {
    let mut app = bridge_test_app();

    let real_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip", 1.5)]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![NoBump {
        bolt:    bolt_entity,
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
        .expect("on_no_bump_occurred should walk global chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: mixed messages — one real, one phantom — only the real one fires.
#[test]
fn on_no_bump_occurred_mixed_messages_only_real_fires() {
    let mut app = bridge_test_app();

    let phantom_entity = spawn_phantom_marker(&mut app);
    let real_entity = app.world_mut().spawn_empty().id();
    let bolt_a = app.world_mut().spawn_empty().id();
    let bolt_b = app.world_mut().spawn_empty().id();
    let chip_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![no_bump_speed_tree("chip", 1.5)]))
        .id();

    app.insert_resource(TestNoBumpMessages(vec![
        NoBump {
            bolt:    bolt_a,
            breaker: real_entity,
        },
        NoBump {
            bolt:    bolt_b,
            breaker: phantom_entity,
        },
    ]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(chip_entity)
        .expect("EffectStack should exist — the real-breaker NoBump fired");
    assert_eq!(
        stack.len(),
        1,
        "only the real-breaker message should walk; phantom skipped (got {})",
        stack.len()
    );
}
