use bevy::{ecs::world::CommandQueue, prelude::*};
use rand::Rng;
use rantzsoft_spatial2d::components::PreviousPosition;

use super::*;
use crate::{
    bolt::{
        components::{BoltAngleSpread, BoltSpawnOffsetY, ExtraBolt, PiercingRemaining},
        definition::BoltDefinition,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    prelude::*,
    shared::{
        GameDrawLayer,
        rng::{BoltRng, GameRng, derive_seed, derive_seed_named},
    },
    state::run::NodeOutcome,
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

fn make_default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name:                 "Bolt".to_string(),
        base_speed:           720.0,
        min_speed:            360.0,
        max_speed:            1440.0,
        radius:               14.0,
        base_damage:          10.0,
        effects:              vec![],
        color_rgb:            [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical:   5.0,
        min_radius:           None,
        max_radius:           None,
    }
}

fn test_app() -> App {
    TestAppBuilder::new()
        .with_message::<crate::bolt::messages::BoltSpawned>()
        .with_resource::<NodeOutcome>()
        .with_resource::<BoltRng>()
        .with_system(Update, reset_bolt)
        .build()
}

/// Spawns a bolt entity via `.definition()` for reset testing.
fn spawn_bolt_entity(app: &mut App, pos: Vec2, velocity: Velocity2D) -> Entity {
    let def = make_default_bolt_definition();
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(pos)
            .definition(&def)
            .with_velocity(velocity)
            .primary()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    entity
}

/// Spawns a breaker entity at the given position using `Position2D`.
fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(x, y)),
            rantzsoft_spatial2d::components::Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id()
}

// ── Existing behavioral tests (migrated from .config() to .definition()) ──

