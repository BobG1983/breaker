//! Tests for `evaluate_bound_effects`.

use bevy::prelude::*;

use super::*;

#[test]
fn bound_entry_not_consumed_after_match() {
    // BoundEffects entries are NEVER consumed, even when the trigger matches.
    // We use a non-Do child to avoid queuing fire_effect commands.
    let mut app = test_app();
    app.add_systems(Update, (sys_evaluate_bound_for_bump, sys_snapshot).chain());

    let inner = when_do(Trigger::Death, EffectKind::DamageBoost(2.0));
    let bound_node = when_child(Trigger::Bump, inner);
    app.world_mut().spawn((
        BoundEffects(vec![("chip_a".into(), bound_node)]),
        StagedEffects::default(),
    ));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(snap.bound_len, 1, "BoundEffects entry must not be consumed");
}

#[test]
fn non_matching_trigger_leaves_bound_and_staged_unchanged() {
    // When the trigger does not match, nothing changes.
    let mut app = test_app();
    app.add_systems(
        Update,
        (sys_evaluate_bound_for_bump_whiff, sys_snapshot).chain(),
    );

    let node = when_do(Trigger::Bump, EffectKind::DamageBoost(2.0));
    app.world_mut().spawn((
        BoundEffects(vec![("chip_a".into(), node)]),
        StagedEffects::default(),
    ));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(snap.bound_len, 1, "BoundEffects unchanged on non-match");
    assert_eq!(snap.staged_len, 0, "StagedEffects empty on non-match");
}

#[test]
fn non_do_children_pushed_to_staged_effects() {
    // When(Bump, [When(Death, Do(X))]) -- the inner When is non-Do,
    // so it gets pushed to StagedEffects instead of being fired.
    let mut app = test_app();
    app.add_systems(Update, (sys_evaluate_bound_for_bump, sys_snapshot).chain());

    let inner_when = when_do(Trigger::Death, EffectKind::DamageBoost(2.0));
    let outer = when_child(Trigger::Bump, inner_when.clone());

    app.world_mut().spawn((
        BoundEffects(vec![("chip_a".into(), outer)]),
        StagedEffects::default(),
    ));

    app.update();

    let snap = app.world().resource::<Snapshot>();
    assert_eq!(
        snap.staged_len, 1,
        "Non-Do child should be pushed to StagedEffects"
    );
    assert_eq!(
        snap.staged_entries[0].1, inner_when,
        "Pushed node should be the inner When(Death, Do(DamageBoost))"
    );
}
