//! Tests for `detect_deaths<T>`.

use super::helpers::{TestEntity, build_detect_deaths_app};
use crate::{
    prelude::*,
    shared::death_pipeline::{kill_yourself::KillYourself, killed_by::KilledBy},
};

#[test]
fn detect_deaths_sends_kill_yourself_when_hp_zero() {
    let mut app = build_detect_deaths_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<TestEntity>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "detect_deaths should send exactly one KillYourself message"
    );
    assert_eq!(
        collector.0[0].victim, entity,
        "KillYourself victim should be the entity with Hp <= 0"
    );
}

#[test]
fn detect_deaths_sends_kill_yourself_when_hp_negative() {
    let mut app = build_detect_deaths_app();
    let entity = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp {
                current:  -5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<TestEntity>>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].victim, entity);
}

#[test]
fn detect_deaths_does_not_send_for_positive_hp() {
    let mut app = build_detect_deaths_app();
    app.world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default()));

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<TestEntity>>>();
    assert!(
        collector.0.is_empty(),
        "detect_deaths should not send KillYourself for entities with positive Hp"
    );
}

#[test]
fn detect_deaths_skips_dead_entities() {
    let mut app = build_detect_deaths_app();
    app.world_mut().spawn((
        TestEntity,
        Hp {
            current:  0.0,
            starting: 10.0,
            max:      None,
        },
        KilledBy::default(),
        Dead,
    ));

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<TestEntity>>>();
    assert!(
        collector.0.is_empty(),
        "detect_deaths should skip entities with Dead marker"
    );
}

#[test]
fn detect_deaths_includes_killer_from_killed_by() {
    let mut app = build_detect_deaths_app();
    let dealer = app.world_mut().spawn_empty().id();
    app.world_mut().spawn((
        TestEntity,
        Hp {
            current:  0.0,
            starting: 10.0,
            max:      None,
        },
        KilledBy {
            dealer: Some(dealer),
        },
    ));

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<TestEntity>>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].killer,
        Some(dealer),
        "KillYourself.killer should carry the dealer from KilledBy"
    );
}
