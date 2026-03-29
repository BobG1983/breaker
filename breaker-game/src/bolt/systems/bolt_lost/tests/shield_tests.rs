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
// Shield Redesign: Charge-per-bolt decrement behaviors (Behaviors 8-15)
// =========================================================================

/// Spawns a breaker WITH `ShieldActive` for shield protection tests.
fn spawn_shielded_breaker(app: &mut App, pos: Vec2, charges: u32) -> Entity {
    let entity = app
        .world_mut()
        .spawn((Breaker, Position2D(pos), Spatial2D, GameDrawLayer::Breaker))
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert(ShieldActive { charges });
    entity
}

// ── Behavior 8: Shield absorbs bolt-loss and decrements charges by 1 ──

#[test]
fn shield_absorbs_bolt_loss_and_decrements_charges() {
    // Given: Breaker at (100.0, -250.0) with ShieldActive { charges: 3 }.
    //        Bolt at (0.0, -309.0) with velocity (100.0, -400.0), BoltRadius(8.0).
    //        PlayfieldConfig::default() so bottom() is -300.0.
    //        Bolt Y (-309.0) < bottom() - radius (-308.0), so bolt is detected as lost.
    // When: bolt_lost runs
    // Then: Bolt velocity Y is positive (reflected upward). X sign preserved.
    //       Bolt X stays at 0.0 (not teleported to breaker X 100.0).
    //       No BoltLost message sent.
    //       Breaker ShieldActive.charges is now 2 (decremented from 3).
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(100.0, -250.0), 3);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    tick(&mut app);

    // Bolt reflected upward
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

    // Bolt should NOT have been teleported to breaker position
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

    // Charges decremented from 3 to 2
    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 2,
        "shield charges should decrement from 3 to 2 after absorbing one bolt, got {}",
        shield.charges
    );
}

#[test]
fn shield_absorbs_bolt_straight_down_and_decrements() {
    // Edge case: Bolt velocity (0.0, -400.0) straight down.
    // Y becomes positive, X stays 0.0. Charges decrement by 1.
    let mut app = test_app();
    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(100.0, -250.0), 3);

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

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 2,
        "charges should decrement from 3 to 2, got {}",
        shield.charges
    );
}

// ── Behavior 9: Shield charges decrement to 0 removes ShieldActive ──

#[test]
fn shield_charges_decrement_to_0_removes_component() {
    // Given: Breaker with ShieldActive { charges: 1 }. Bolt below floor.
    // When: bolt_lost runs
    // Then: Bolt reflected. No BoltLost. Breaker no longer has ShieldActive.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 1);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    tick(&mut app);

    // Bolt reflected
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

    // No BoltLost
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield-saved bolt should NOT send BoltLost message"
    );

    // ShieldActive removed (charges was 1, decremented to 0)
    assert!(
        app.world().get::<ShieldActive>(breaker).is_none(),
        "ShieldActive should be removed when charges reach 0"
    );
}

#[test]
fn shield_charges_0_behaves_as_no_shield() {
    // Edge case: Breaker with ShieldActive { charges: 0 } — bolt falls through.
    // This is a degenerate state but the system must handle it defensively.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<BoltLostCount>()
        .add_systems(FixedUpdate, (bolt_lost, count_bolt_lost.after(bolt_lost)));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 0);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    tick(&mut app);

    // BoltLost should be sent (no shield protection with charges: 0)
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 1,
        "breaker with charges: 0 should NOT protect bolt, expected 1 BoltLost, got {}",
        count.0
    );
}

// ── Behavior 10: Multiple bolts lost in same frame each consume one charge ──

