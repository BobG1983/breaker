use bevy::{ecs::world::CommandQueue, prelude::*};

use super::system::*;
use crate::{
    bolt::{components::*, definition::BoltDefinition, resources::DEFAULT_BOLT_ANGLE_SPREAD},
    input::resources::GameAction,
    prelude::*,
};

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
        .with_resource::<InputActions>()
        .with_resource::<GameRng>()
        .with_system(FixedUpdate, launch_bolt)
        .build()
}

/// Spawns a serving bolt using the builder with `.definition()`.
fn spawn_serving_bolt(app: &mut App) -> Entity {
    let def = make_default_bolt_definition();
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
}

// ── Existing behavioral tests (migrated to .definition()) ──

#[test]
fn bump_launches_serving_bolt() {
    let mut app = test_app();

    spawn_serving_bolt(&mut app);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    let serving_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        serving_count, 0,
        "BoltServing should be removed after launch"
    );

    let velocity = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");
    assert!(velocity.0.y > 0.0, "bolt should launch upward");
    assert!(velocity.speed() > 0.0, "bolt should have non-zero speed");
}

#[test]
fn no_input_keeps_serving() {
    let mut app = test_app();

    spawn_serving_bolt(&mut app);

    tick(&mut app);

    let serving_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        serving_count, 1,
        "bolt should still be serving without input"
    );

    let velocity = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");
    assert!(
        velocity.speed() < f32::EPSILON,
        "serving bolt should have zero velocity"
    );
}

#[test]
fn non_serving_bolt_unaffected() {
    let mut app = test_app();

    // Bolt without BoltServing
    app.world_mut()
        .spawn((Bolt, Velocity2D(Vec2::new(100.0, 300.0))));

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    let velocity = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .expect("bolt should have velocity");
    assert!(
        (velocity.0.x - 100.0).abs() < f32::EPSILON,
        "non-serving bolt velocity should be unchanged"
    );
}

// ── Migration tests: Behaviors 27-33 ──

// Behavior 27: launch_bolt queries BoltAngleSpread instead of BoltInitialAngle
#[test]
fn launch_bolt_uses_angle_spread_not_initial_angle() {
    // Given: Serving bolt built via .definition() with BoltAngleSpread(0.524).
    //        InputActions contains Bump.
    // Then: BoltServing removed. Velocity non-zero, upward. Angle within 0.524 rad.
    let mut app = test_app();

    let bolt_id = spawn_serving_bolt(&mut app);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    assert!(
        app.world().get::<BoltServing>(bolt_id).is_none(),
        "BoltServing should be removed after launch"
    );

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(vel.0.y > 0.0, "bolt should launch upward");

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= DEFAULT_BOLT_ANGLE_SPREAD + 0.01,
        "launch angle {angle:.3} should be within BoltAngleSpread ({DEFAULT_BOLT_ANGLE_SPREAD:.3})"
    );
}

#[test]
fn launch_bolt_zero_angle_spread_launches_straight_up() {
    // Edge case: BoltAngleSpread(0.0) -> bolt launches straight up
    let mut app = test_app();
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
                .serving()
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    app.world_mut()
        .entity_mut(bolt_id)
        .insert(BoltAngleSpread(0.0));

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        vel.0.x.abs() < 0.01,
        "zero angle spread should launch straight up, got vx={:.3}",
        vel.0.x
    );
}

// Behavior 28: launch_bolt uses random angle from GameRng within spread
#[test]
fn launch_bolt_uses_random_angle_within_spread() {
    let mut app = test_app();
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
    app.world_mut()
        .entity_mut(bolt_id)
        .insert(BoltAngleSpread(0.3));

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(vel.0.y > 0.0, "bolt should launch upward");
    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= 0.3 + 0.01,
        "launch angle {angle:.3} should be within BoltAngleSpread (0.3 rad)"
    );
}

// Behavior 29: no bump input does not launch (already covered above)

// Behavior 30: non-serving bolts unaffected (already covered above)