#[test]
fn reset_bolt_writes_position2d_above_breaker() {
    let mut app = test_app();
    spawn_bolt_entity(
        &mut app,
        Vec2::new(150.0, 100.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let expected = Vec2::new(0.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    let position = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have Position2D");

    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "bolt Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn reset_bolt_snaps_previous_position_to_prevent_interpolation_teleport() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(150.0, 100.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let pos = app
        .world()
        .get::<Position2D>(bolt_id)
        .expect("bolt should have Position2D");
    let prev = app
        .world()
        .get::<PreviousPosition>(bolt_id)
        .expect("bolt should have PreviousPosition");
    assert_eq!(
        pos.0, prev.0,
        "PreviousPosition should match Position2D after reset to prevent teleport"
    );
}

#[test]
fn reset_bolt_zeroes_velocity_on_node_zero() {
    let mut app = test_app();
    spawn_bolt_entity(
        &mut app,
        Vec2::new(0.0, 0.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");

    assert!(
        velocity.0 == Vec2::ZERO,
        "velocity should be zero on node 0, got {:?}",
        velocity.0
    );
}

#[test]
fn reset_bolt_sets_initial_velocity_on_subsequent_nodes() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
    spawn_bolt_entity(
        &mut app,
        Vec2::new(0.0, 0.0),
        Velocity2D(Vec2::new(0.0, 0.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");

    assert!(
        velocity.0.y > 0.0,
        "velocity y should be positive on subsequent node, got {}",
        velocity.0.y
    );
    let speed = velocity.speed();
    // Definition base_speed is 720.0
    assert!(
        (speed - 720.0).abs() < 2.0,
        "speed should be approximately 720.0 (definition base_speed), got {speed:.1}"
    );
}

#[test]
fn reset_bolt_inserts_serving_on_node_zero() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(0.0, 0.0),
        Velocity2D(Vec2::new(0.0, 0.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    assert!(
        app.world().get::<BoltServing>(bolt_id).is_some(),
        "bolt should have BoltServing on node 0"
    );
}

#[test]
fn reset_bolt_removes_serving_on_subsequent_nodes() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    let def = make_default_bolt_definition();
    let bolt_id = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&def)
                .serving()
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    assert!(
        app.world().get::<BoltServing>(bolt_id).is_none(),
        "bolt should NOT have BoltServing on node 1"
    );
}

#[test]
fn reset_bolt_resets_piercing_remaining_to_active_piercings_total() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    app.world_mut().entity_mut(bolt_id).insert((
        crate::bolt::test_utils::piercing_stack(&[3]),
        PiercingRemaining(0),
    ));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let remaining = app
        .world()
        .get::<PiercingRemaining>(bolt_id)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        remaining.0, 3,
        "PiercingRemaining should be reset to aggregate = 3.0, got {}",
        remaining.0
    );
}

#[test]
fn reset_bolt_preserves_effect_state() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    app.world_mut().entity_mut(bolt_id).insert((
        crate::bolt::test_utils::damage_stack(&[1.5]),
        crate::bolt::test_utils::speed_stack(&[1.2]),
        crate::bolt::test_utils::piercing_stack(&[3]),
        PiercingRemaining(0),
    ));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let world = app.world();

    let active_dmg = world
        .get::<DamageBoostStack>(bolt_id)
        .expect("DamageBoostStack should be present");
    assert!(
        !active_dmg.is_empty(),
        "DamageBoostStack should be non-empty after reset"
    );
    assert!(
        (active_dmg.aggregate_persistent(None) - 1.5).abs() <= f32::EPSILON,
        "DamageBoostStack aggregate should be 1.5 after reset, got {}",
        active_dmg.aggregate_persistent(None)
    );

    let active_spd = world
        .get::<EffectStack<SpeedBoostConfig>>(bolt_id)
        .expect("EffectStack<SpeedBoostConfig> should be present");
    assert_eq!(
        active_spd.len(),
        1,
        "EffectStack<SpeedBoostConfig> should have 1 entry after reset"
    );

    let pr = world
        .get::<PiercingRemaining>(bolt_id)
        .expect("PiercingRemaining should be present");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should be reset to aggregate = 3.0, got {}",
        pr.0
    );
}

#[test]
fn reset_bolt_is_noop_when_no_bolt_exists() {
    let mut app = test_app();
    spawn_breaker(&mut app, 0.0, -250.0);

    // Should not panic
    app.update();

    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(bolt_count, 0, "no bolt should be created by reset");
}

#[test]
fn reset_bolt_ignores_extra_bolt_entities() {
    let mut app = test_app();
    let baseline_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(150.0, 100.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );

    let extra_id = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(200.0, 300.0)),
            Position2D(Vec2::new(-100.0, 50.0)),
            PreviousPosition(Vec2::new(-100.0, 50.0)),
        ))
        .id();
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y;

    let baseline_pos = app.world().get::<Position2D>(baseline_id).unwrap();
    assert!(
        (baseline_pos.0.y - expected_y).abs() < f32::EPSILON,
        "baseline bolt should be repositioned to y={expected_y}, got y={}",
        baseline_pos.0.y,
    );

    let extra_pos = app.world().get::<Position2D>(extra_id).unwrap();
    assert!(
        (extra_pos.0.x - (-100.0)).abs() < f32::EPSILON,
        "extra bolt x should be unchanged at -100.0, got {}",
        extra_pos.0.x,
    );
    assert!(
        (extra_pos.0.y - 50.0).abs() < f32::EPSILON,
        "extra bolt y should be unchanged at 50.0, got {}",
        extra_pos.0.y,
    );
}

#[test]
fn reset_bolt_resets_piercing_remaining_from_multi_entry_active_piercings() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    app.world_mut().entity_mut(bolt_id).insert((
        crate::bolt::test_utils::piercing_stack(&[2, 1]),
        PiercingRemaining(0),
    ));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let remaining = app
        .world()
        .get::<PiercingRemaining>(bolt_id)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        remaining.0, 3,
        "PiercingRemaining should be reset to aggregate = 3.0, got {}",
        remaining.0
    );
}

// ── Migration tests: Behaviors 11-19 ──

// Behavior 11: reset_bolt reads BoltAngleSpread from bolt entity
#[test]
fn reset_bolt_reads_angle_spread_from_entity() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(velocity.0.y > 0.0, "bolt should launch upward");

    let angle = velocity.0.x.atan2(velocity.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "launch angle {angle:.3} rad should be within BoltAngleSpread ({DEFAULT_BOLT_ANGLE_SPREAD:.3} rad)"
    );
}