#[test]
fn three_bolts_lost_consume_three_charges() {
    // Given: Breaker with ShieldActive { charges: 3 }. Three bolts below floor.
    // When: bolt_lost runs
    // Then: All three reflected. No BoltLost. ShieldActive removed (3 charges consumed).
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    // Bolt A
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(-100.0, -309.0)),
    ));
    // Bolt B
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -309.0)),
    ));
    // Bolt C
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(100.0, -309.0)),
    ));
    tick(&mut app);

    // All three reflected upward
    let vels: Vec<Vec2> = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .map(|v| v.0)
        .collect();
    assert_eq!(vels.len(), 3, "all three bolts should still exist");
    for vel in &vels {
        assert!(
            vel.y > 0.0,
            "all shield-saved bolts should have positive vy, got {:.1}",
            vel.y
        );
    }

    // No BoltLost
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "shield should prevent BoltLost for all 3 bolts");

    // ShieldActive removed (3 charges consumed)
    assert!(
        app.world().get::<ShieldActive>(breaker).is_none(),
        "ShieldActive should be removed after all 3 charges consumed"
    );
}

#[test]
fn four_bolts_lost_but_only_three_charges_fourth_falls_through() {
    // Edge case: Four bolts lost but only 3 charges.
    // First 3 reflected (shield saves them). 4th handled per normal bolt-lost logic.
    // One BoltLost sent for the 4th bolt.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<crate::bolt::messages::RequestBoltDestroyed>()
        .init_resource::<BoltLostCount>()
        .add_systems(FixedUpdate, (bolt_lost, count_bolt_lost.after(bolt_lost)));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    // Four bolts below floor
    for x in [-150.0, -50.0, 50.0, 150.0] {
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, -400.0)),
            bolt_lost_bundle(),
            Position2D(Vec2::new(x, -309.0)),
        ));
    }
    tick(&mut app);

    // Exactly 1 BoltLost message (for the 4th bolt)
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 1,
        "4 bolts lost with 3 charges: exactly 1 BoltLost for the unshielded bolt, got {}",
        count.0
    );

    // ShieldActive should be removed
    assert!(
        app.world().get::<ShieldActive>(breaker).is_none(),
        "ShieldActive should be removed after all charges consumed"
    );
}

// ── Behavior 11: Shield reflects velocity and clamps position ──

#[test]
fn shield_reflects_velocity_and_clamps_position() {
    // Given: Bolt at (50.0, -310.0) with velocity (200.0, -346.4) (magnitude ~400.0).
    // When: bolt_lost runs
    // Then: Velocity becomes (200.0, 346.4) — Y abs(), X preserved.
    //       Position Y clamped to bottom() + radius = -300.0 + 8.0 = -292.0.
    //       Position X preserved at 50.0.
    let mut app = test_app();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5);

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

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    let playfield = PlayfieldConfig::default();
    let bolt_config = BoltConfig::default();
    let expected_y = playfield.bottom() + bolt_config.radius;
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "shield-saved bolt X should be preserved at 50.0, got {:.1}",
        pos.0.x
    );
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "shield-saved bolt Y should be clamped to bottom() + radius ({expected_y:.1}), got {:.1}",
        pos.0.y
    );
}

#[test]
fn shield_reflects_zero_velocity_unchanged() {
    // Edge case: Velocity (0.0, 0.0) — remains (0.0, 0.0) after reflection.
    // This is a degenerate case. The bolt is still below floor so it will be
    // detected as lost, and the shield reflect produces (0.0, 0.0.abs()) = (0.0, 0.0).
    // The test just verifies no panic and the velocity is "reflected" (trivially).
    let mut app = test_app();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, 0.0)),
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
        "zero velocity reflect vx should be 0.0, got {:.3}",
        vel.0.x
    );
    assert!(
        (vel.0.y).abs() < f32::EPSILON,
        "zero velocity reflect vy should be 0.0, got {:.3}",
        vel.0.y
    );
}

// ── Behavior 12: Shield protects ExtraBolt equally ──

