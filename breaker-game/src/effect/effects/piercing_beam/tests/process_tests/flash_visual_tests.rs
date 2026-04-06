//! Tests for piercing beam flash visual entity spawning.

use rantzsoft_spatial2d::components::{Position2D, Rotation2D, Scale2D};

use crate::{
    effect::effects::piercing_beam::tests::helpers::*, fx::EffectFlashTimer, shared::GameDrawLayer,
};

// ── Test app with asset support ────────────────────────────────────

fn flash_test_app() -> App {
    let mut app = piercing_beam_damage_test_app();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<ColorMaterial>>();
    app
}

// ── Behavior 6: process_piercing_beam spawns flash visual entity ──

#[test]
fn process_piercing_beam_spawns_flash_visual_entity_with_required_components() {
    let mut app = flash_test_app();

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    // Request should be despawned
    assert!(
        app.world().get_entity(request).is_err(),
        "PiercingBeamRequest should be despawned after processing"
    );

    // Query for flash entities by EffectFlashTimer
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(
        flash_entities.len(),
        1,
        "expected exactly 1 flash entity, got {}",
        flash_entities.len()
    );

    let flash = flash_entities[0];

    // Check all required components
    assert!(
        app.world().get::<Mesh2d>(flash).is_some(),
        "flash entity should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(flash)
            .is_some(),
        "flash entity should have MeshMaterial2d<ColorMaterial>"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(flash).is_some(),
        "flash entity should have CleanupOnExit<NodeState>"
    );
    assert!(
        matches!(
            app.world().get::<GameDrawLayer>(flash),
            Some(GameDrawLayer::Fx)
        ),
        "flash entity should have GameDrawLayer::Fx"
    );
}

// ── Behavior 7: Flash entity has EffectFlashTimer(0.15) ──

#[test]
fn piercing_beam_flash_has_timer_value_0_15() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app.world_mut().query::<&EffectFlashTimer>();
    let timer = query
        .iter(app.world())
        .next()
        .expect("flash entity should exist");
    assert!(
        (timer.0 - 0.15).abs() < f32::EPSILON,
        "EffectFlashTimer should be 0.15, got {}",
        timer.0
    );
}

// ── Behavior 8: Flash positioned at beam midpoint ──

#[test]
fn piercing_beam_flash_positioned_at_beam_midpoint_vertical() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Position2D, With<EffectFlashTimer>>();
    let pos = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Position2D");
    // Midpoint: origin + direction * length/2 = (0,0) + (0,1)*150 = (0, 150)
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "flash x should be 0.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 150.0).abs() < 0.1,
        "flash y should be 150.0 (beam midpoint), got {}",
        pos.0.y
    );
}

#[test]
fn piercing_beam_flash_positioned_at_beam_midpoint_diagonal() {
    let mut app = flash_test_app();

    let dir = Vec2::new(1.0, 1.0).normalize();
    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(50.0, 50.0),
        direction: dir,
        length: 200.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Position2D, With<EffectFlashTimer>>();
    let pos = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Position2D");
    // Midpoint: (50, 50) + (0.707, 0.707) * 100 ~= (120.7, 120.7)
    let expected = Vec2::new(50.0, 50.0) + dir * 100.0;
    assert!(
        (pos.0.x - expected.x).abs() < 1.0,
        "flash x should be ~{}, got {}",
        expected.x,
        pos.0.x
    );
    assert!(
        (pos.0.y - expected.y).abs() < 1.0,
        "flash y should be ~{}, got {}",
        expected.y,
        pos.0.y
    );
}

// ── Behavior 9: Flash scaled to beam dimensions ──

#[test]
fn piercing_beam_flash_scale_matches_beam_dimensions() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Scale2D, With<EffectFlashTimer>>();
    let scale = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Scale2D");
    // Scale2D { x: length, y: 2*half_width } = { x: 300.0, y: 20.0 }
    assert!(
        (scale.x - 300.0).abs() < f32::EPSILON,
        "Scale2D.x should be beam length (300.0), got {}",
        scale.x
    );
    assert!(
        (scale.y - 20.0).abs() < f32::EPSILON,
        "Scale2D.y should be beam width (2*10.0=20.0), got {}",
        scale.y
    );
}

#[test]
fn piercing_beam_flash_scale_short_beam() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 1.0,
        half_width: 5.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Scale2D, With<EffectFlashTimer>>();
    let scale = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Scale2D");
    assert!(
        (scale.x - 1.0).abs() < f32::EPSILON,
        "Scale2D.x should be 1.0, got {}",
        scale.x
    );
    assert!(
        (scale.y - 10.0).abs() < f32::EPSILON,
        "Scale2D.y should be 10.0, got {}",
        scale.y
    );
}

