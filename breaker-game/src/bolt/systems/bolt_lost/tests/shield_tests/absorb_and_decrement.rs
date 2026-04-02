//! Behaviors 8-9: Shield absorbs `bolt`-loss and decrements charges.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    super::{super::system::bolt_lost, helpers::*},
    helpers::spawn_shielded_breaker,
};
use crate::{
    bolt::{components::Bolt, messages::BoltLost},
    effect::effects::shield::ShieldActive,
    shared::{GameRng, PlayfieldConfig},
};

// ── Behavior 8: Shield absorbs bolt-loss and decrements charges by 1 ──

#[test]
fn shield_absorbs_bolt_loss_and_decrements_charges() {
    // Given: Breaker at (100.0, -250.0) with ShieldActive { charges: 3 }.
    //        Bolt at (0.0, -315.0) with velocity (100.0, -400.0), BaseRadius(14.0).
    //        PlayfieldConfig::default() so bottom() is -300.0.
    //        Bolt Y (-315.0) < bottom() - radius (-314.0), so bolt is detected as lost.
    // When: bolt_lost runs
    // Then: Bolt velocity Y is positive (reflected upward). X sign preserved.
    //       Bolt X stays at 0.0 (not teleported to breaker X 100.0).
    //       No BoltLost message sent.
    //       Breaker ShieldActive.charges is now 2 (decremented from 3).
    let mut app = test_app();
    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let breaker = spawn_shielded_breaker(&mut app, Vec2::new(100.0, -250.0), 3);

    spawn_bolt(&mut app, Vec2::new(0.0, -315.0), Vec2::new(100.0, -400.0));
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

    let def = crate::bolt::definition::BoltDefinition {
        min_angle_horizontal: 0.0,
        min_angle_vertical: 0.0,
        ..make_default_bolt_definition()
    };
    spawn_bolt_with_definition(
        &mut app,
        Vec2::new(0.0, -315.0),
        Vec2::new(0.0, -400.0),
        &def,
    );
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

    spawn_bolt(&mut app, Vec2::new(0.0, -315.0), Vec2::new(0.0, -400.0));
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

    let _breaker = spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 0);

    spawn_bolt(&mut app, Vec2::new(0.0, -315.0), Vec2::new(0.0, -400.0));
    tick(&mut app);

    // BoltLost should be sent (no shield protection with charges: 0)
    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 1,
        "breaker with charges: 0 should NOT protect bolt, expected 1 BoltLost, got {}",
        count.0
    );
}
