//! Tests for shield reflection cost — each bolt reflection off the `ShieldWall`
//! deducts `ShieldReflectionCost` from `ShieldWallTimer`.

use bevy::prelude::*;

use super::helpers::*;
use crate::bolt::messages::BoltImpactWall;

// ── Message capture pattern for BoltImpactWall ─────────────────────────

#[derive(Resource, Default)]
struct TestBoltImpactWallMessages(Vec<BoltImpactWall>);

fn enqueue_bolt_impact_wall(
    msg_res: Res<TestBoltImpactWallMessages>,
    mut writer: MessageWriter<BoltImpactWall>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

/// App with only the deduction system (no timer tick system).
/// This isolates reflection cost behavior from natural timer decay.
fn deduct_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactWall>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_bolt_impact_wall.before(deduct_shield_on_reflection),
                deduct_shield_on_reflection,
            ),
        );
    app
}

// ── Behavior 1: deduct_shield_on_reflection subtracts time on bolt-wall impact ──

#[test]
fn deduct_subtracts_reflection_cost_from_timer_on_bolt_impact() {
    let mut app = deduct_test_app();

    let bolt = app.world_mut().spawn_empty().id();
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(3.0, TimerMode::Once)),
            ShieldReflectionCost(0.5),
        ))
        .id();

    app.init_resource::<TestBoltImpactWallMessages>();
    app.world_mut()
        .resource_mut::<TestBoltImpactWallMessages>()
        .0 = vec![BoltImpactWall { bolt, wall }];

    tick(&mut app);

    let timer = app.world().get::<ShieldWallTimer>(wall).unwrap();
    // Only the deduction system runs (no tick_shield_wall_timer in this app).
    // Expected: 3.0 - 0.5 = 2.5
    let expected = 2.5;
    assert!(
        (timer.0.remaining_secs() - expected).abs() < 0.05,
        "timer should have ~{expected:.1} remaining after 0.5 reflection cost deduction, got {:.4}",
        timer.0.remaining_secs()
    );
}

// ── Behavior 2: deduct does nothing when impacted wall is NOT ShieldWall ──

#[test]
fn deduct_does_nothing_when_impacted_wall_is_not_shield_wall() {
    let mut app = deduct_test_app();

    let bolt = app.world_mut().spawn_empty().id();
    let regular_wall = app.world_mut().spawn(Transform::default()).id();
    let shield_wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(3.0, TimerMode::Once)),
            ShieldReflectionCost(0.5),
        ))
        .id();

    app.init_resource::<TestBoltImpactWallMessages>();
    app.world_mut()
        .resource_mut::<TestBoltImpactWallMessages>()
        .0 = vec![BoltImpactWall {
        bolt,
        wall: regular_wall,
    }];

    tick(&mut app);

    let timer = app.world().get::<ShieldWallTimer>(shield_wall).unwrap();
    // No deduction should occur because the impacted wall is not a ShieldWall.
    // Timer remains at 3.0 (no tick_shield_wall_timer in this app either).
    assert!(
        (timer.0.remaining_secs() - 3.0).abs() < 0.01,
        "shield timer should remain at ~3.0 when a non-shield wall is hit, got {:.4}",
        timer.0.remaining_secs()
    );
}

// ── Behavior 3: multiple reflections in one frame each deduct ──

#[test]
fn multiple_reflections_in_one_frame_each_deduct() {
    let mut app = deduct_test_app();

    let bolt_a = app.world_mut().spawn_empty().id();
    let bolt_b = app.world_mut().spawn_empty().id();
    let bolt_c = app.world_mut().spawn_empty().id();
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(3.0, TimerMode::Once)),
            ShieldReflectionCost(0.5),
        ))
        .id();

    app.init_resource::<TestBoltImpactWallMessages>();
    app.world_mut()
        .resource_mut::<TestBoltImpactWallMessages>()
        .0 = vec![
        BoltImpactWall { bolt: bolt_a, wall },
        BoltImpactWall { bolt: bolt_b, wall },
        BoltImpactWall { bolt: bolt_c, wall },
    ];

    tick(&mut app);

    let timer = app.world().get::<ShieldWallTimer>(wall).unwrap();
    // 3 reflections x 0.5 = 1.5 subtracted. Expected: 3.0 - 1.5 = 1.5
    let expected = 1.5;
    assert!(
        (timer.0.remaining_secs() - expected).abs() < 0.05,
        "3 reflections should deduct 1.5s total. Expected ~{expected:.1}, got {:.4}",
        timer.0.remaining_secs()
    );
}

// ── Behavior 4: timer deducted below zero causes despawn on next tick ──

#[test]
fn timer_deducted_below_zero_causes_despawn_on_next_timer_tick() {
    // Use a full app with both deduction and timer systems registered.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<BoltImpactWall>();
    register(&mut app);
    // Add message enqueue system before deduction (register() adds deduction).
    app.add_systems(
        FixedUpdate,
        enqueue_bolt_impact_wall.before(deduct_shield_on_reflection),
    );

    let bolt = app.world_mut().spawn_empty().id();
    let wall = app
        .world_mut()
        .spawn((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(0.3, TimerMode::Once)),
            ShieldReflectionCost(0.5),
        ))
        .id();

    // First tick: deduction of 0.5 from 0.3 timer makes remaining <= 0
    app.init_resource::<TestBoltImpactWallMessages>();
    app.world_mut()
        .resource_mut::<TestBoltImpactWallMessages>()
        .0 = vec![BoltImpactWall { bolt, wall }];

    tick(&mut app);

    // After deduction of 0.5 from a 0.3 timer, remaining should be <= 0.
    // The tick_shield_wall_timer should then detect finished and despawn it.
    assert!(
        app.world().get_entity(wall).is_err(),
        "ShieldWall should be despawned when timer goes below zero from reflection cost"
    );
}

// ── Behavior 5: EffectKind::Shield has reflection_cost field ──

#[test]
fn effect_kind_shield_has_reflection_cost_field() {
    use crate::effect::EffectKind;
    // Compile-time verification: Shield { duration, reflection_cost } compiles.
    let shield = EffectKind::Shield {
        duration: 3.0,
        reflection_cost: 0.5,
    };
    drop(shield);
}

// ── Behavior 6: fire() stores reflection_cost on the wall entity ──

#[test]
fn fire_stores_reflection_cost_on_wall_entity() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 3.0, 0.5, "parry", &mut world);

    let costs: Vec<&ShieldReflectionCost> = world
        .query_filtered::<&ShieldReflectionCost, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(
        costs.len(),
        1,
        "ShieldWall should have ShieldReflectionCost component"
    );
    assert!(
        (costs[0].0 - 0.5).abs() < f32::EPSILON,
        "ShieldReflectionCost should be 0.5, got {}",
        costs[0].0
    );
}

#[test]
fn fire_stores_zero_reflection_cost() {
    // Edge case: reflection_cost=0.0 means no deduction per reflection
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 3.0, 0.0, "parry", &mut world);

    let costs: Vec<&ShieldReflectionCost> = world
        .query_filtered::<&ShieldReflectionCost, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(
        costs.len(),
        1,
        "ShieldWall should have ShieldReflectionCost even when 0.0"
    );
    assert!(
        (costs[0].0).abs() < f32::EPSILON,
        "ShieldReflectionCost should be 0.0, got {}",
        costs[0].0
    );
}
