//! Behavior 12: `HealDealt<T1>` does not affect a `T2` target.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{HealCap, HealDealt, Hp};

use super::common::*;

#[test]
fn heal_dealt_t1_does_not_affect_t2_target() {
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((
            T1,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((
            T2,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<T1>>>()
        .write(HealDealt::<T1> {
            healer:        None,
            attributed_to: None,
            target:        victim_b,
            amount:        5.0,
            source:        None,
            cap:           HealCap::Starting,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 3.0);
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 3.0);
}

#[test]
fn heal_dealt_t2_affects_t2_target() {
    // Edge case 12a.
    let mut app = app_with_both_types();

    let victim_b = app
        .world_mut()
        .spawn((
            T2,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<T2>>>()
        .write(HealDealt::<T2> {
            healer:        None,
            attributed_to: None,
            target:        victim_b,
            amount:        5.0,
            source:        None,
            cap:           HealCap::Starting,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 8.0);
}
