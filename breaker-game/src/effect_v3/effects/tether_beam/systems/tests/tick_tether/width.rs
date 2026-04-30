use bevy::prelude::*;

use super::super::helpers::*;
use crate::{effect_v3::effects::tether_beam::components::*, shared::test_utils::tick};

// ── Group A (width) — TetherBeamWidth drives per-entity hit-test ───────

/// Behavior 1: Narrow beam (width 5.0) damages only cells inside or on
/// the 5.0 half-width boundary.
#[test]
fn narrow_beam_width_5_damages_only_cells_within_5_units_perpendicular() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_on_line = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
    let cell_inside = spawn_alive_cell(&mut app, Vec2::new(50.0, 4.0));
    let cell_boundary = spawn_alive_cell(&mut app, Vec2::new(50.0, 5.0));
    let _cell_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 6.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(5.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(
        msgs.len(),
        3,
        "expected exactly 3 DamageDealt<Cell> messages (on_line, inside, boundary)"
    );
    let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
    assert!(
        targets.contains(&cell_on_line),
        "cell on beam line must be damaged"
    );
    assert!(
        targets.contains(&cell_inside),
        "cell at distance 4.0 must be damaged"
    );
    assert!(
        targets.contains(&cell_boundary),
        "cell exactly on half-width 5.0 must be damaged (inclusive boundary)"
    );
}

/// Behavior 2: Wide beam (width 20.0) damages cells out to 20.0 perp
/// distance — locks that width is not stale 10.0 constant.
#[test]
fn wide_beam_width_20_damages_cells_out_to_20_units_perpendicular() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_beyond_old_10 = spawn_alive_cell(&mut app, Vec2::new(50.0, 11.0));
    let cell_near_boundary = spawn_alive_cell(&mut app, Vec2::new(50.0, 19.0));
    let cell_boundary = spawn_alive_cell(&mut app, Vec2::new(50.0, 20.0));
    let _cell_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 21.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(20.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(
        msgs.len(),
        3,
        "expected 3 damage messages for cells within half-width 20.0"
    );
    let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
    assert!(
        targets.contains(&cell_beyond_old_10),
        "cell at 11.0 must be damaged — proves width is NOT stale 10.0"
    );
    assert!(targets.contains(&cell_near_boundary));
    assert!(
        targets.contains(&cell_boundary),
        "cell exactly on half-width 20.0 must be damaged (inclusive boundary)"
    );
}

/// Behavior 3: Two beams in the same world read width per-entity.
/// Regression lock: width must be stamped on the entity, not hardcoded.
#[test]
fn two_coexisting_beams_use_per_entity_widths() {
    let mut app = tether_test_app();

    // Beam 1: narrow (width 5.0) along y=0
    let beam1_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let beam1_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_narrow_inside = spawn_alive_cell(&mut app, Vec2::new(50.0, 3.0));
    let _cell_narrow_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 8.0));
    let beam1_entity = app
        .world_mut()
        .spawn((
            TetherBeamSource {
                bolt_a: beam1_a,
                bolt_b: beam1_b,
            },
            TetherBeamDamage(10.0),
            TetherBeamWidth(5.0),
        ))
        .id();

    // Beam 2: wide (width 20.0) along y=200
    let beam2_a = spawn_endpoint(&mut app, Vec2::new(0.0, 200.0));
    let beam2_b = spawn_endpoint(&mut app, Vec2::new(100.0, 200.0));
    let cell_wide_inside = spawn_alive_cell(&mut app, Vec2::new(50.0, 215.0));
    let _cell_wide_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 225.0));
    let beam2_entity = app
        .world_mut()
        .spawn((
            TetherBeamSource {
                bolt_a: beam2_a,
                bolt_b: beam2_b,
            },
            TetherBeamDamage(20.0),
            TetherBeamWidth(20.0),
        ))
        .id();

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(
        msgs.len(),
        2,
        "expected exactly 2 damage messages (one per beam, inside-cells only)"
    );

    let beam1_msg = msgs
        .iter()
        .find(|m| m.target == cell_narrow_inside)
        .expect("narrow-inside cell must be damaged by beam 1");
    assert_eq!(beam1_msg.dealer, Some(beam1_entity));
    assert!((beam1_msg.amount - 10.0).abs() < 1e-6);

    let beam2_msg = msgs
        .iter()
        .find(|m| m.target == cell_wide_inside)
        .expect("wide-inside cell must be damaged by beam 2");
    assert_eq!(beam2_msg.dealer, Some(beam2_entity));
    assert!((beam2_msg.amount - 20.0).abs() < 1e-6);
}

/// Behavior 4: Width 0.0 damages only cells exactly on the beam line.
#[test]
fn width_zero_damages_only_cells_exactly_on_beam_line() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_on_line = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
    let _cell_just_off = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.001));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(0.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(
        msgs.len(),
        1,
        "with width 0.0, only the cell exactly on the line may be damaged"
    );
    assert_eq!(msgs[0].target, cell_on_line);
}

/// Behavior 5: Diagonal beam with width 5.0 — perpendicular distance
/// from (60, 40) to line y=x is ~14.14, which is outside half-width 5.0.
#[test]
fn diagonal_beam_width_5_does_not_damage_cell_at_14_14_perpendicular() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 100.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(60.0, 40.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(5.0),
    ));

    tick(&mut app);

    assert_eq!(
        damage_msgs(&app).len(),
        0,
        "perp distance ~14.14 > half-width 5.0, must not be damaged"
    );
}

/// Behavior 6: Same diagonal geometry, width 20.0 — cell at ~14.14
/// perpendicular distance IS damaged.
#[test]
fn diagonal_beam_width_20_damages_cell_at_14_14_perpendicular() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 100.0));
    let cell = spawn_alive_cell(&mut app, Vec2::new(60.0, 40.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(20.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(
        msgs.len(),
        1,
        "perp distance ~14.14 < half-width 20.0, must be damaged exactly once"
    );
    assert_eq!(msgs[0].target, cell);
}

/// Behavior 7: Width and damage are orthogonal — width does not clamp
/// or scale damage.
#[test]
fn width_is_independent_of_damage_on_same_beam_entity() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 2.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(99.0),
        TetherBeamWidth(3.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 99.0).abs() < 1e-6,
        "damage must be 99.0 — width 3.0 must not affect damage amount"
    );
}

/// Behavior 7a: Beam entity without `TetherBeamWidth` is silently
/// skipped by the required-component query — not damaged with a fallback.
#[test]
fn beam_without_tether_beam_width_is_silently_skipped_by_query() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    // Intentionally spawn WITHOUT TetherBeamWidth — the tick query must
    // require &TetherBeamWidth and skip this beam entirely.
    let beam_entity = app
        .world_mut()
        .spawn((TetherBeamSource { bolt_a, bolt_b }, TetherBeamDamage(12.5)))
        .id();

    tick(&mut app);

    assert_eq!(
        damage_msgs(&app).len(),
        0,
        "beam without TetherBeamWidth must be skipped by the query — \
         this locks a required-component contract, not an Option fallback"
    );
    // The beam entity itself must NOT be despawned by tick (tick only
    // reads; cleanup is a separate system).
    assert!(
        app.world().get_entity(beam_entity).is_ok(),
        "tick_tether_beam must not despawn the beam"
    );
}
