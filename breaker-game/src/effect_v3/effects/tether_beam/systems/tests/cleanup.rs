use bevy::prelude::*;

use super::helpers::*;
use crate::{effect_v3::effects::tether_beam::components::*, shared::test_utils::tick};

// ── Group C — cleanup_tether_beams despawn semantics ───────────────────

#[test]
fn cleanup_despawns_beam_when_bolt_a_is_gone() {
    let mut app = cleanup_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    app.world_mut().despawn(bolt_a);

    tick(&mut app);

    let beam_count = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .count();
    assert_eq!(beam_count, 0);
}

#[test]
fn cleanup_despawns_beam_when_bolt_b_is_gone() {
    let mut app = cleanup_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    let beam_count = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .count();
    assert_eq!(beam_count, 0);
}

#[test]
fn cleanup_leaves_beam_intact_when_both_endpoints_alive() {
    let mut app = cleanup_test_app();

    // Beam 1
    let beam1_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let beam1_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    app.world_mut().spawn((
        TetherBeamSource {
            bolt_a: beam1_a,
            bolt_b: beam1_b,
        },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    // Beam 2 — independent pair of alive bolts
    let beam2_a = spawn_endpoint(&mut app, Vec2::new(200.0, 0.0));
    let beam2_b = spawn_endpoint(&mut app, Vec2::new(300.0, 0.0));
    app.world_mut().spawn((
        TetherBeamSource {
            bolt_a: beam2_a,
            bolt_b: beam2_b,
        },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let beam_count = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .count();
    assert_eq!(beam_count, 2);

    // Verify the first beam still points at its original endpoints.
    let beams: Vec<TetherBeamSource> = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .cloned()
        .collect();
    let has_beam1 = beams
        .iter()
        .any(|b| b.bolt_a == beam1_a && b.bolt_b == beam1_b);
    let has_beam2 = beams
        .iter()
        .any(|b| b.bolt_a == beam2_a && b.bolt_b == beam2_b);
    assert!(has_beam1, "beam 1 endpoint pair must still exist");
    assert!(has_beam2, "beam 2 endpoint pair must still exist");
}

#[test]
fn cleanup_despawns_beam_when_both_endpoints_are_gone() {
    let mut app = cleanup_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    app.world_mut().despawn(bolt_a);
    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    let beam_count = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .count();
    assert_eq!(beam_count, 0);
}
