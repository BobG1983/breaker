//! Behavior 13: dual-T kill in one tick — each victim dies in its own
//! `Destroyed<T>` queue, killers attributed independently.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageDealt, Destroyed, Hp};
use rantzsoft_spatial2d::components::Position2D;

use super::common::*;

#[test]
fn dual_t_kill_emits_one_destroyed_per_type_with_correct_killer() {
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((T1, Hp::new(5.0), Position2D(Vec2::new(1.0, 1.0))))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((T2, Hp::new(5.0), Position2D(Vec2::new(9.0, 9.0))))
        .id();
    let dealer_1 = app.world_mut().spawn_empty().id();
    let dealer_2 = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer_1),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  Some(dealer_2),
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let t1_destroyed: Vec<Destroyed<T1>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T1>>>()
        .drain()
        .collect();
    let t2_destroyed: Vec<Destroyed<T2>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T2>>>()
        .drain()
        .collect();
    assert_eq!(t1_destroyed.len(), 1);
    assert_eq!(t1_destroyed[0].victim, victim_a);
    assert_f32_eq(t1_destroyed[0].victim_pos.x, 1.0);
    assert_f32_eq(t1_destroyed[0].victim_pos.y, 1.0);
    assert_eq!(t1_destroyed[0].killer, Some(dealer_1));

    assert_eq!(t2_destroyed.len(), 1);
    assert_eq!(t2_destroyed[0].victim, victim_b);
    assert_f32_eq(t2_destroyed[0].victim_pos.x, 9.0);
    assert_f32_eq(t2_destroyed[0].victim_pos.y, 9.0);
    assert_eq!(t2_destroyed[0].killer, Some(dealer_2));

    assert!(app.world().get_entity(victim_a).is_err());
    assert!(app.world().get_entity(victim_b).is_err());
}

#[test]
fn single_t1_kill_leaves_t2_victim_alive_and_t2_destroyed_queue_empty() {
    // Edge case 13a.
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((T1, Hp::new(5.0), Position2D(Vec2::new(1.0, 1.0))))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((T2, Hp::new(5.0), Position2D(Vec2::new(9.0, 9.0))))
        .id();
    let dealer_1 = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer_1),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let t1_destroyed: Vec<Destroyed<T1>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T1>>>()
        .drain()
        .collect();
    let t2_destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T2>>>()
        .drain()
        .count();
    assert_eq!(t1_destroyed.len(), 1);
    assert_eq!(t1_destroyed[0].killer, Some(dealer_1));
    assert_eq!(t2_destroyed_count, 0);
    assert!(app.world().get_entity(victim_b).is_ok());
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 5.0);
}
