use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

use super::{super::system::bolt_lost, helpers::*};
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        messages::BoltLost,
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    effect::effects::shield::ShieldActive,
    shared::{GameDrawLayer, GameRng, PlayfieldConfig},
};

// =========================================================================
// Wave 4B: Shield Protection — bolt_lost shield-save behaviors
// =========================================================================

/// Spawns a breaker WITH `ShieldActive` for shield protection tests.
fn spawn_shielded_breaker(app: &mut App, pos: Vec2, remaining: f32) -> Entity {
    let entity = app
        .world_mut()
        .spawn((Breaker, Position2D(pos), Spatial2D, GameDrawLayer::Breaker))
        .id();
    app.world_mut().entity_mut(entity).insert(ShieldActive {
        remaining,
        owner: entity,
    });
    entity
}

#[test]
fn shield_reflects_bolt_below_floor_upward() {
    // Behavior 1: Bolt below floor bounces up when breaker has ShieldActive.
    // Given: Breaker at (100.0, -250.0) with ShieldActive { remaining: 5.0 }.
    //        Bolt at (0.0, -309.0) with velocity (100.0, -400.0), BoltRadius(8.0).
    //        PlayfieldConfig::default() so bottom() is -300.0.
    //        Bolt Y (-309.0) < bottom() - radius (-308.0), so bolt is detected as lost.
    // When: bolt_lost runs
    // Then: Bolt velocity Y is positive (reflected upward). X sign is preserved.
    //       Bolt is NOT respawned (no position reset to breaker).
    //       No BoltLost message is sent.
    // Note: Breaker X (100.0) differs from bolt X (0.0) so the position assertion
    //       can distinguish "shield preserved bolt X" from "respawn teleported to breaker X".
    let mut app = test_app();
    let _playfield = PlayfieldConfig::default();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    spawn_shielded_breaker(&mut app, Vec2::new(100.0, -250.0), 5.0);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "shield should reflect bolt upward, got vy={:.1}",
        vel.0.y
    );
    assert!(
        vel.0.x > 0.0,
        "shield reflect should preserve X sign, got vx={:.1}",
        vel.0.x
    );

    // Bolt should NOT have been teleported to breaker position (breaker X is 100.0)
    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "shield-saved bolt X should stay at original X (0.0), not breaker X (100.0), got {:.1}",
        pos.0.x
    );

    // No BoltLost message
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield-saved bolt should NOT send BoltLost message"
    );
}

#[test]
fn shield_reflects_bolt_straight_down() {
    // Behavior 1 edge case: Bolt velocity (0.0, -400.0) straight down.
    // Y becomes positive (0.0, 400.0), X stays 0.0.
    // Breaker X (100.0) differs from bolt X (0.0) to distinguish shield-save from respawn.
    let mut app = test_app();
    spawn_shielded_breaker(&mut app, Vec2::new(100.0, -250.0), 5.0);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.x).abs() < f32::EPSILON,
        "straight-down shield reflect should have vx=0.0, got {:.3}",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "straight-down shield reflect should have positive vy, got {:.1}",
        vel.0.y
    );

    // Bolt X should remain at 0.0, NOT teleported to breaker X (100.0)
    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "shield-saved bolt X should stay at original X (0.0), not breaker X (100.0), got {:.1}",
        pos.0.x
    );
}

#[test]
fn shield_active_bolt_above_floor_unaffected() {
    // Behavior 5: Bolt above floor with ShieldActive breaker is not affected.
    // Given: Breaker at (0.0, -250.0) with ShieldActive { remaining: 5.0 }.
    //        Bolt at (0.0, 100.0) with velocity (100.0, -200.0), ABOVE floor threshold.
    // When: bolt_lost runs
    // Then: Bolt velocity unchanged (100.0, -200.0). Bolt position unchanged (0.0, 100.0).
    //       No BoltLost message. Shield logic path is NOT entered for bolts above the floor.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5.0);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -200.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.x - 100.0).abs() < f32::EPSILON,
        "bolt above floor should keep vx=100.0, got {:.1}",
        vel.0.x
    );
    assert!(
        (vel.0.y - (-200.0)).abs() < f32::EPSILON,
        "bolt above floor should keep vy=-200.0, got {:.1}",
        vel.0.y
    );

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "bolt above floor should keep x=0.0, got {:.1}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 100.0).abs() < f32::EPSILON,
        "bolt above floor should keep y=100.0, got {:.1}",
        pos.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "bolt above floor should NOT send BoltLost message"
    );
}

#[test]
fn shield_save_preserves_velocity_magnitude() {
    // Behavior 2: Shield bolt-save reflects Y velocity while preserving magnitude.
    // Given: Bolt at (50.0, -310.0) with velocity (200.0, -346.4) (magnitude 400.0).
    // When: bolt_lost runs
    // Then: Velocity becomes (200.0, 346.4) — Y negated, X preserved. Magnitude 400.0.
    let mut app = test_app();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 10.0);

    let original_vel = Vec2::new(200.0, -346.4);
    let original_magnitude = original_vel.length();

    app.world_mut().spawn((
        Bolt,
        Velocity2D(original_vel),
        bolt_lost_bundle(),
        Position2D(Vec2::new(50.0, -310.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.x - 200.0).abs() < 1.0,
        "shield reflect should preserve X component (200.0), got {:.1}",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "shield reflect should make Y positive, got {:.1}",
        vel.0.y
    );
    let new_magnitude = vel.0.length();
    assert!(
        (new_magnitude - original_magnitude).abs() < 1.0,
        "shield reflect should preserve magnitude ({original_magnitude:.1}), got {new_magnitude:.1}"
    );
}

#[test]
fn shield_save_clamps_bolt_y_above_floor() {
    // Behavior 2: Bolt Y position is clamped to playfield.bottom() + radius = -300.0 + 8.0 = -292.0.
    // Bolt X position is preserved (50.0).
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    let bolt_config = BoltConfig::default();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 10.0);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(200.0, -346.4)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(50.0, -310.0)),
    ));
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    let expected_y = playfield.bottom() + bolt_config.radius;
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "shield-saved bolt X should be preserved at 50.0, got {:.1}",
        pos.0.x
    );
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "shield-saved bolt Y should be clamped to bottom() + radius ({:.1}), got {:.1}",
        expected_y,
        pos.0.y
    );
}

