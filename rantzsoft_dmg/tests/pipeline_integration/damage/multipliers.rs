//! Behavior 2 — `DamageBoostStack` persistent + `VulnerableStack` persistent
//! multiply damage end-to-end across the full pipeline.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageBoostStack, DamageDealt, Destroyed, Hp, RantzDmgAppExt, SourceId, VulnerableStack,
};
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{TestT, app_with_plugin, assert_f32_eq, tick};

// ── Behavior 2: DamageBoostStack persistent + VulnerableStack persistent
//    multiply damage across the pipeline. ──

#[test]
fn persistent_damage_boost_and_vulnerable_multiply_end_to_end() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:boost"), 2.0);
        app.world_mut().spawn(s).id()
    };
    let victim = {
        let mut v = VulnerableStack::default();
        v.add(SourceId::from("src:vuln"), 2.5);
        app.world_mut()
            .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(0.0, 0.0)), v))
            .id()
    };

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        1.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    // 1.0 * 2.0 (boost) * 2.5 (vulnerable) = 5.0 damage; HP 10 - 5 = 5.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 5.0);
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
    assert!(app.world().get_entity(victim).is_ok());
}

#[test]
fn persistent_boost_and_vulnerable_kill_when_amount_pushes_to_lethal() {
    // Edge case 2a: 2.0 base * 2.0 * 2.5 = 10.0 lethal damage.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:boost"), 2.0);
        app.world_mut().spawn(s).id()
    };
    let victim = {
        let mut v = VulnerableStack::default();
        v.add(SourceId::from("src:vuln"), 2.5);
        app.world_mut()
            .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(0.0, 0.0)), v))
            .id()
    };

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        2.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 1);
    assert!(app.world().get_entity(victim).is_err());
    // NOTE: do not read Hp after despawn — the entity no longer exists.
}