#[test]
fn reset_bolt_zero_angle_spread_launches_straight_up() {
    // Edge case: BoltAngleSpread(0.0) -- bolt launches straight up
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
    let def = BoltDefinition {
        min_angle_horizontal: 0.0,
        min_angle_vertical: 0.0,
        ..make_default_bolt_definition()
    };
    let bolt_id = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::ZERO))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    // Override angle spread to 0.0 after builder inserts it
    app.world_mut()
        .entity_mut(bolt_id)
        .insert(BoltAngleSpread(0.0));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        velocity.0.x.abs() < 0.01,
        "zero angle spread should launch straight up (vx ~ 0), got vx={:.3}",
        velocity.0.x
    );
}

// Behavior 12: reset_bolt reads BoltSpawnOffsetY from bolt entity
#[test]
fn reset_bolt_reads_spawn_offset_y_from_entity() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(100.0, 200.0),
        Velocity2D(Vec2::new(300.0, 400.0)),
    );
    spawn_breaker(&mut app, 42.0, -250.0);

    app.update();

    let pos = app.world().get::<Position2D>(bolt_id).unwrap();
    let expected = Vec2::new(42.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0.x - expected.x).abs() < f32::EPSILON && (pos.0.y - expected.y).abs() < f32::EPSILON,
        "bolt position should be {expected:?} (from entity BoltSpawnOffsetY), got {:?}",
        pos.0
    );
}

#[test]
fn reset_bolt_zero_spawn_offset_resets_to_breaker_y() {
    // Edge case: BoltSpawnOffsetY(0.0) -> bolt resets to breaker Y exactly
    let mut app = test_app();
    let def = make_default_bolt_definition();
    let bolt_id = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::new(100.0, 200.0))
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::ZERO))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    // Override offset to 0.0
    app.world_mut()
        .entity_mut(bolt_id)
        .insert(BoltSpawnOffsetY(0.0));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let pos = app.world().get::<Position2D>(bolt_id).unwrap();
    assert!(
        (pos.0.y - (-250.0)).abs() < f32::EPSILON,
        "zero offset should reset bolt to breaker Y exactly (-250.0), got {}",
        pos.0.y
    );
}

// Behavior 13: reset_bolt reads BaseSpeed from bolt entity
#[test]
fn reset_bolt_uses_base_speed_from_entity() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt_id).unwrap();
    let speed = velocity.speed();
    // Definition base_speed is 720.0 (NOT BoltConfig default 400.0)
    assert!(
        (speed - 720.0).abs() < 2.0,
        "speed should be approximately 720.0 (entity BaseSpeed), got {speed:.1}"
    );
}

// Behavior 14: reset_bolt zeroes velocity and inserts BoltServing on node_index 0
#[test]
fn reset_bolt_zeroes_velocity_and_inserts_serving_on_node_zero_with_definition_bolt() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::new(300.0, 400.0)));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        vel.0 == Vec2::ZERO,
        "velocity should be zero on node 0, got {:?}",
        vel.0
    );
    assert!(
        app.world().get::<BoltServing>(bolt_id).is_some(),
        "bolt should have BoltServing on node 0"
    );

    let pos = app.world().get::<Position2D>(bolt_id).unwrap();
    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "position y should be {expected_y}, got {}",
        pos.0.y
    );
}

// Behavior 15: reset_bolt does NOT read Res<BoltConfig>
#[test]
fn reset_bolt_uses_entity_values_not_bolt_config() {
    // Given: BoltConfig with spawn_offset_y: 30.0, base_speed: 400.0.
    //        Bolt entity with BoltSpawnOffsetY(54.0), BaseSpeed(720.0).
    // Then: Position uses entity offset (54.0), NOT config offset (30.0).
    //       Speed is ~720.0, NOT 400.0.
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let pos = app.world().get::<Position2D>(bolt_id).unwrap();
    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y; // -196.0
    let config_y = -250.0 + 30.0; // -220.0
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "position y should be {expected_y} (from entity component), NOT {config_y} (from BoltConfig). Got {}",
        pos.0.y
    );

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    let speed = vel.speed();
    assert!(
        (speed - 720.0).abs() < 2.0,
        "speed should be approximately 720.0 (entity BaseSpeed), NOT 400.0 (BoltConfig). Got {speed:.1}"
    );
}

