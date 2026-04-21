//! Group H: Heal attribution (healer / source) — Behaviors 24-25.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::helpers::{PendingHeal, TestEntity, build_apply_heal_app, spawn_test_entity};
use crate::{
    prelude::*,
    shared::death_pipeline::heal_dealt::{HealCap, HealDealt},
};

/// Behavior 24: heal does NOT write to `KilledBy`.
#[test]
fn heal_does_not_write_to_killed_by() {
    let mut app = build_apply_heal_app();
    let entity = spawn_test_entity(&mut app, 10.0);
    app.world_mut().get_mut::<Hp>(entity).unwrap().current = 5.0;
    let some_entity = app.world_mut().spawn_empty().id();

    let msg = HealDealt::<TestEntity> {
        healer:  Some(some_entity),
        target:  entity,
        amount:  3.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    };
    app.insert_resource(PendingHeal(vec![msg]));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(entity).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "heal must never touch KilledBy.dealer"
    );
}

/// Behavior 25: heal payload fields are read-only — no leakage into unrelated components.
#[test]
fn heal_payload_fields_do_not_leak_into_other_components() {
    let mut app = build_apply_heal_app();
    let healer = app.world_mut().spawn_empty().id();
    let victim = app
        .world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default()))
        .id();
    // Pre-damage victim to 4.0.
    app.world_mut().get_mut::<Hp>(victim).unwrap().current = 4.0;

    let msg = HealDealt::<TestEntity> {
        healer:  Some(healer),
        target:  victim,
        amount:  2.0,
        source:  Some("cascade".to_string()),
        cap:     HealCap::Max,
        _marker: PhantomData,
    };
    app.insert_resource(PendingHeal(vec![msg]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(victim).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "Hp should be 6.0 after 2.0 heal, got {}",
        hp.current
    );
    assert!(
        app.world().get::<Dead>(victim).is_none(),
        "Dead should remain absent after a heal"
    );
    assert!(
        app.world().get::<Invulnerable>(victim).is_none(),
        "Invulnerable should remain absent after a heal"
    );
    let killed_by = app.world().get::<KilledBy>(victim).unwrap();
    assert_eq!(
        killed_by.dealer, None,
        "KilledBy.dealer should remain None after a heal"
    );
}