#[test]
fn shield_protects_extra_bolt_consuming_one_charge() {
    // Given: Breaker with ShieldActive { charges: 2 }. One baseline bolt and one ExtraBolt
    //        both below floor.
    // When: bolt_lost runs
    // Then: Both reflected upward. No BoltLost. No RequestBoltDestroyed. charges → 0, removed.
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

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 2);

    // Baseline bolt
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(-50.0, -309.0)),
    ));
    // Extra bolt
    app.world_mut().spawn((
        Bolt,
        ExtraBolt,
        Velocity2D(Vec2::new(-100.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(50.0, -309.0)),
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
    assert_eq!(count.0, 0, "no BoltLost messages when shield protects");
    let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
    assert!(
        captured.0.is_empty(),
        "no RequestBoltDestroyed when shield protects"
    );

    // ShieldActive removed (2 charges consumed, one per bolt)
    assert!(
        app.world().get::<ShieldActive>(breaker).is_none(),
        "ShieldActive should be removed after all charges consumed"
    );
}

#[test]
fn shield_protects_only_extra_bolt_below_floor() {
    // Edge case: Only ExtraBolt below floor (baseline above). Shield absorbs it, charges 2→1.
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

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 2);

    // ExtraBolt below floor
    app.world_mut().spawn((
        Bolt,
        ExtraBolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(50.0, -309.0)),
    ));
    // Baseline bolt above floor
    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(100.0, -200.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, 100.0)),
    ));
    tick(&mut app);

    // No messages
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 0,
        "shield should prevent BoltLost for the extra bolt"
    );
    let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shield should prevent RequestBoltDestroyed for the extra bolt"
    );

    // Charges decremented from 2 to 1
    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 1,
        "shield charges should decrement from 2 to 1, got {}",
        shield.charges
    );
}

// ── Behavior 13: Bolt above floor does not consume shield charges ──

#[test]
fn bolt_above_floor_does_not_consume_charges() {
    // Given: Breaker with ShieldActive { charges: 3 }. Bolt at (0.0, 100.0) above floor.
    // When: bolt_lost runs
    // Then: Bolt velocity unchanged. Bolt position unchanged. charges remain 3.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

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
        (pos.0.y - 100.0).abs() < f32::EPSILON,
        "bolt above floor should keep y=100.0, got {:.1}",
        pos.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "bolt above floor should not send BoltLost");

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 3,
        "charges should remain 3 when no bolt is lost, got {}",
        shield.charges
    );
}

// ── Behavior 14: Bolt at exactly bottom()-radius is NOT lost (boundary) ──

#[test]
fn bolt_at_exactly_threshold_is_not_lost() {
    // Given: Bolt at (0.0, -308.0) which is exactly bottom() - radius = -300.0 - 8.0 = -308.0.
    //        Condition is strict `<`, so -308.0 is NOT below threshold.
    // Then: Bolt NOT considered lost, velocity unchanged, charges remain 3.
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -308.0)),
    ));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.y - (-400.0)).abs() < f32::EPSILON,
        "bolt at exact threshold should keep vy=-400.0, got {:.1}",
        vel.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "bolt at exact threshold should NOT be lost");

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 3,
        "charges should remain 3 (bolt was not lost), got {}",
        shield.charges
    );
}

#[test]
fn bolt_barely_below_threshold_is_absorbed_by_shield() {
    // Edge case: Bolt at (0.0, -308.001) — IS below threshold, shield absorbs, charges 3→2.
    let mut app = test_app();
    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 3);

    app.world_mut().spawn((
        Bolt,
        Velocity2D(Vec2::new(0.0, -400.0)),
        bolt_lost_bundle(),
        Position2D(Vec2::new(0.0, -308.001)),
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
        "bolt barely below threshold should be reflected, got vy={:.1}",
        vel.0.y
    );

    let shield = app.world().get::<ShieldActive>(breaker).unwrap();
    assert_eq!(
        shield.charges, 2,
        "charges should decrement from 3 to 2, got {}",
        shield.charges
    );
}

// ── Behavior 15: Shield with barely-below-floor bolt still absorbs and decrements ──

#[test]
fn shield_absorbs_barely_below_floor_bolt_and_removes_on_last_charge() {
    // Given: Breaker with ShieldActive { charges: 1 }. Bolt at (0.0, -308.5).
    //        Floor threshold = -308.0. Bolt Y below threshold.
    // Then: Bolt reflected. ShieldActive removed (charges was 1, now 0).
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 1);

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
        "shield should reflect bolt barely below threshold, got vy={:.1}",
        vel.0.y
    );

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 0, "shield should prevent BoltLost");

    assert!(
        app.world().get::<ShieldActive>(breaker).is_none(),
        "ShieldActive should be removed when charges reach 0"
    );
}
