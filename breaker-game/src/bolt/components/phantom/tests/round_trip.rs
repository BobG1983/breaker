//! Round-trip tests exercising `become_phantom` then `become_normal`.

use bevy::prelude::*;

use super::{super::inner::*, helpers::test_app};
use crate::{
    bolt::components::{Bolt, PrimaryBolt},
    prelude::*,
};

// ── Behavior 14: round-trip become_phantom → become_normal ───

#[test]
fn round_trip_become_phantom_then_become_normal_leaves_clean_bolt() {
    let mut app = test_app();
    let e = app
        .world_mut()
        .spawn((
            Bolt,
            PrimaryBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    // Phase 0 — become_phantom
    app.world_mut().insert_resource(PhaseGate(0_u32));
    app.add_systems(
        Update,
        move |mut commands: Commands, phase: Res<PhaseGate>| {
            if phase.0 == 0 {
                Bolt::become_phantom(&mut commands, e, PhantomDedupKey::Bolt(e));
            } else if phase.0 == 1 {
                PhantomBolt::become_normal(&mut commands, e);
            }
        },
    );
    app.update();

    // Verify intermediate phantom state
    assert!(
        app.world().get::<PhantomBolt>(e).is_some(),
        "PhantomBolt should be present after phase 0"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(e).is_some(),
        "PhantomDedupKey should be present after phase 0"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(e).is_some(),
        "PhantomDamagedCells should be present after phase 0"
    );

    // Advance to phase 1 — become_normal
    app.world_mut().resource_mut::<PhaseGate>().0 = 1;
    app.update();

    // All phantom components gone, normal bolt components intact
    assert!(
        app.world().get::<PhantomBolt>(e).is_none(),
        "PhantomBolt should be gone after phase 1"
    );
    assert!(
        app.world().get::<PhantomDedupKey>(e).is_none(),
        "PhantomDedupKey should be gone after phase 1"
    );
    assert!(
        app.world().get::<PhantomDamagedCells>(e).is_none(),
        "PhantomDamagedCells should be gone after phase 1"
    );
    assert!(
        app.world().get::<Bolt>(e).is_some(),
        "Bolt should still be present after round-trip"
    );
    assert!(
        app.world().get::<PrimaryBolt>(e).is_some(),
        "PrimaryBolt should still be present after round-trip"
    );
    let vel = app
        .world()
        .get::<Velocity2D>(e)
        .expect("Velocity2D should still be present after round-trip");
    assert_eq!(
        vel.0,
        Vec2::new(0.0, 400.0),
        "Velocity2D should be unchanged after round-trip"
    );
    let pos = app
        .world()
        .get::<Position2D>(e)
        .expect("Position2D should still be present after round-trip");
    assert_eq!(
        pos.0,
        Vec2::new(10.0, 20.0),
        "Position2D should be unchanged after round-trip"
    );
}

/// Phase gate resource for round-trip test — drives which closure fires.
#[derive(Resource)]
struct PhaseGate(u32);
