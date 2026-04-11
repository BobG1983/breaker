use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use super::{super::effect::*, helpers::*};

#[test]
fn tick_gravity_well_despawns_expired_wells() {
    let mut app = test_app();
    enter_playing(&mut app);

    let well = app
        .world_mut()
        .spawn((
            GravityWell,
            GravityWellConfig {
                strength: 100.0,
                radius: 80.0,
                remaining: 0.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.update();

    assert!(
        app.world().get_entity(well).is_err(),
        "expired gravity well should be despawned"
    );
}
