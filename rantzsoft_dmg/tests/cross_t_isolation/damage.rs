//! Behavior 11: `DamageDealt<T1>` does not affect a `T2` target.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageDealt, Destroyed, Hp};

use super::common::*;

#[test]
fn damage_dealt_t1_does_not_affect_t2_target() {
    let mut app = app_with_both_types();

    let victim_a = app.world_mut().spawn((T1, Hp::new(10.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(10.0))).id();

    // T1 message targeting a T2 entity — apply_damage::<T1> query filters
    // With<T1>, so victim_b is skipped.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:        None,
            attributed_to: None,
            target:        victim_b,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 10.0);
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 10.0);

    let t1_destroyed = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T1>>>()
        .drain()
        .count();
    let t2_destroyed = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T2>>>()
        .drain()
        .count();
    assert_eq!(t1_destroyed, 0);
    assert_eq!(t2_destroyed, 0);
}

#[test]
fn damage_dealt_t2_affects_t2_target_and_leaves_t1_alone() {
    // Edge case 11a.
    let mut app = app_with_both_types();

    let victim_a = app.world_mut().spawn((T1, Hp::new(10.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(10.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:        None,
            attributed_to: None,
            target:        victim_b,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 5.0);
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 10.0);
}
