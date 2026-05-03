//! Phantom-breaker gating tests for `BumpPerformed`-driven bridges
//! (`on_bumped`, `on_perfect_bumped`, `on_early_bumped`, `on_late_bumped`).
//!
//! Each bridge must skip its chip walk when `msg.breaker` is a phantom entity.
//! Today none of the bridges have this gate, so all phantom-gate assertions
//! fail (RED). Regression-guard assertions (real breaker) pass today.

use bevy::prelude::*;

use super::super::{
    super::system::{on_bumped, on_early_bumped, on_late_bumped, on_perfect_bumped},
    helpers::*,
};
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, storage::BoundEffects, types::Trigger,
    },
};

// ── Behavior #1: on_bumped skips phantom breakers ────────────────────────────

#[test]
fn on_bumped_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree("chip", Trigger::Bumped, 1.5)]));

    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "on_bumped must NOT walk chip effects for a phantom breaker; today it does (RED)"
    );
}

#[test]
fn on_bumped_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let real_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree("chip", Trigger::Bumped, 1.5)]))
        .id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(real_entity)
        .expect("on_bumped should walk chip effects for a real breaker");
    assert_eq!(
        stack.len(),
        1,
        "exactly one effect should fire for real breaker"
    );
}

/// Edge case: Early grade on phantom — gate still applies.
#[test]
fn on_bumped_skips_phantom_breaker_msg_early_grade() {
    let mut app = bump_performed_test_app((inject_bump_performed.before(on_bumped), on_bumped));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree("chip", Trigger::Bumped, 1.5)]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "on_bumped phantom gate must apply regardless of grade"
    );
}

// ── Behavior #2: on_perfect_bumped skips phantom breakers ────────────────────

#[test]
fn on_perfect_bumped_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::PerfectBumped,
            1.5,
        )]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "on_perfect_bumped must NOT walk chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_perfect_bumped_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let real_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::PerfectBumped,
            1.5,
        )]))
        .id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(real_entity)
        .expect("on_perfect_bumped should walk chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: non-Perfect grade on `on_perfect_bumped` — `None` due to grade
/// filter, not the phantom gate.
#[test]
fn on_perfect_bumped_skips_non_perfect_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_perfect_bumped),
        on_perfect_bumped,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::PerfectBumped,
            1.5,
        )]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "EffectStack must be None — grade filter rejects non-Perfect grade before phantom gate"
    );
}

// ── Behavior #3: on_early_bumped skips phantom breakers ──────────────────────

#[test]
fn on_early_bumped_skips_phantom_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bumped),
        on_early_bumped,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::EarlyBumped,
            1.5,
        )]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "on_early_bumped must NOT walk chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_early_bumped_walks_real_breaker_msg() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bumped),
        on_early_bumped,
    ));

    let real_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::EarlyBumped,
            1.5,
        )]))
        .id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Early,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(real_entity)
        .expect("on_early_bumped should walk chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: non-Early grade on `on_early_bumped` — `None` due to grade
/// filter, not the phantom gate.
#[test]
fn on_early_bumped_skips_non_early_grade() {
    let mut app = bump_performed_test_app((
        inject_bump_performed.before(on_early_bumped),
        on_early_bumped,
    ));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::EarlyBumped,
            1.5,
        )]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "EffectStack must be None — grade filter rejects non-Early grade before phantom gate"
    );
}

// ── Behavior #4: on_late_bumped skips phantom breakers ───────────────────────

#[test]
fn on_late_bumped_skips_phantom_breaker_msg() {
    let mut app =
        bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::LateBumped,
            1.5,
        )]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "on_late_bumped must NOT walk chip effects for a phantom breaker (RED)"
    );
}

#[test]
fn on_late_bumped_walks_real_breaker_msg() {
    let mut app =
        bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

    let real_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::LateBumped,
            1.5,
        )]))
        .id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Late,
        bolt:    Some(bolt_entity),
        breaker: real_entity,
    }]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(real_entity)
        .expect("on_late_bumped should walk chip effects for a real breaker");
    assert_eq!(stack.len(), 1);
}

/// Edge case: non-Late grade on `on_late_bumped` — `None` due to grade
/// filter, not the phantom gate.
#[test]
fn on_late_bumped_skips_non_late_grade() {
    let mut app =
        bump_performed_test_app((inject_bump_performed.before(on_late_bumped), on_late_bumped));

    let phantom_entity = spawn_phantom_marker(&mut app);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BoundEffects(vec![speed_tree(
            "chip",
            Trigger::LateBumped,
            1.5,
        )]));
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestBumpPerformedMessages(vec![BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    Some(bolt_entity),
        breaker: phantom_entity,
    }]));

    tick(&mut app);

    assert!(
        app.world()
            .get::<EffectStack<SpeedBoostConfig>>(phantom_entity)
            .is_none(),
        "EffectStack must be None — grade filter rejects non-Late grade before phantom gate"
    );
}