#[test]
fn shield_save_does_not_send_bolt_lost_message() {
    // Behavior 3: Shield bolt-save does not send BoltLost message.
    // Given: Breaker with ShieldActive. Bolt below floor.
    // When: bolt_lost runs
    // Then: No BoltLost message is written.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5.0);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield-saved bolt should NOT send BoltLost message"
    );
}

#[test]
fn shield_save_multiple_bolts_none_send_bolt_lost() {
    // Behavior 3 edge case: Multiple bolts below floor, breaker has ShieldActive.
    // None of them send BoltLost.
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5.0);

    // Two bolts below floor
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(50.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(-100.0, playfield.bottom() - 50.0)),
    ));
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(-50.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(100.0, playfield.bottom() - 50.0)),
    ));
    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield should prevent BoltLost for ALL bolts, got {} messages",
        count.0
    );
}

#[test]
fn shield_protects_extra_bolt_equally() {
    // Behavior 4: Shield protects ExtraBolts too (all bolts saved equally).
    // Given: ExtraBolt at (0.0, -310.0) with velocity (0.0, -400.0) below floor.
    // When: bolt_lost runs
    // Then: ExtraBolt velocity Y is reflected to positive (0.0, 400.0).
    //       Y position clamped to bottom() + radius = -292.0.
    //       No RequestBoltDestroyed. No BoltLost.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<BoltLostCount>()
        .init_resource::<CapturedRequestBoltDestroyed>()
        .add_systems(
            FixedUpdate,
            (
                bolt_lost,
                count_bolt_lost.after(bolt_lost),
                capture_request_bolt_destroyed.after(bolt_lost),
            ),
        );

    let playfield = PlayfieldConfig::default();
    let bolt_config = BoltConfig::default();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5.0);

    let extra = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, -310.0)),
        ))
        .id();
    tick(&mut app);

    // Entity should still exist (not despawned or destroyed)
    assert!(
        app.world().get_entity(extra).is_ok(),
        "shield-saved extra bolt should still exist"
    );

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "shield should reflect extra bolt upward, got vy={:.1}",
        vel.0.y
    );

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .unwrap();
    let expected_y = playfield.bottom() + bolt_config.radius;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "shield-saved extra bolt Y should be clamped to {expected_y:.1}, got {:.1}",
        pos.0.y
    );

    // No messages
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield-saved extra bolt should NOT send BoltLost"
    );

    let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shield-saved extra bolt should NOT send RequestBoltDestroyed"
    );
}

#[test]
fn shield_protects_both_baseline_and_extra_bolt() {
    // Behavior 4 edge case: Both a baseline bolt AND an ExtraBolt below floor with
    // ShieldActive — both are reflected, neither sends BoltLost or RequestBoltDestroyed.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<BoltLostCount>()
        .init_resource::<CapturedRequestBoltDestroyed>()
        .add_systems(
            FixedUpdate,
            (
                bolt_lost,
                count_bolt_lost.after(bolt_lost),
                capture_request_bolt_destroyed.after(bolt_lost),
            ),
        );

    let playfield = PlayfieldConfig::default();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5.0);

    // Baseline bolt
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(-50.0, playfield.bottom() - 50.0)),
    ));
    // Extra bolt
    app.world_mut().spawn((
        Bolt,
        ExtraBolt,
        Velocity2D(Vec2::new(-100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(50.0, playfield.bottom() - 50.0)),
    ));
    tick(&mut app);

    // Both bolts should still exist
    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolt_count, 2,
        "shield should save both bolts, got {bolt_count}"
    );

    // Both reflected upward
    let vels: Vec<Vec2> = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .map(|v| v.0)
        .collect();
    for vel in &vels {
        assert!(
            vel.y > 0.0,
            "all shield-saved bolts should have positive vy, got {:.1}",
            vel.y
        );
    }

    // No messages
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "no BoltLost messages when shield is active");

    let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
    assert!(
        captured.0.is_empty(),
        "no RequestBoltDestroyed when shield is active"
    );
}

#[test]
fn shield_protects_bolt_barely_below_floor_threshold() {
    // Behavior 7: Shield protects bolt at exact floor threshold.
    // Given: Breaker with ShieldActive { remaining: 0.01 }.
    //        Bolt at (0.0, -308.5) with velocity (0.0, -400.0). BoltRadius(8.0).
    //        Floor threshold: bottom() - radius = -300.0 - 8.0 = -308.0.
    //        Bolt Y (-308.5) is barely below the threshold.
    // When: bolt_lost runs
    // Then: Bolt velocity Y is reflected to positive. Shield saves the bolt.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 0.01);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -308.5)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "shield should reflect bolt barely below threshold upward, got vy={:.1}",
        vel.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield-saved bolt barely below threshold should NOT send BoltLost"
    );
}
