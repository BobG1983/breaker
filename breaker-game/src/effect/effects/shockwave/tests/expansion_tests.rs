use bevy::prelude::*;

use super::*;

// -- system tests ────────────────────────────────────────────────

#[test]
fn tick_shockwave_expands_radius_by_speed_times_dt() {
    let mut app = test_app();
    enter_playing(&mut app);

    let shockwave = app
        .world_mut()
        .spawn((
            ShockwaveRadius(0.0),
            ShockwaveMaxRadius(100.0),
            ShockwaveSpeed(50.0),
        ))
        .id();

    app.update();

    let radius = app.world().get::<ShockwaveRadius>(shockwave).unwrap();
    // After one update tick, radius should have increased by speed * dt.
    // dt is not zero since MinimalPlugins provides Time.
    assert!(
        radius.0 > 0.0,
        "shockwave radius should expand after tick, got {}",
        radius.0
    );
}

#[test]
fn despawn_finished_shockwave_removes_entity_when_radius_ge_max() {
    let mut app = test_app();
    enter_playing(&mut app);

    let shockwave = app
        .world_mut()
        .spawn((
            ShockwaveRadius(100.0),
            ShockwaveMaxRadius(100.0),
            ShockwaveSpeed(50.0),
        ))
        .id();

    app.update();

    // Entity should be despawned because radius >= max_radius
    assert!(
        app.world().get_entity(shockwave).is_err(),
        "shockwave entity should be despawned when radius >= max_radius"
    );
}

#[test]
fn reverse_is_noop_shockwave_entity_remains() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    // Verify shockwave exists before reverse
    let mut query = world.query::<&ShockwaveSource>();
    assert_eq!(query.iter(&world).count(), 1);

    reverse(entity, "", &mut world);

    // Shockwave entity should still exist after reverse (no-op)
    assert_eq!(query.iter(&world).count(), 1);
}