// Behavior 16: reset_bolt uses random angle within BoltAngleSpread
#[test]
fn reset_bolt_uses_random_angle_within_spread() {
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 3;
    let def = make_default_bolt_definition();
    let bolt_id = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::ZERO))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    // Set a specific angle spread
    app.world_mut()
        .entity_mut(bolt_id)
        .insert(BoltAngleSpread(0.3));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let velocity = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(velocity.0.y > 0.0, "bolt should launch upward");
    let angle = velocity.0.x.atan2(velocity.0.y).abs();
    assert!(
        angle <= 0.3 + 0.01,
        "launch angle {angle:.3} should be within BoltAngleSpread (0.3 rad)"
    );
}

// Behavior 17: reset_bolt snaps PreviousPosition to new position (definition-built)
#[test]
fn reset_bolt_snaps_previous_position_definition_built() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(
        &mut app,
        Vec2::new(200.0, 300.0),
        Velocity2D(Vec2::new(100.0, -200.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let pos = app.world().get::<Position2D>(bolt_id).unwrap();
    let prev = app.world().get::<PreviousPosition>(bolt_id).unwrap();
    let expected = Vec2::new(0.0, -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y);
    assert!(
        (pos.0 - expected).length() < f32::EPSILON,
        "Position2D should be {expected:?}, got {:?}",
        pos.0
    );
    assert!(
        (prev.0 - expected).length() < f32::EPSILON,
        "PreviousPosition should match Position2D at {expected:?}, got {:?}",
        prev.0
    );
}

// Behavior 18: reset_bolt ignores ExtraBolt entities (with definition-built bolts)
#[test]
fn reset_bolt_ignores_extra_bolt_with_definition_built() {
    let mut app = test_app();
    let def = make_default_bolt_definition();

    // Baseline bolt
    let baseline_id = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::new(150.0, 100.0))
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::new(300.0, 400.0)))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };

    // Extra bolt
    let extra_id = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::new(-100.0, 50.0))
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::new(200.0, 300.0)))
                .extra()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };

    spawn_breaker(&mut app, 0.0, -250.0);
    app.update();

    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    let baseline_pos = app.world().get::<Position2D>(baseline_id).unwrap();
    assert!(
        (baseline_pos.0.y - expected_y).abs() < f32::EPSILON,
        "baseline bolt should be repositioned, got y={}",
        baseline_pos.0.y,
    );

    let extra_vel = app.world().get::<Velocity2D>(extra_id).unwrap();
    assert!(
        (extra_vel.0.x - 200.0).abs() < f32::EPSILON,
        "extra bolt velocity should be unchanged"
    );
}

// Behavior 19: reset_bolt system signature no longer includes Res<BoltConfig>
#[test]
fn reset_bolt_runs_without_bolt_config_resource() {
    // Given: No BoltConfig resource inserted.
    // When: reset_bolt runs with a definition-built bolt.
    // Then: System runs without panic (proves BoltConfig is not a system parameter).
    // Deliberately NOT inserting BoltConfig
    let mut app = test_app();

    let def = make_default_bolt_definition();
    {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::new(100.0, 200.0)))
                .primary()
                .headless()
                .spawn(&mut commands);
        }
        queue.apply(world);
    }
    spawn_breaker(&mut app, 0.0, -250.0);

    // Should not panic if BoltConfig is no longer a system parameter
    app.update();
}

// ── Group D (Behavior 10) — reset_bolt uses BoltRng ──────────────────────────

