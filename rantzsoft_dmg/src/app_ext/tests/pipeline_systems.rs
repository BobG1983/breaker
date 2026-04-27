use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    super::system::*,
    helpers::{TestT, app_with_plugin, assert_f32_eq, tick},
};
use crate::{
    HealCap, Hp, KilledBy, VulnerableStack,
    components::{DamageBoostStack, Invulnerable},
    messages::{DamageDealt, Destroyed, HealDealt},
};

// ── Behavior 144: apply_damage_boosts::<TestT> attached to ApplyDamageBoosts ──

#[test]
fn apply_damage_boosts_runs_in_pipeline() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(2.0);
            s
        })
        .id();
    let target = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer: Some(dealer),
            attributed_to: None,
            target,
            amount: 5.0,
            source: None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 0.0);
    let killed_by = app
        .world()
        .get::<KilledBy>(target)
        .expect("KilledBy expected on killing blow");
    assert_eq!(killed_by.killer, Some(dealer));
}

#[test]
fn invulnerable_filter_zeroes_damage_within_apply_damage_set() {
    // Edge case 144a.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = app
        .world_mut()
        .spawn({
            let mut s = DamageBoostStack::default();
            s.add_one_shot(100.0);
            s
        })
        .id();
    let target = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Invulnerable))
        .id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer: Some(dealer),
            attributed_to: None,
            target,
            amount: 5.0,
            source: None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 10.0);
}

// ── Behavior 145: apply_vulnerable::<TestT> attached to ApplyVulnerable ──

#[test]
fn apply_vulnerable_runs_in_pipeline() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let target = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), {
            let mut s = VulnerableStack::default();
            s.add_one_shot(2.0);
            s
        }))
        .id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer: None,
            attributed_to: None,
            target,
            amount: 5.0,
            source: None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 0.0);
    assert!(app.world().get::<KilledBy>(target).is_some());
}

// ── Behavior 146: detect_deaths::<TestT> attached to EmitKill ──

#[test]
fn detect_deaths_runs_in_pipeline() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(0.0), Position2D(Vec2::new(1.0, 2.0))))
        .id();

    tick(&mut app);

    // A nonempty Destroyed queue proves the full pipeline executed:
    // detect_deaths emitted KillYourself → handle_kill consumed it
    // and wrote Destroyed. (The victim itself is despawned in the
    // same tick by process_despawn_requests in FixedPostUpdate, so
    // the entity is no longer present to query for Dead.)
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert!(destroyed_count > 0);
    assert!(app.world().get_entity(victim).is_err());
}

// ── Behavior 147: handle_kill::<TestT> attached to ApplyKill ──

#[test]
fn handle_kill_runs_in_pipeline() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0)), Hp::new(0.0)))
        .id();

    tick(&mut app);

    // The entity is despawned in FixedPostUpdate (same tick), so we
    // prove handle_kill ran by checking the Destroyed message it
    // emitted plus the post-despawn absence of the victim entity.
    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_f32_eq(destroyed[0].victim_pos.x, 10.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 20.0);
    assert!(app.world().get_entity(victim).is_err());
}

// ── Behavior 148: apply_heal::<TestT> attached to ApplyHeal ──

#[test]
fn apply_heal_runs_in_pipeline() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let target = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer: None,
            attributed_to: None,
            target,
            amount: 3.0,
            source: None,
            cap: HealCap::Max,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 8.0);
}