// ── Behavior 10: Flash rotated to beam direction ──

#[test]
fn piercing_beam_flash_rotation_horizontal_beam() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(1.0, 0.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Rotation2D, With<EffectFlashTimer>>();
    let rotation = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Rotation2D");
    // atan2(0.0, 1.0) = 0.0
    assert!(
        (rotation.as_radians() - 0.0).abs() < 0.01,
        "horizontal beam rotation should be ~0.0 radians, got {}",
        rotation.as_radians()
    );
}

#[test]
fn piercing_beam_flash_rotation_vertical_beam() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Rotation2D, With<EffectFlashTimer>>();
    let rotation = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Rotation2D");
    // atan2(1.0, 0.0) = PI/2
    assert!(
        (rotation.as_radians() - std::f32::consts::FRAC_PI_2).abs() < 0.01,
        "vertical beam rotation should be ~PI/2 radians, got {}",
        rotation.as_radians()
    );
}

#[test]
fn piercing_beam_flash_rotation_diagonal_beam() {
    let mut app = flash_test_app();

    let dir = Vec2::new(1.0, 1.0).normalize();
    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: dir,
        length: 300.0,
        half_width: 10.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut query = app
        .world_mut()
        .query_filtered::<&Rotation2D, With<EffectFlashTimer>>();
    let rotation = query
        .iter(app.world())
        .next()
        .expect("flash entity should have Rotation2D");
    // atan2(0.707, 0.707) = PI/4
    assert!(
        (rotation.as_radians() - std::f32::consts::FRAC_PI_4).abs() < 0.01,
        "diagonal beam rotation should be ~PI/4 radians, got {}",
        rotation.as_radians()
    );
}

// ── Behavior 11: Multiple requests spawn independent flash entities ──

#[test]
fn multiple_piercing_beam_requests_spawn_independent_flash_entities() {
    let mut app = flash_test_app();

    let req_a = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 100.0,
            half_width: 5.0,
            damage: 10.0,
        })
        .id();

    let req_b = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 200.0,
            half_width: 5.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    // Both requests despawned
    assert!(
        app.world().get_entity(req_a).is_err(),
        "request A should be despawned"
    );
    assert!(
        app.world().get_entity(req_b).is_err(),
        "request B should be despawned"
    );

    // Two flash entities spawned
    let mut query = app
        .world_mut()
        .query_filtered::<&EffectFlashTimer, With<EffectFlashTimer>>();
    let flash_count = query.iter(app.world()).count();
    assert_eq!(
        flash_count, 2,
        "expected 2 flash entities (one per request), got {flash_count}"
    );
}

// ── Behavior 12: Flash spawns even with no cells ──

#[test]
fn piercing_beam_flash_spawns_with_no_cells_present() {
    let mut app = flash_test_app();

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned"
    );

    let mut flash_query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_count = flash_query.iter(app.world()).count();
    assert_eq!(
        flash_count, 1,
        "flash entity should be spawned even with no cells, got {flash_count}"
    );

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no damage should be dealt with no cells"
    );
}

// ── Behavior 13: Zero-length beam does not spawn flash ──

#[test]
fn piercing_beam_zero_length_does_not_spawn_flash() {
    let mut app = flash_test_app();

    let request = app
        .world_mut()
        .spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 0.0,
            half_width: 5.0,
            damage: 10.0,
        })
        .id();

    tick(&mut app);

    assert!(
        app.world().get_entity(request).is_err(),
        "request should be despawned even with zero length"
    );

    let mut flash_query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_count = flash_query.iter(app.world()).count();
    assert_eq!(
        flash_count, 0,
        "zero-length beam should NOT spawn a flash entity, got {flash_count}"
    );
}

#[test]
fn piercing_beam_near_zero_length_does_not_spawn_flash() {
    let mut app = flash_test_app();

    app.world_mut().spawn(PiercingBeamRequest {
        origin: Vec2::new(0.0, 0.0),
        direction: Vec2::new(0.0, 1.0),
        length: f32::EPSILON / 2.0,
        half_width: 5.0,
        damage: 10.0,
    });

    tick(&mut app);

    let mut flash_query = app
        .world_mut()
        .query_filtered::<Entity, With<EffectFlashTimer>>();
    let flash_count = flash_query.iter(app.world()).count();
    assert_eq!(
        flash_count, 0,
        "near-zero-length beam should NOT spawn a flash entity, got {flash_count}"
    );
}
