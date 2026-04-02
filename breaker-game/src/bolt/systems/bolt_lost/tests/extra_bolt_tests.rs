use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

use super::{super::system::bolt_lost, helpers::*};
use crate::{
    bolt::{
        components::{Bolt, ExtraBolt},
        messages::BoltLost,
    },
    breaker::components::Breaker,
    shared::{GameDrawLayer, GameRng, PlayfieldConfig},
};

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

    let def = make_default_bolt_definition();
    let entity = Bolt::builder()
        .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
        .extra()
        .headless()
        .spawn(app.world_mut());
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "extra bolt should be despawned when lost"
    );
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

    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
        .extra()
        .headless()
        .spawn(app.world_mut());
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
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    // Extra bolt
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::new(50.0, playfield.bottom() - 100.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
        .extra()
        .headless()
        .spawn(app.world_mut());
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

    let def = make_default_bolt_definition();
    let entity = Bolt::builder()
        .at_position(Vec2::new(50.0, playfield.bottom() - 100.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
        .extra()
        .headless()
        .spawn(app.world_mut());
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
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

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

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 1,
        "baseline bolt should still send BoltLost for game-logic purposes"
    );
}
