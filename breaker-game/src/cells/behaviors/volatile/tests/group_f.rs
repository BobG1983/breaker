//! Group F — Safety and idempotency.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::helpers::*;
use crate::{effect_v3::EffectV3Systems, prelude::*};

/// Builds a plugin-integration `App` for behaviors 28/28-edge that drive the
/// death bridge directly via injected `Destroyed<Cell>` messages (no damage
/// pipeline). Includes `RantzPhysics2dPlugin` so `ExplodeConfig::fire()` can
/// query the quadtree.
fn build_injection_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_physics()
        .with_effects_pipeline()
        .build();
    attach_message_capture::<Destroyed<Cell>>(&mut app);
    attach_message_capture::<DamageDealt<Cell>>(&mut app);
    app.init_resource::<TestCellDestroyedMessages>();
    app.add_systems(
        FixedUpdate,
        inject_cell_destroyed.before(EffectV3Systems::Death),
    );
    app
}

// Behavior 28
#[test]
fn volatile_cell_already_dead_still_fires_explosion_exactly_once() {
    let mut app = build_injection_test_app();

    let source = spawn_dead_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 10.0);

    app.insert_resource(TestCellDestroyedMessages(vec![Destroyed::<Cell> {
        victim:     source,
        killer:     None,
        victim_pos: Vec2::new(0.0, 0.0),
        killer_pos: None,
        _marker:    PhantomData,
    }]));

    tick(&mut app);

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let target_hits = damage
        .0
        .iter()
        .filter(|m| m.target == target && (m.amount - 25.0).abs() < f32::EPSILON)
        .count();
    assert_eq!(
        target_hits, 1,
        "the explosion must fire exactly once and deliver 25.0 to target"
    );

    assert!(
        app.world().get_entity(source).is_ok(),
        "source was never passed through handle_kill — it should still be present"
    );
}

// Behavior 28 edge: injecting the same Destroyed<Cell> twice => explosion fires twice
#[test]
fn injecting_destroyed_twice_fires_explosion_twice() {
    let mut app = build_injection_test_app();

    let source = spawn_dead_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 10.0);

    let msg = Destroyed::<Cell> {
        victim:     source,
        killer:     None,
        victim_pos: Vec2::new(0.0, 0.0),
        killer_pos: None,
        _marker:    PhantomData,
    };
    app.insert_resource(TestCellDestroyedMessages(vec![msg.clone(), msg]));

    tick(&mut app);

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let target_hits = damage.0.iter().filter(|m| m.target == target).count();
    assert_eq!(
        target_hits, 2,
        "two injected Destroyed<Cell> messages should drive the bridge twice"
    );
}

// Behavior 29
#[test]
fn volatile_detonation_skips_dead_targets_inside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target_live = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 25.0);
    let target_dead = spawn_dead_cell(&mut app, Vec2::new(-20.0, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion (target_dead is filtered);
    // tick 2 applies damage to target_live and kills it.
    let (destroyed_msgs, damage_msgs) = run_ticks_capture_destroyed_and_damage(&mut app, 2);

    assert!(app.world().get_entity(target_live).is_err());
    assert!(
        app.world().get_entity(target_dead).is_ok(),
        "dead target spawned with Dead should still be present"
    );
    let dead_hp = app.world().get::<Hp>(target_dead).unwrap();
    assert!((dead_hp.current - 25.0).abs() < f32::EPSILON);

    assert_eq!(
        damage_msgs.len(),
        2,
        "expected pending(100) + explosion_target_live(25) — dead target is filtered"
    );

    let victims: std::collections::HashSet<Entity> =
        destroyed_msgs.iter().map(|m| m.victim).collect();
    assert_eq!(
        victims,
        std::collections::HashSet::from([source, target_live])
    );
}

// Behavior 29 edge: second dead target at different position — also filtered
#[test]
fn volatile_detonation_skips_multiple_dead_targets_inside_radius() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 25.0, 40.0, 10.0);
    let target_live = spawn_plain_cell(&mut app, Vec2::new(20.0, 0.0), 25.0);
    let target_dead = spawn_dead_cell(&mut app, Vec2::new(-20.0, 0.0), 25.0);
    let target_dead2 = spawn_dead_cell(&mut app, Vec2::new(10.0, 10.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires explosion (both dead targets are
    // filtered); tick 2 applies damage to target_live and kills it.
    let (destroyed_msgs, damage_msgs) = run_ticks_capture_destroyed_and_damage(&mut app, 2);

    assert!(app.world().get_entity(target_live).is_err());
    assert!(app.world().get_entity(target_dead).is_ok());
    assert!(app.world().get_entity(target_dead2).is_ok());
    assert!((app.world().get::<Hp>(target_dead).unwrap().current - 25.0).abs() < f32::EPSILON);
    assert!((app.world().get::<Hp>(target_dead2).unwrap().current - 25.0).abs() < f32::EPSILON);

    assert_eq!(
        damage_msgs.len(),
        2,
        "still exactly pending(100) + explosion(25) — both dead targets filtered"
    );

    let victims: std::collections::HashSet<Entity> =
        destroyed_msgs.iter().map(|m| m.victim).collect();
    assert_eq!(
        victims,
        std::collections::HashSet::from([source, target_live])
    );
}

// Behavior 30
#[test]
fn volatile_small_radius_does_not_damage_target_outside() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 0.001, 0.001, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(0.5, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    tick(&mut app);

    assert!(app.world().get_entity(target).is_ok());
    let hp = app.world().get::<Hp>(target).unwrap();
    assert!((hp.current - 25.0).abs() < f32::EPSILON);

    let destroyed = app.world().resource::<MessageCollector<Destroyed<Cell>>>();
    assert_eq!(destroyed.0.len(), 1);
    assert_eq!(destroyed.0[0].victim, source);
}

// Behavior 30 edge: target just inside the 0.001 radius takes 0.001 damage
#[test]
fn volatile_small_radius_damages_target_just_inside() {
    let mut app = build_volatile_test_app();

    let source = spawn_volatile_cell(&mut app, Vec2::new(0.0, 0.0), 0.001, 0.001, 10.0);
    let target = spawn_plain_cell(&mut app, Vec2::new(0.0005, 0.0), 25.0);

    app.insert_resource(PendingCellDamage(vec![damage_msg(source, 100.0)]));

    // 2 ticks: tick 1 kills source + fires tiny explosion; tick 2 applies
    // 0.001 damage to target (target survives).
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(target).is_ok(),
        "target takes 0.001 damage but should not die (hp 25 > damage 0.001)"
    );
    let hp = app.world().get::<Hp>(target).unwrap();
    assert!(
        (hp.current - 24.999).abs() < 25.0f32.mul_add(f32::EPSILON, 1e-5),
        "target hp should be 24.999 within tolerance, got {}",
        hp.current
    );
}