// Behavior 31: launch_bolt uses BoltAngleSpread (BoltInitialAngle deleted in Wave 6)
#[test]
fn launch_bolt_works_with_definition_built_bolt() {
    // Given: Bolt built via .definition() with BoltAngleSpread.
    //        InputActions contains Bump.
    // Then: System runs without error, bolt launches.
    let mut app = test_app();
    let bolt_id = spawn_serving_bolt(&mut app);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    assert!(
        app.world().get::<BoltServing>(bolt_id).is_none(),
        "bolt should be launched (BoltServing removed)"
    );

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        vel.speed() > 0.0,
        "bolt should have non-zero velocity after launch"
    );
}

// Behavior 32: launch_bolt velocity matches base_speed after velocity formula
#[test]
fn launch_bolt_velocity_matches_base_speed() {
    // Given: BaseSpeed(720.0), BoltAngleSpread(0.0), no ActiveSpeedBoosts.
    // Then: Velocity magnitude is 720.0, direction straight up.
    let mut app = test_app();
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
                .serving()
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    app.world_mut()
        .entity_mut(bolt_id)
        .insert(BoltAngleSpread(0.0));

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        (vel.speed() - 720.0).abs() < 2.0,
        "velocity magnitude should be ~720.0 (BaseSpeed), got {}",
        vel.speed()
    );
    assert!(
        vel.0.x.abs() < 1.0,
        "vx should be ~0.0 (straight up), got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 720.0).abs() < 2.0,
        "vy should be ~720.0, got {}",
        vel.0.y
    );
}

// ── Behaviors 5-6: launch_bolt skips bolts with Birthing ──

/// Helper to create a `Birthing` component for tests.
fn test_birthing() -> Birthing {
    use crate::shared::birthing::BIRTHING_DURATION;

    Birthing {
        timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
        target_scale:   Scale2D { x: 8.0, y: 8.0 },
        stashed_layers: CollisionLayers::default(),
    }
}

// Behavior 5: launch_bolt skips bolts with Birthing
#[test]
fn launch_bolt_skips_serving_bolt_with_birthing() {
    let mut app = test_app();

    let bolt_id = spawn_serving_bolt(&mut app);
    // Insert Birthing after spawning (spec says: do NOT modify spawn_serving_bolt helper)
    app.world_mut().entity_mut(bolt_id).insert(test_birthing());

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    // Bolt should STILL be serving (was NOT launched)
    assert!(
        app.world().get::<BoltServing>(bolt_id).is_some(),
        "serving bolt with Birthing should NOT be launched"
    );

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        vel.speed() < f32::EPSILON,
        "serving bolt with Birthing should still have zero velocity, got {}",
        vel.speed()
    );
}

// Behavior 5 edge case: a second serving bolt WITHOUT Birthing IS launched
#[test]
fn launch_bolt_skips_birthing_but_launches_non_birthing() {
    let mut app = test_app();

    // Bolt A: serving + Birthing — should NOT be launched
    let bolt_a = spawn_serving_bolt(&mut app);
    app.world_mut().entity_mut(bolt_a).insert(test_birthing());

    // Bolt B: serving, NO Birthing — should be launched
    let bolt_b = spawn_serving_bolt(&mut app);

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    // Bolt A: still serving
    assert!(
        app.world().get::<BoltServing>(bolt_a).is_some(),
        "birthing serving bolt should NOT be launched"
    );

    // Bolt B: launched (BoltServing removed)
    assert!(
        app.world().get::<BoltServing>(bolt_b).is_none(),
        "non-birthing serving bolt should be launched (BoltServing removed)"
    );
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    assert!(vel_b.0.y > 0.0, "non-birthing bolt should launch upward");
}

// Behavior 6: launch_bolt launches serving bolts without Birthing normally
// (covered by existing `bump_launches_serving_bolt` test — ensures filter
//  change does not break existing behavior)

#[test]
fn launch_bolt_velocity_with_speed_boost() {
    // Edge case: EffectStack<SpeedBoostConfig> with 1.5 -> speed = 720.0 * 1.5 = 1080.0
    use crate::bolt::test_utils::speed_stack;

    let mut app = test_app();
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
                .serving()
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };
    app.world_mut()
        .entity_mut(bolt_id)
        .insert((BoltAngleSpread(0.0), speed_stack(&[1.5])));

    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_id).unwrap();
    assert!(
        (vel.speed() - 1080.0).abs() < 2.0,
        "velocity magnitude should be ~1080.0 (720.0 * 1.5), got {}",
        vel.speed()
    );
}