// Behavior 10: reset_bolt reads ResMut<BoltRng> — harness has only BoltRng
#[test]
fn reset_bolt_uses_bolt_rng_not_game_rng_on_subsequent_node() {
    // test_app() now registers BoltRng instead of GameRng.
    // If reset_bolt still names GameRng, the app panics here.
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
    spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should exist after reset");

    assert!(
        (vel.0.length() - 720.0).abs() < 2.0,
        "bolt speed should be ~720.0, got {}",
        vel.0.length()
    );
    assert!(vel.0.y > 0.0, "bolt should launch upward");
}

// Behavior 10 edge case: BoltRng AND GameRng present — system must NOT advance GameRng
#[test]
fn reset_bolt_does_not_advance_game_rng_on_subsequent_node() {
    let mut app = test_app();
    app.insert_resource(GameRng::from_seed(SENTINEL));
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
    spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    // Sentinel: bolt launched upward proves system ran
    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "bolt launched — system executed");

    let actual: u64 = app.world_mut().resource_mut::<GameRng>().0.random();
    let expected: u64 = GameRng::from_seed(SENTINEL).0.random();
    assert_eq!(
        actual, expected,
        "reset_bolt must NOT advance GameRng on subsequent-node path"
    );
}

// ── Group D (Behavior 11) ─────────────────────────────────────────────────────

// Behavior 11: same BoltRng seed yields same subsequent-node launch angle
#[test]
fn reset_bolt_same_seed_produces_identical_velocity_subsequent_node() {
    let seed = 0xFEED_BEEF_u64;

    let vel_a = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
        spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
        spawn_breaker(&mut app, 0.0, -250.0);
        app.update();
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    let vel_b = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
        spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
        spawn_breaker(&mut app, 0.0, -250.0);
        app.update();
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    assert!(
        (vel_a.0.x - vel_b.0.x).abs() < f32::EPSILON,
        "reset_bolt vx must be identical for same seed: a={}, b={}",
        vel_a.0.x,
        vel_b.0.x
    );
    assert!(
        (vel_a.0.y - vel_b.0.y).abs() < f32::EPSILON,
        "reset_bolt vy must be identical for same seed: a={}, b={}",
        vel_a.0.y,
        vel_b.0.y
    );
}

// Behavior 11 edge case: seed 0 vs seed 99 must produce different angles
#[test]
fn reset_bolt_different_seeds_produce_different_velocity_subsequent_node() {
    let launch_vel = |seed: u64| -> Velocity2D {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
        spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
        spawn_breaker(&mut app, 0.0, -250.0);
        app.update();
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    let vel_0 = launch_vel(0);
    let vel_99 = launch_vel(99);
    let diff_x = (vel_0.0.x - vel_99.0.x).abs();
    let diff_y = (vel_0.0.y - vel_99.0.y).abs();
    assert!(
        diff_x > 0.01 || diff_y > 0.01,
        "seeds 0 and 99 must produce different velocities (diff_x={diff_x:.4}, diff_y={diff_y:.4})"
    );
}

// ── Group D (Behavior 12) ─────────────────────────────────────────────────────

// Behavior 12: reset_bolt does NOT draw from BoltRng on node_index == 0
#[test]
fn reset_bolt_does_not_advance_bolt_rng_on_node_zero() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));
    // node_index defaults to 0
    spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::new(300.0, 400.0)));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0 == Vec2::ZERO,
        "velocity should be zero on node 0, got {:?}",
        vel.0
    );

    let actual: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    let fresh_first: u64 = BoltRng::from_seed(42).0.random();
    assert_eq!(
        actual, fresh_first,
        "BoltRng must NOT advance on node_index=0 (serving branch)"
    );
}

// Behavior 12 edge case: two bolts on node 0 — both get BoltServing, BoltRng un-advanced
#[test]
fn reset_bolt_does_not_advance_bolt_rng_on_node_zero_with_two_bolts() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));
    // Spawn two primary bolts
    spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::new(300.0, 400.0)));
    spawn_bolt_entity(
        &mut app,
        Vec2::new(50.0, 0.0),
        Velocity2D(Vec2::new(-300.0, 400.0)),
    );
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    // Both bolts should have zero velocity and BoltServing
    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolt_count, 2,
        "both bolts should have BoltServing on node 0"
    );

    let actual: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    let fresh_first: u64 = BoltRng::from_seed(42).0.random();
    assert_eq!(
        actual, fresh_first,
        "BoltRng must NOT advance when two bolts reset on node_index=0"
    );
}

