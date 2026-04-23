//! Behavior 14: `DamageBoostStack` on a dealer is indiscriminate of
//! pipeline `T`.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageBoostStack, DamageDealt, Hp, SourceId};

use super::common::*;

#[test]
fn persistent_boost_on_single_dealer_multiplies_both_t_queues() {
    let mut app = app_with_both_types();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:boost"), 2.0);
        app.world_mut().spawn(s).id()
    };
    let victim_a = app.world_mut().spawn((T1, Hp::new(100.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(100.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  Some(dealer),
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    // 100 - 5 * 2 = 90, for both victims independently.
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 90.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 90.0);
}

#[test]
fn one_shot_on_single_dealer_drained_by_whichever_t_system_runs_first() {
    // Edge case 14a: DOCUMENTED IRREGULARITY — a single one-shot is
    // consumed by whichever of `apply_damage_boosts::<T1>` or
    // `apply_damage_boosts::<T2>` runs first this tick. Rather than assume
    // the order, assert the order-independent aggregate HP drop across
    // both victims. One victim takes 5 base, the other takes 5 * 2 = 10.
    let mut app = app_with_both_types();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot(2.0);
        app.world_mut().spawn(s).id()
    };
    let victim_a = app.world_mut().spawn((T1, Hp::new(100.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(100.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  Some(dealer),
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(victim_a).unwrap().current;
    let hp_b = app.world().get::<Hp>(victim_b).unwrap().current;
    // Total remaining HP is 200 - (5 + 10) = 185, regardless of order.
    assert_f32_eq(hp_a + hp_b, 185.0);
}
