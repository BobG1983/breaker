use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Spatial2D, Velocity2D};

use super::*;
use crate::{
    bolt::{
        components::{
            Bolt, BoltBaseSpeed, BoltRadius, BoltRespawnAngleSpread, BoltRespawnOffsetY, ExtraBolt,
        },
        messages::BoltLost,
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    effect::effects::shield::ShieldActive,
    shared::{EntityScale, GameDrawLayer, GameRng, PlayfieldConfig},
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_systems(FixedUpdate, bolt_lost);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn bolt_lost_bundle() -> (
    BoltBaseSpeed,
    BoltRadius,
    BoltRespawnOffsetY,
    BoltRespawnAngleSpread,
) {
    let config = BoltConfig::default();
    (
        BoltBaseSpeed(config.base_speed),
        BoltRadius(config.radius),
        BoltRespawnOffsetY(config.respawn_offset_y),
        BoltRespawnAngleSpread(config.respawn_angle_spread),
    )
}

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

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
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
fn respawn_inserts_position2d_at_breaker_x() {
    let mut app = test_app();
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(200.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let (vel, pos) = app
        .world_mut()
        .query::<(&Velocity2D, &Position2D)>()
        .iter(app.world())
        .next()
        .unwrap();

    let speed = vel.0.length();
    assert!(
        (speed - bolt_config.base_speed).abs() < 1.0,
        "respawn speed should equal base_speed {:.0}, got {:.1}",
        bolt_config.base_speed,
        speed,
    );

    let angle = vel.0.x.atan2(vel.0.y).abs();
    assert!(
        angle <= bolt_config.respawn_angle_spread + f32::EPSILON,
        "respawn angle {angle:.3} rad should be within spread {:.3} rad",
        bolt_config.respawn_angle_spread,
    );

    assert!(vel.0.y > 0.0, "respawn should launch upward");

    assert!(
        (pos.0.x - breaker_x).abs() < f32::EPSILON,
        "respawn Position2D.0.x should match breaker X {breaker_x:.0}, got {:.1}",
        pos.0.x,
    );
}

#[test]
fn respawn_with_zero_spread_launches_straight_up() {
    let mut app = test_app();
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        (
            BoltBaseSpeed(bolt_config.base_speed),
            BoltRadius(bolt_config.radius),
            BoltRespawnOffsetY(bolt_config.respawn_offset_y),
            BoltRespawnAngleSpread(0.0),
        ),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();

    assert!(
        vel.0.x.abs() < f32::EPSILON,
        "zero spread should launch straight up, got vx={:.3}",
        vel.0.x,
    );
}

#[test]
fn respawn_position2d_y_uses_respawn_offset() {
    let mut app = test_app();
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, breaker_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected_y = breaker_y + bolt_config.respawn_offset_y;
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "respawn Position2D.0.y should be breaker_y + respawn_offset_y ({expected_y}), got {}",
        pos.0.y,
    );
}

#[test]
fn respawn_inserts_previous_position_matching_position2d() {
    let mut app = test_app();
    let bolt_config = BoltConfig::default();
    let playfield = PlayfieldConfig::default();
    let breaker_x = 42.0;
    let breaker_y = -250.0;
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(breaker_x, breaker_y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let (pos, prev_pos) = app
        .world_mut()
        .query_filtered::<(&Position2D, &PreviousPosition), With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();

    let expected = Vec2::new(breaker_x, breaker_y + bolt_config.respawn_offset_y);
    assert!(
        (pos.0 - expected).length() < f32::EPSILON,
        "respawn Position2D should be ({expected:?}), got {:?}",
        pos.0,
    );
    assert!(
        (prev_pos.0 - expected).length() < f32::EPSILON,
        "respawn PreviousPosition should match Position2D ({expected:?}), got {:?}",
        prev_pos.0,
    );
}

#[test]
fn extra_bolt_below_floor_is_despawned() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
        ))
        .id();
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "extra bolt should be despawned when lost"
    );
}

#[derive(Resource, Default)]
struct BoltLostCount(u32);

fn count_bolt_lost(mut reader: MessageReader<BoltLost>, mut count: ResMut<BoltLostCount>) {
    for _msg in reader.read() {
        count.0 += 1;
    }
}

#[test]
fn extra_bolt_sends_bolt_lost_on_despawn() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    app.world_mut().spawn((
        Bolt,
        ExtraBolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 1, "BoltLost message should be sent for extra bolt");
}

#[test]
fn baseline_bolt_still_respawns_with_extra_present() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // Baseline bolt (no ExtraBolt)
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    // Extra bolt
    app.world_mut().spawn((
        Bolt,
        ExtraBolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(50.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    // Baseline bolt should still exist (respawned)
    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(bolt_count, 1, "only baseline bolt should remain");

    // Verify it's the baseline (no ExtraBolt)
    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(extra_count, 0, "extra bolt should be gone");
}

// =========================================================================
// C7 Wave 2a: Two-Phase Destruction — bolt_lost writes
// RequestBoltDestroyed for ExtraBolt only (behaviors 33, 33a)
// =========================================================================

#[derive(Resource, Default)]
struct CapturedRequestBoltDestroyed(Vec<crate::bolt::messages::RequestBoltDestroyed>);

fn capture_request_bolt_destroyed(
    mut reader: MessageReader<crate::bolt::messages::RequestBoltDestroyed>,
    mut captured: ResMut<CapturedRequestBoltDestroyed>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

#[test]
fn extra_bolt_writes_request_bolt_destroyed_instead_of_despawning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<CapturedRequestBoltDestroyed>()
        .add_systems(
            FixedUpdate,
            (bolt_lost, capture_request_bolt_destroyed).chain(),
        );

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(50.0, playfield.bottom() - 100.0)),
        ))
        .id();
    tick(&mut app);

    let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "extra bolt should write RequestBoltDestroyed"
    );
    assert_eq!(
        captured.0[0].bolt, entity,
        "RequestBoltDestroyed should carry the bolt entity"
    );

    // Entity should STILL BE ALIVE (two-phase flow — no immediate despawn)
    assert!(
        app.world().get_entity(entity).is_ok(),
        "extra bolt entity should still be alive — bridge evaluates before cleanup despawns"
    );
}

#[test]
fn baseline_bolt_does_not_write_request_bolt_destroyed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<CapturedRequestBoltDestroyed>()
        .add_systems(
            FixedUpdate,
            (bolt_lost, capture_request_bolt_destroyed).chain(),
        );

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // Baseline bolt (no ExtraBolt marker)
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));

    tick(&mut app);

    let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
    assert!(
        captured.0.is_empty(),
        "baseline bolt should NOT write RequestBoltDestroyed — it gets respawned"
    );
}

#[test]
fn baseline_bolt_still_sends_bolt_lost_message() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<BoltLostCount>()
        .add_systems(FixedUpdate, (bolt_lost, count_bolt_lost).chain());

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));

    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 1,
        "baseline bolt should still send BoltLost for game-logic purposes"
    );
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

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -200.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y < 0.0, "bolt above floor should keep going down");
}

// --- EntityScale lost detection tests ---

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

    let bolt_y = playfield.bottom() - 4.0 - 1.0; // -305.0
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        EntityScale(0.5),
        Position2D(Vec2::new(0.0, bolt_y)),
    ));
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

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        // No EntityScale
        Position2D(Vec2::new(0.0, playfield.bottom() - 100.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt without EntityScale should be respawned normally, got vy={:.1}",
        vel.0.y
    );
}

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
