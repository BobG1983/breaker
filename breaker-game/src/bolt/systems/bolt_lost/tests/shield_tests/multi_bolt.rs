//! Behavior 10: Multiple bolts lost in same frame each consume one shield charge.

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
    spawn_bolt(&mut app, Vec2::new(-100.0, -309.0), Vec2::new(0.0, -400.0));
    // Bolt B
    spawn_bolt(&mut app, Vec2::new(0.0, -309.0), Vec2::new(0.0, -400.0));
    // Bolt C
    spawn_bolt(&mut app, Vec2::new(100.0, -309.0), Vec2::new(0.0, -400.0));
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
        spawn_bolt(&mut app, Vec2::new(x, -309.0), Vec2::new(0.0, -400.0));
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
