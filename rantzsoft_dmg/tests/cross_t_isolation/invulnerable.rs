//! Behavior 15: `Invulnerable` on a T1 victim does NOT shield a T2 victim.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageDealt, Hp, Invulnerable};

use super::common::*;

#[test]
fn invulnerable_on_t1_does_not_shield_t2_victim() {
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((T1, Hp::new(10.0), Invulnerable))
        .id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(10.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:        None,
            attributed_to: None,
            target:        victim_a,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
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

    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 10.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 5.0);
}

#[test]
fn invulnerable_on_t2_does_not_shield_t1_victim() {
    // Edge case 15a: swap the invulnerable marker.
    let mut app = app_with_both_types();

    let victim_a = app.world_mut().spawn((T1, Hp::new(10.0))).id();
    let victim_b = app
        .world_mut()
        .spawn((T2, Hp::new(10.0), Invulnerable))
        .id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:        None,
            attributed_to: None,
            target:        victim_a,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
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

    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 5.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 10.0);
}
