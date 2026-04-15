use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::components::BoltBaseDamage,
    cells::components::Cell,
    effect_v3::traits::Fireable,
    shared::{
        death_pipeline::{DamageDealt, Dead},
        test_utils::MessageCollector,
    },
};

// ── B1: Cell directly ahead is hit ────────────────────────────────

#[test]
fn cell_directly_ahead_of_bolt_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 200.0))))
        .id();

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "cell directly ahead should be hit");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "damage should be 10.0 * 1.0 = 10.0, got {}",
        msgs.0[0].amount,
    );
    assert_eq!(msgs.0[0].dealer, Some(source));
}

#[test]
fn cell_barely_ahead_of_bolt_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 1.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "cell barely ahead (1 unit) should be hit");
}

// ── B1: Cell behind bolt is NOT hit ───────────────────────────────

#[test]
fn cell_behind_bolt_is_not_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 100.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 0, "cell behind bolt should NOT be hit");
}

#[test]
fn cell_one_unit_behind_bolt_is_not_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 100.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 99.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 0, "cell 1 unit behind should NOT be hit");
}

// ── B1: Cell at bolt position (along == 0) is hit ─────────────────

#[test]
fn cell_at_bolt_position_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(50.0, 50.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    let cell = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(50.0, 50.0))))
        .id();

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "cell at exact bolt position (along == 0) should be hit"
    );
    assert_eq!(msgs.0[0].target, cell);
}

// ── B2: Width threshold — within half_width ───────────────────────

#[test]
fn cell_within_half_width_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // 9 units to the right, within half_width of 10.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(9.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "cell within half_width (9 < 10) should be hit"
    );
}

#[test]
fn cell_within_half_width_left_side_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // 9 units to the left — symmetric check.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(-9.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "cell 9 units to the left should also be hit (symmetric)"
    );
}

// ── B2: Width threshold — exactly at boundary ─────────────────────

#[test]
fn cell_exactly_at_half_width_boundary_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // Exactly at half_width boundary (10.0).
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(10.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "cell exactly at half_width (perp <= half_width) should be hit"
    );
}

#[test]
fn cell_exactly_at_negative_half_width_boundary_is_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(-10.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "cell at -10.0 (negative side boundary) should also be hit"
    );
}

// ── B2: Width threshold — outside half_width ──────────────────────

#[test]
fn cell_outside_half_width_is_not_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(11.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "cell outside half_width (11 > 10) should NOT be hit"
    );
}

#[test]
fn cell_outside_negative_half_width_is_not_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(-11.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 0, "cell at -11 should NOT be hit");
}

// ── B3: Diagonal beam direction ───────────────────────────────────

#[test]
fn beam_fires_along_diagonal_velocity_direction() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(300.0, 400.0)),
        ))
        .id();

    // Cell exactly along the beam direction at distance 50.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(30.0, 40.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "cell along diagonal beam should be hit");
}

#[test]
fn cell_behind_diagonal_beam_is_not_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(300.0, 400.0)),
        ))
        .id();

    // Cell in the opposite direction.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(-30.0, -40.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "cell behind diagonal beam should NOT be hit"
    );
}

// ── B4: Dead cell filtering ───────────────────────────────────────

#[test]
fn dead_cells_are_excluded_from_beam() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0)), Dead));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "dead cells should be excluded from beam hits"
    );
}

#[test]
fn dead_cell_excluded_alive_cell_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // Dead cell ahead.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0)), Dead));
    // Alive cell ahead.
    let alive = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 200.0))))
        .id();

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "only alive cell should produce damage");
    assert_eq!(msgs.0[0].target, alive);
}

// ── B9: Missing source entity components ──────────────────────────

#[test]
fn source_without_velocity_defaults_direction_to_y() {
    let mut app = piercing_test_app();

    // No Velocity2D component.
    let source = app
        .world_mut()
        .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
        .id();

    // Cell directly above (along Vec2::Y).
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should fall back to Vec2::Y and hit cell above"
    );
}

#[test]
fn source_without_velocity_does_not_hit_perpendicular_cell() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
        .id();

    // Cell far to the right — perpendicular to Y direction, outside beam width.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(100.0, 0.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        0,
        "cell 100 units perpendicular to Y should not be hit"
    );
}

#[test]
fn source_with_zero_velocity_defaults_direction_to_y() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::ZERO),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "zero-magnitude velocity should fall back to Vec2::Y via normalize_or",
    );
}

#[test]
fn source_without_position_defaults_origin_to_zero() {
    let mut app = piercing_test_app();

    // No Position2D component.
    let source = app
        .world_mut()
        .spawn((BoltBaseDamage(10.0), Velocity2D(Vec2::new(0.0, 400.0))))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

    geometry_config().fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        msgs.0.len(),
        1,
        "missing Position2D should default to Vec2::ZERO as origin",
    );
}
