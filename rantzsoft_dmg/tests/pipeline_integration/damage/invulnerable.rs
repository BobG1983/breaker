//! Behavior 5 — `Invulnerable` target short-circuits damage end-to-end.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageBoostStack, DamageDealt, Destroyed, Hp, Invulnerable, RantzDmgAppExt};
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{TestT, app_with_plugin, assert_f32_eq, tick};

// ── Behavior 5: Invulnerable target short-circuits damage end-to-end. ──

#[test]
fn invulnerable_target_short_circuits_damage_end_to_end() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot(100.0);
        app.world_mut().spawn(s).id()
    };
    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp::new(10.0),
            Invulnerable,
            Position2D(Vec2::new(0.0, 0.0)),
        ))
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

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 10.0);
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
    assert!(app.world().get_entity(victim).is_ok());
}
