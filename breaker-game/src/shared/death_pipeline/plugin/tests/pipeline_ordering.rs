use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::shared::{
    death_pipeline::{
        damage_dealt::DamageDealt,
        dead::Dead,
        destroyed::Destroyed,
        heal_dealt::{HealCap, HealDealt},
        hp::Hp,
        killed_by::KilledBy,
        sets::DeathPipelineSystems,
    },
    test_utils::{MessageCollector, attach_message_capture, tick},
};

// ── Group I / Behaviors 26–29: Full pipeline ordering ──

/// Behavior 26: damage kill + heal in same tick → entity dies, heal blocked.
#[test]
fn damage_kills_before_heal_runs_max_cap() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 5.0, HealCap::Max)]));

    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "exactly one Destroyed<PluginTestEntity> should be emitted; heal must not prevent it"
    );

    // If the entity still exists, it must carry the Dead marker with KilledBy set.
    if app.world().get_entity(victim).is_ok() {
        assert!(
            app.world().get::<Dead>(victim).is_some(),
            "victim entity (if still alive) must carry the Dead marker"
        );
        let hp = app.world().get::<Hp>(victim).unwrap();
        assert!(
            hp.current <= 0.0,
            "victim Hp should be <= 0.0, heal must not revive, got {}",
            hp.current
        );
    }
}

/// Behavior 26 edge: massive heal still cannot revive.
#[test]
fn damage_kills_before_heal_runs_massive_heal_blocked() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
        victim,
        1_000_000.0,
        HealCap::Max,
    )]));

    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "massive heal should not prevent exactly 1 Destroyed; got {}",
        destroyed.0.len()
    );
}

/// Behavior 26 edge: 11.0 heal that would more-than-cover the overkill.
#[test]
fn damage_kills_before_heal_runs_cover_overkill_blocked() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 11.0, HealCap::Max)]));

    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "11.0 heal must not prevent death; exactly 1 Destroyed expected"
    );
}

/// Behavior 26 edge: cap-agnostic — Starting cap also cannot revive.
#[test]
fn damage_kills_before_heal_runs_starting_cap_blocked() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
        victim,
        1_000_000.0,
        HealCap::Starting,
    )]));

    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "Starting cap heal must not prevent death"
    );
}

/// Behavior 27: non-lethal damage + heal in same tick — additive.
#[test]
fn heal_after_non_lethal_damage_adds_in_same_tick() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp {
                current:  10.0,
                starting: 20.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 3.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 5.0, HealCap::Max)]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(victim).unwrap();
    assert!(
        (hp.current - 12.0).abs() < f32::EPSILON,
        "Non-lethal damage + heal should leave 12.0, got {}",
        hp.current
    );
    assert!(
        app.world().get::<Dead>(victim).is_none(),
        "victim must NOT be Dead — damage was non-lethal"
    );
    let killed_by = app.world().get::<KilledBy>(victim).unwrap();
    assert_eq!(killed_by.dealer, None, "KilledBy should remain unset");

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
    assert!(
        destroyed.0.is_empty(),
        "No Destroyed should be emitted for non-lethal damage"
    );
}

/// Behavior 27 edge: same result regardless of message enqueue order.
/// (The test app enqueues via separate systems so message order is governed
/// by set ordering, not enqueue ordering — this edge is an observational
/// restatement that the two systems run in their own sets.)
#[test]
fn heal_after_non_lethal_damage_order_insensitive() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp {
                current:  10.0,
                starting: 20.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 3.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(victim, 5.0, HealCap::Max)]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(victim).unwrap();
    assert!(
        (hp.current - 12.0).abs() < f32::EPSILON,
        "Order-independent result should still be 12.0, got {}",
        hp.current
    );
}

/// Behavior 28: tick 1 damage-kills, tick 2 heal is skipped.
#[test]
fn sequence_tick_kill_then_heal_is_skipped() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    // Tick 1: damage kills the victim.
    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 5.0)]));
    app.insert_resource(PendingPtHeal(Vec::<HealDealt<PluginTestEntity>>::new()));
    tick(&mut app);

    // After tick 1: collector holds the Destroyed from this tick.
    {
        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "tick 1 damage-kill must emit exactly 1 Destroyed",
        );
    }

    // Tick 2: massive heal targeting the now-dead/despawned victim.
    app.insert_resource(PendingPtDamage(Vec::<DamageDealt<PluginTestEntity>>::new()));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
        victim,
        100.0,
        HealCap::Max,
    )]));
    tick(&mut app);

    // After tick 2: collector is cleared at First of tick 2 and re-populated
    // only from tick-2 emissions. It must be empty — no second kill, no revival.
    {
        let destroyed = app
            .world()
            .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
        assert_eq!(
            destroyed.0.len(),
            0,
            "tick 2 heal must not emit Destroyed (no revival, no second kill)",
        );
    }

    // Final check: the victim is not alive with a positive HP. Either the
    // entity was despawned (query returns None) or, if it survived despawn
    // somehow, it still holds the Dead marker and HP did not rebound.
    let world = app.world();
    if let Some(hp) = world.get::<Hp>(victim) {
        assert!(
            hp.current <= 0.0,
            "heal must not rebound HP above zero on a dead entity; got {}",
            hp.current,
        );
        assert!(
            world.entity(victim).contains::<Dead>(),
            "victim must still hold Dead marker after tick-2 heal is skipped",
        );
    }
}

/// Behavior 29: observational proof that `apply_heal` runs AFTER `handle_kill`.
#[test]
fn apply_heal_runs_after_handle_kill_observationally() {
    let mut app = build_pipeline_app();
    attach_message_capture::<Destroyed<PluginTestEntity>>(&mut app);

    app.init_resource::<PendingPtDamage>();
    app.init_resource::<PendingPtHeal>();
    app.add_systems(
        FixedUpdate,
        enqueue_pt_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_pt_heal.before(DeathPipelineSystems::ApplyHeal),
    );

    let victim = app
        .world_mut()
        .spawn((
            PluginTestEntity,
            Hp::new(10.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingPtDamage(vec![pt_damage_msg(victim, 10.0)]));
    app.insert_resource(PendingPtHeal(vec![pt_heal_msg(
        victim,
        999.0,
        HealCap::Max,
    )]));

    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<PluginTestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "pinned ApplyDamage->DetectDeaths->HandleKill->ApplyHeal ordering should produce exactly 1 Destroyed"
    );

    // If the entity is still alive (before FixedPostUpdate despawn), Hp must be <= 0.
    if app.world().get_entity(victim).is_ok() {
        let hp = app.world().get::<Hp>(victim).unwrap();
        assert!(
            hp.current <= 0.0,
            "Hp should remain <= 0.0 — heal must NOT run before DetectDeaths; got {}",
            hp.current
        );
    }
}
