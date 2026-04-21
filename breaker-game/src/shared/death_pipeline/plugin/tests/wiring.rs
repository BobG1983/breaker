use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{super::system::*, helpers::*};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
    shared::{
        death_pipeline::{
            heal_dealt::{HealCap, HealDealt},
            hp::Hp,
            killed_by::KilledBy,
            sets::DeathPipelineSystems,
        },
        test_utils::tick,
    },
    walls::components::Wall,
};

// ── Group K / Behavior 33: apply_heal::<T> wired for all 5 types ──

#[test]
fn plugin_registers_apply_heal_cell() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    app.init_resource::<PendingCellHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingCellHeal(vec![HealDealt::<Cell> {
        healer:  None,
        target:  cell,
        amount:  2.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Cell Hp should be 7.0 after 2.0 heal, got {}",
        hp.current
    );
}

#[test]
fn plugin_registers_apply_heal_bolt() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    app.init_resource::<PendingBoltHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_bolt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingBoltHeal(vec![HealDealt::<Bolt> {
        healer:  None,
        target:  bolt,
        amount:  2.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(bolt).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Bolt Hp should be 7.0 after 2.0 heal, got {}",
        hp.current
    );
}

#[test]
fn plugin_registers_apply_heal_wall() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    app.init_resource::<PendingWallHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_wall_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let wall = app
        .world_mut()
        .spawn((
            Wall,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingWallHeal(vec![HealDealt::<Wall> {
        healer:  None,
        target:  wall,
        amount:  2.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(wall).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Wall Hp should be 7.0 after 2.0 heal, got {}",
        hp.current
    );
}

#[test]
fn plugin_registers_apply_heal_breaker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    app.init_resource::<PendingBreakerHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_breaker_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let breaker = app
        .world_mut()
        .spawn((
            Breaker,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingBreakerHeal(vec![HealDealt::<Breaker> {
        healer:  None,
        target:  breaker,
        amount:  2.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(breaker).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Breaker Hp should be 7.0 after 2.0 heal, got {}",
        hp.current
    );
}

#[test]
fn plugin_registers_apply_heal_salvo() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);
    app.init_resource::<PendingSalvoHealPlugin>();
    app.add_systems(
        FixedUpdate,
        enqueue_salvo_heal_plugin.before(DeathPipelineSystems::ApplyHeal),
    );

    let salvo = app
        .world_mut()
        .spawn((
            Salvo,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingSalvoHealPlugin(vec![HealDealt::<Salvo> {
        healer:  None,
        target:  salvo,
        amount:  2.0,
        source:  None,
        cap:     HealCap::Max,
        _marker: PhantomData,
    }]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(salvo).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Salvo Hp should be 7.0 after 2.0 heal, got {}",
        hp.current
    );
}
