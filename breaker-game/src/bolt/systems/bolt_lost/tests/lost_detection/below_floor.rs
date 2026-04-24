use bevy::prelude::*;
use rantzsoft_spatial2d::components::Spatial2D;

use super::super::helpers::*;
use crate::{
    bolt::{messages::BoltLost, systems::bolt_lost::system::bolt_lost},
    prelude::*,
    shared::GameDrawLayer,
};

#[test]
fn bolt_below_floor_detected_via_position2d() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "bolt should be relaunched upward");
}

#[test]
fn bolt_above_floor_not_lost() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(&mut app, Vec2::new(0.0, 100.0), Vec2::new(100.0, -200.0));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y < 0.0, "bolt above floor should keep going down");
}

// --- NodeScalingFactor lost detection tests ---

#[test]
fn scaled_bolt_uses_effective_radius_for_lost_detection() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // Effective radius = 14.0 * 0.5 = 7.0, threshold = -300.0 - 7.0 = -307.0
    // Bolt must be below -307.0 to be detected as lost
    let bolt_y = playfield.bottom() - 7.0 - 1.0; // -308.0
    let entity = spawn_bolt(&mut app, Vec2::new(0.0, bolt_y), Vec2::new(0.0, -400.0));
    app.world_mut()
        .entity_mut(entity)
        .insert(NodeScalingFactor(0.5));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "scaled bolt below effective threshold should be respawned (vy > 0), got vy={:.1}",
        vel.0.y
    );
}

#[test]
fn bolt_without_entity_scale_in_lost_detection_is_backward_compatible() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // No NodeScalingFactor
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt without NodeScalingFactor should be respawned normally, got vy={:.1}",
        vel.0.y
    );
}

// ── BoltLost message carries entity fields ──

#[test]
fn bolt_lost_sends_correct_bolt_and_breaker_entities_for_baseline() {
    let mut app = TestAppBuilder::new()
        .with_playfield()
        .with_resource::<GameRng>()
        .with_message::<BoltLost>()
        // Required: `bolt_lost` takes `MessageWriter<KillYourself<Bolt>>` as a
        // plain SystemParam, so the message MUST be registered.
        .with_message::<KillYourself<Bolt>>()
        .with_system(FixedUpdate, (bolt_lost, capture_bolt_lost.after(bolt_lost)))
        .build();

    app.init_resource::<CapturedBoltLost>();

    let playfield = PlayfieldConfig::default();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id();

    let bolt_entity = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedBoltLost>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltLost message should be captured"
    );
    assert_eq!(
        captured.0[0].bolt, bolt_entity,
        "BoltLost.bolt should equal the bolt entity"
    );
    assert_eq!(
        captured.0[0].breaker, breaker_entity,
        "BoltLost.breaker should equal the breaker entity"
    );

    // Baseline bolt should still be alive (respawned, not despawned)
    assert!(
        app.world().get_entity(bolt_entity).is_ok(),
        "baseline bolt entity should still be alive after respawn"
    );
}

#[test]
fn bolt_lost_sends_correct_entities_when_multiple_bolts_lost_in_same_frame() {
    let mut app = TestAppBuilder::new()
        .with_playfield()
        .with_resource::<GameRng>()
        .with_message::<BoltLost>()
        // Required: `bolt_lost` takes `MessageWriter<KillYourself<Bolt>>` as a
        // plain SystemParam, so the message MUST be registered.
        .with_message::<KillYourself<Bolt>>()
        .with_system(FixedUpdate, (bolt_lost, capture_bolt_lost.after(bolt_lost)))
        .build();

    app.init_resource::<CapturedBoltLost>();

    let playfield = PlayfieldConfig::default();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id();

    let bolt_a = spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    let bolt_b = spawn_bolt(
        &mut app,
        Vec2::new(50.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedBoltLost>();
    assert_eq!(
        captured.0.len(),
        2,
        "exactly two BoltLost messages should be captured"
    );

    // Both messages should have breaker == breaker_entity
    for msg in &captured.0 {
        assert_eq!(
            msg.breaker, breaker_entity,
            "each BoltLost.breaker should equal the breaker entity"
        );
    }

    // The bolt values should be distinct and match bolt_a and bolt_b (order may vary)
    let bolt_entities: Vec<Entity> = captured.0.iter().map(|m| m.bolt).collect();
    assert!(
        bolt_entities.contains(&bolt_a),
        "BoltLost bolt entities should contain bolt_a"
    );
    assert!(
        bolt_entities.contains(&bolt_b),
        "BoltLost bolt entities should contain bolt_b"
    );
    assert_ne!(
        bolt_entities[0], bolt_entities[1],
        "BoltLost bolt entities should be distinct"
    );
}
