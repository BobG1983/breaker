//! Behavior 12: Shield protects `ExtraBolt` equally.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::{super::system::bolt_lost, helpers::*},
    helpers::spawn_shielded_breaker,
};
use crate::{
    bolt::{components::Bolt, messages::BoltLost},
    effect::effects::shield::ShieldActive,
    shared::{GameRng, PlayfieldConfig},
};

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
    spawn_bolt(&mut app, Vec2::new(-50.0, -315.0), Vec2::new(100.0, -400.0));
    // Extra bolt
    let def = make_default_bolt_definition();
    let extra = Bolt::builder()
        .at_position(Vec2::new(50.0, -315.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(-100.0, -400.0)))
        .extra()
        .headless()
        .spawn(app.world_mut());
    let _ = extra;
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
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::new(50.0, -315.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
        .extra()
        .headless()
        .spawn(app.world_mut());
    // Baseline bolt above floor
    spawn_bolt(&mut app, Vec2::new(0.0, 100.0), Vec2::new(100.0, -200.0));
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
