use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    super::system::*,
    helpers::{TestT, app_with_plugin, assert_f32_eq, tick},
};
use crate::{
    Hp,
    messages::{DamageDealt, Destroyed},
};

// ── Behavior 150: end-to-end pipeline ──

#[test]
fn end_to_end_kill_and_despawn() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(5.0), Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    // Destroyed message drained and describes the victim.
    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, victim);
    assert_eq!(destroyed[0].killer, Some(dealer));
    assert_f32_eq(destroyed[0].victim_pos.x, 1.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 2.0);
    assert!(destroyed[0].killer_pos.is_none());

    // Entity actually despawned by process_despawn_requests in
    // FixedPostUpdate (same update() call).
    assert!(app.world().get_entity(victim).is_err());
}

#[test]
fn end_to_end_kill_with_positioned_dealer_records_killer_pos() {
    // Edge case 150a.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(5.0), Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let dealer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    let kp = destroyed[0].killer_pos.unwrap();
    assert_f32_eq(kp.x, 50.0);
    assert_f32_eq(kp.y, 60.0);
}
