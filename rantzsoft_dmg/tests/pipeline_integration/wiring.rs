//! Group D — Plugin/API wiring sanity, behaviors 16–19.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageDealt, DespawnEntity, Destroyed, Dmgable, HealDealt, Hp, KillYourself, RantzDmgAppExt,
    RantzDmgPlugin,
};
use rantzsoft_spatial2d::components::Position2D;

#[derive(Component)]
struct TestT;
impl Dmgable for TestT {}

#[derive(Component)]
struct TestU;
impl Dmgable for TestU {}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// ── Behavior 16: plugin-first ordering yields a functional pipeline. ──

#[test]
fn plugin_first_then_register_dmgable_produces_working_pipeline() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Position2D(Vec2::ZERO)))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        10.0,
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
}

#[test]
fn plugin_first_then_register_two_types_fires_both_pipelines() {
    // Edge case 16a.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<TestT>().register_dmgable::<TestU>();

    let victim_t = app
        .world_mut()
        .spawn((TestT, Hp::new(5.0), Position2D(Vec2::ZERO)))
        .id();
    let victim_u = app
        .world_mut()
        .spawn((TestU, Hp::new(5.0), Position2D(Vec2::new(1.0, 1.0))))
        .id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        None,
            attributed_to: None,
            target:        victim_t,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestU>>>()
        .write(DamageDealt::<TestU> {
            dealer:        None,
            attributed_to: None,
            target:        victim_u,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let t_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    let u_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestU>>>()
        .drain()
        .count();
    assert_eq!(t_count, 1);
    assert_eq!(u_count, 1);
    assert!(app.world().get_entity(victim_t).is_err());
    assert!(app.world().get_entity(victim_u).is_err());
}

// ── Behavior 17: register_dmgable returns `&mut App` and chains. ──

#[test]
fn register_dmgable_chains_with_add_message_and_other_registers() {
    #[derive(Message)]
    struct Unrelated;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app
        .register_dmgable::<TestT>()
        .register_dmgable::<TestU>()
        .add_message::<Unrelated>();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestU>>>()
    );
    assert!(app.world().contains_resource::<Messages<Unrelated>>());
}

#[test]
fn single_register_dmgable_call_registers_all_four_per_t_resources() {
    // Edge case 17a.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<TestT>();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<HealDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<KillYourself<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<Destroyed<TestT>>>()
    );
}

// ── Behavior 18: DespawnEntity from outside the pipeline is processed. ──

#[test]
fn external_despawn_entity_message_despawns_plain_entity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let plain_entity = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DespawnEntity>>()
        .write(DespawnEntity {
            entity: plain_entity,
        });
    tick(&mut app);

    assert!(app.world().get_entity(plain_entity).is_err());
}

#[test]
fn external_despawn_entity_messages_despawn_multiple_plain_entities() {
    // Edge case 18a.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();

    for entity in [e1, e2] {
        app.world_mut()
            .resource_mut::<Messages<DespawnEntity>>()
            .write(DespawnEntity { entity });
    }
    tick(&mut app);

    assert!(app.world().get_entity(e1).is_err());
    assert!(app.world().get_entity(e2).is_err());
}

#[test]
fn external_despawn_entity_for_nonexistent_entity_does_not_panic() {
    // Edge case 18b.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    app.world_mut()
        .resource_mut::<Messages<DespawnEntity>>()
        .write(DespawnEntity {
            entity: Entity::PLACEHOLDER,
        });
    tick(&mut app);

    // No assertion needed beyond "tick did not panic"; placeholder is
    // silently absent.
    assert!(app.world().get_entity(Entity::PLACEHOLDER).is_err());
}

// ── Behavior 19: no per-T messages, no per-T systems before
//    register_dmgable (negative contract). ──

#[test]
fn without_register_dmgable_no_per_t_resources_exist_and_zero_hp_not_killed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(0.0), Position2D(Vec2::ZERO)))
        .id();

    tick(&mut app);

    assert!(
        !app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<HealDealt<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<KillYourself<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<Destroyed<TestT>>>()
    );

    // No detect_deaths::<TestT> is registered — the zero-HP victim is not
    // killed and remains in the world with HP unchanged.
    assert!(app.world().get_entity(victim).is_ok());
    assert!(
        (app.world().get::<Hp>(victim).unwrap().current - 0.0).abs() < f32::EPSILON,
        "zero-HP victim should remain at 0.0 HP when detect_deaths is not registered"
    );
}

#[test]
fn late_register_dmgable_after_first_update_wires_systems_on_next_tick() {
    // Edge case 19a: late-registration contract.
    //
    // `register_dmgable::<T>()` called after the first `app.update()` MUST
    // still produce a working pipeline for `T` on the next tick — Bevy
    // 0.18's schedule rebuilds on `add_systems` post-update.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(0.0), Position2D(Vec2::ZERO)))
        .id();

    // First tick: no per-T systems registered; victim unchanged.
    tick(&mut app);
    assert!(app.world().get_entity(victim).is_ok());

    // Register late, then tick again.
    let _ = app.register_dmgable::<TestT>();
    tick(&mut app);

    // Now detect_deaths::<TestT> has fired and emitted Destroyed; the
    // victim has been despawned same-tick by process_despawn_requests.
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 1);
    assert!(app.world().get_entity(victim).is_err());
}
