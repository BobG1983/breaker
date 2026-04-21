//! Group F: Target query edge cases (silent skips) — Behaviors 17-19.

use bevy::prelude::*;

use super::super::helpers::{
    PendingHeal, TestEntity, build_apply_heal_app, heal_msg, spawn_test_entity,
};
use crate::{prelude::*, shared::death_pipeline::heal_dealt::HealCap};

/// Behavior 17: entity without `TestEntity` marker is skipped.
#[test]
fn heal_targeting_entity_without_marker_is_skipped() {
    let mut app = build_apply_heal_app();
    let entity = app
        .world_mut()
        .spawn((
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Entity without TestEntity should remain at 5.0, got {}",
        hp.current
    );
}

/// Behavior 18: entity without Hp is silently skipped.
#[test]
fn heal_targeting_entity_without_hp_does_not_panic() {
    let mut app = build_apply_heal_app();
    let entity = app.world_mut().spawn(TestEntity).id();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app);

    assert!(
        app.world().get::<Hp>(entity).is_none(),
        "Entity without Hp should still have no Hp"
    );
}

/// Behavior 19: heal targeting despawned entity is silently skipped.
#[test]
fn heal_targeting_despawned_entity_does_not_panic() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity(&mut app, 5.0);
    app.world_mut().entity_mut(entity).despawn();

    app.insert_resource(PendingHeal(vec![heal_msg(entity, 3.0, HealCap::Max)]));
    tick(&mut app); // must not panic

    assert!(
        app.world().get_entity(entity).is_err(),
        "entity should remain despawned"
    );
}
