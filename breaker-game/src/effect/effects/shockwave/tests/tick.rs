use bevy::prelude::*;

use super::helpers::tick;
use crate::effect::effects::shockwave::system::*;

// =========================================================================
// Part B: tick_shockwave
// =========================================================================

/// Behavior 5: `tick_shockwave` expands radius by speed * dt.
#[test]
fn tick_shockwave_expands_radius_by_speed_times_dt() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, tick_shockwave);

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 0.0,
                max: 96.0,
            },
            ShockwaveSpeed(400.0),
        ))
        .id();

    tick(&mut app);

    let radius = app
        .world()
        .get::<ShockwaveRadius>(entity)
        .expect("entity should still exist");
    // dt = 1/64 = 0.015625, expansion = 400.0 * 0.015625 = 6.25
    let expected = 400.0 / 64.0;
    assert!(
        (radius.current - expected).abs() < 0.1,
        "after one tick, radius.current should be ~{expected:.2}, got {:.2}",
        radius.current
    );
}

/// Behavior 6: `tick_shockwave` despawns when current >= max.
#[test]
fn tick_shockwave_despawns_when_fully_expanded() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, tick_shockwave);

    let entity = app
        .world_mut()
        .spawn((
            ShockwaveRadius {
                current: 95.0,
                max: 96.0,
            },
            ShockwaveSpeed(400.0),
        ))
        .id();

    // One tick: 95.0 + 6.25 = 101.25 >= 96.0 -> despawn
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "entity should be despawned when current >= max"
    );
}