// ── Group D (Behavior 13) ─────────────────────────────────────────────────────

// Behavior 13: reset_bolt draws from BoltRng exactly once per non-extra bolt on subsequent nodes
#[test]
fn reset_bolt_advances_bolt_rng_once_per_non_extra_bolt_on_subsequent_node() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 3;

    let def = make_default_bolt_definition();

    // Two primary bolts
    spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    spawn_bolt_entity(&mut app, Vec2::new(10.0, 0.0), Velocity2D(Vec2::ZERO));

    // One extra bolt (should be skipped by the query)
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(Vec2::new(20.0, 0.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(100.0, 200.0)))
            .extra()
            .headless()
            .spawn(&mut commands);
    }
    queue.apply(world);

    spawn_breaker(&mut app, 0.0, -250.0);
    app.update();

    // The extra bolt's velocity should be unchanged (skipped by Without<ExtraBolt> filter)
    let extra_vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<ExtraBolt>>()
        .iter(app.world())
        .next()
        .expect("extra bolt should still exist");
    assert!(
        (extra_vel.0.x - 100.0).abs() < f32::EPSILON,
        "extra bolt vx should be unchanged at 100.0, got {}",
        extra_vel.0.x
    );

    // BoltRng should have advanced exactly twice (once per primary bolt).
    // random_range::<f32> in rand 0.9.4 calls next_u32() once per call.
    // 2 bolts × 1 draw each = 2 u32 draws total.
    let mut reference = BoltRng::from_seed(42);
    let _: u32 = reference.0.random(); // bolt 1 (the only draw)
    let _: u32 = reference.0.random(); // bolt 2 (the only draw)
    let expected_next: u64 = reference.0.random();

    let actual_next: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    assert_eq!(
        actual_next, expected_next,
        "BoltRng should have advanced exactly twice (1 u32 draw per primary bolt), \
         so the next u64 draw should equal the second u64 of a fresh BoltRng::from_seed(42)"
    );
}

// ── Group F (Behavior 16) — reset_bolt determinism via production seed formula ─

// Behavior 16: reset_bolt with production formula seed produces stable velocity
#[test]
fn reset_bolt_with_production_bolt_rng_seed_is_deterministic() {
    let bolt_rng_seed = derive_seed_named(derive_seed(42, 1u64), "bolt");

    let vel_a = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
        spawn_breaker(&mut app, 0.0, -250.0);
        app.update();
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    let vel_b = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
        spawn_breaker(&mut app, 0.0, -250.0);
        app.update();
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    assert!(
        (vel_a.0.x - vel_b.0.x).abs() < f32::EPSILON,
        "reset_bolt with production formula seed must be deterministic (vx mismatch)"
    );
    assert!(
        (vel_a.0.y - vel_b.0.y).abs() < f32::EPSILON,
        "reset_bolt with production formula seed must be deterministic (vy mismatch)"
    );
}

// Behavior 16 edge case: different node_index yields different velocity
#[test]
fn reset_bolt_different_node_index_yields_different_velocity_with_production_seed() {
    let vel_for_node = |node_index: u32| -> Velocity2D {
        let bolt_rng_seed = derive_seed_named(derive_seed(42, u64::from(node_index)), "bolt");
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = node_index;
        spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
        spawn_breaker(&mut app, 0.0, -250.0);
        app.update();
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    let vel_1 = vel_for_node(1);
    let vel_2 = vel_for_node(2);
    let diff_x = (vel_1.0.x - vel_2.0.x).abs();
    let diff_y = (vel_1.0.y - vel_2.0.y).abs();
    assert!(
        diff_x > 0.01 || diff_y > 0.01,
        "node_index=1 and node_index=2 must produce different velocities with production formula \
         (diff_x={diff_x:.4}, diff_y={diff_y:.4})"
    );
}
