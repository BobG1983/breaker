use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use super::*;
use crate::{
    bolt::{
        components::{
            Bolt, BoltAngleSpread, BoltServing, BoltSpawnOffsetY, ExtraBolt, PiercingRemaining,
        },
        definition::BoltDefinition,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    breaker::components::Breaker,
    effect::effects::{
        damage_boost::ActiveDamageBoosts, piercing::ActivePiercings, speed_boost::ActiveSpeedBoosts,
    },
    run::RunState,
    shared::{GameDrawLayer, GameRng},
};

fn make_default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<RunState>()
        .init_resource::<GameRng>()
        .add_systems(Update, reset_bolt);
    app
}

/// Spawns a bolt entity via `.definition()` for reset testing.
fn spawn_bolt_entity(app: &mut App, pos: Vec2, velocity: Velocity2D) -> Entity {
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(pos)
        .definition(&def)
        .with_velocity(velocity)
        .primary()
        .spawn(app.world_mut())
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
    app.world_mut().resource_mut::<RunState>().node_index = 2;
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
    app.world_mut().resource_mut::<RunState>().node_index = 1;
    let def = make_default_bolt_definition();
    let bolt_id = Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .serving()
        .primary()
        .spawn(app.world_mut());
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
    app.world_mut()
        .entity_mut(bolt_id)
        .insert((ActivePiercings(vec![3]), PiercingRemaining(0)));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let remaining = app
        .world()
        .get::<PiercingRemaining>(bolt_id)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        remaining.0, 3,
        "PiercingRemaining should be reset to ActivePiercings.total() (3), got {}",
        remaining.0
    );
}

#[test]
fn reset_bolt_preserves_effect_state() {
    let mut app = test_app();
    let bolt_id = spawn_bolt_entity(&mut app, Vec2::ZERO, Velocity2D(Vec2::ZERO));
    app.world_mut().entity_mut(bolt_id).insert((
        ActiveDamageBoosts(vec![1.5]),
        ActiveSpeedBoosts(vec![1.2]),
        ActivePiercings(vec![3]),
        PiercingRemaining(0),
    ));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let world = app.world();

    let active_dmg = world
        .get::<ActiveDamageBoosts>(bolt_id)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        active_dmg.0,
        vec![1.5],
        "ActiveDamageBoosts should be unchanged after reset"
    );

    let active_spd = world
        .get::<ActiveSpeedBoosts>(bolt_id)
        .expect("ActiveSpeedBoosts should be present");
    assert_eq!(
        active_spd.0,
        vec![1.2],
        "ActiveSpeedBoosts should be unchanged after reset"
    );

    assert!(
        (active_dmg.multiplier() - 1.5).abs() < f32::EPSILON,
        "ActiveDamageBoosts multiplier should be 1.5 after reset, got {}",
        active_dmg.multiplier()
    );

    let pr = world
        .get::<PiercingRemaining>(bolt_id)
        .expect("PiercingRemaining should be present");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should be reset to ActivePiercings.total() (3), got {}",
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
    app.world_mut()
        .entity_mut(bolt_id)
        .insert((ActivePiercings(vec![2, 1]), PiercingRemaining(0)));
    spawn_breaker(&mut app, 0.0, -250.0);

    app.update();

    let remaining = app
        .world()
        .get::<PiercingRemaining>(bolt_id)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        remaining.0, 3,
        "PiercingRemaining should be reset to ActivePiercings.total() (2 + 1 = 3), got {}",
        remaining.0
    );
}

// ── Migration tests: Behaviors 11-19 ──

// Behavior 11: reset_bolt reads BoltAngleSpread from bolt entity
#[test]
fn reset_bolt_reads_angle_spread_from_entity() {
    let mut app = test_app();
    app.world_mut().resource_mut::<RunState>().node_index = 2;
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
    app.world_mut().resource_mut::<RunState>().node_index = 2;
    let def = BoltDefinition {
        min_angle_horizontal: 0.0,
        min_angle_vertical: 0.0,
        ..make_default_bolt_definition()
    };
    let bolt_id = Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::ZERO))
        .primary()
        .spawn(app.world_mut());
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
    let bolt_id = Bolt::builder()
        .at_position(Vec2::new(100.0, 200.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::ZERO))
        .primary()
        .spawn(app.world_mut());
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
    app.world_mut().resource_mut::<RunState>().node_index = 1;
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
    app.world_mut().resource_mut::<RunState>().node_index = 1;
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
    app.world_mut().resource_mut::<RunState>().node_index = 3;
    let def = make_default_bolt_definition();
    let bolt_id = Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::ZERO))
        .primary()
        .spawn(app.world_mut());
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
    let baseline_id = Bolt::builder()
        .at_position(Vec2::new(150.0, 100.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(300.0, 400.0)))
        .primary()
        .spawn(app.world_mut());

    // Extra bolt
    let extra_id = Bolt::builder()
        .at_position(Vec2::new(-100.0, 50.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(200.0, 300.0)))
        .extra()
        .spawn(app.world_mut());

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
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<RunState>()
        .init_resource::<GameRng>()
        // Deliberately NOT inserting BoltConfig
        .add_systems(Update, reset_bolt);

    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::ZERO)
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(100.0, 200.0)))
        .primary()
        .spawn(app.world_mut());
    spawn_breaker(&mut app, 0.0, -250.0);

    // Should not panic if BoltConfig is no longer a system parameter
    app.update();
}
