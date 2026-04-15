use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{super::config_impl::*, helpers::*};
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    cells::components::Cell,
    effect_v3::traits::Fireable,
    shared::{
        death_pipeline::{DamageDealt, Dead},
        test_utils::MessageCollector,
    },
};

// ── C8: PiercingBeam base damage reads BoltBaseDamage from source entity ──

#[test]
fn piercing_beam_uses_bolt_base_damage_from_source_entity() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(30.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // Spawn a cell directly ahead of the bolt.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

    make_config().fire(source, "laser", app.world_mut());
    // Run an update cycle so the message collector picks up the written message.
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    let expected_damage = 30.0 * 2.0;
    assert!(
        (msgs.0[0].amount - expected_damage).abs() < f32::EPSILON,
        "piercing beam damage should be 30.0 * 2.0 = {expected_damage}, got {}",
        msgs.0[0].amount,
    );
}

#[test]
fn piercing_beam_zero_bolt_base_damage() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(0.0),
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

    make_config().fire(source, "laser", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    assert!(
        msgs.0[0].amount.abs() < f32::EPSILON,
        "piercing beam damage should be 0.0 * 2.0 = 0.0, got {}",
        msgs.0[0].amount,
    );
}

#[test]
fn piercing_beam_falls_back_to_default_when_bolt_base_damage_absent() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

    make_config().fire(source, "laser", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
    let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0;
    assert!(
        (msgs.0[0].amount - expected_damage).abs() < f32::EPSILON,
        "piercing beam damage should fall back to DEFAULT_BOLT_BASE_DAMAGE * 2.0 = {expected_damage}, got {}",
        msgs.0[0].amount,
    );
}

// ── B5: Damage multiplier propagation ─────────────────────────────

#[test]
fn damage_is_base_damage_times_damage_mult() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(15.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(3.0),
        width:       OrderedFloat(20.0),
    };
    config.fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1);
    assert!(
        (msgs.0[0].amount - 45.0).abs() < f32::EPSILON,
        "damage should be 15.0 * 3.0 = 45.0, got {}",
        msgs.0[0].amount,
    );
}

#[test]
fn damage_with_half_multiplier() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(15.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(0.5),
        width:       OrderedFloat(20.0),
    };
    config.fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1);
    assert!(
        (msgs.0[0].amount - 7.5).abs() < f32::EPSILON,
        "damage should be 15.0 * 0.5 = 7.5, got {}",
        msgs.0[0].amount,
    );
}

// ── B6: Source chip propagation ────────────────────────────────────

#[test]
fn non_empty_source_propagates_as_some_source_chip() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

    geometry_config().fire(source, "laser_chip", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1);
    assert_eq!(
        msgs.0[0].source_chip,
        Some("laser_chip".to_string()),
        "non-empty source should propagate as Some(source_chip)",
    );
}

#[test]
fn empty_source_propagates_as_none_source_chip() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

    geometry_config().fire(source, "", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1);
    assert_eq!(
        msgs.0[0].source_chip, None,
        "empty source string should propagate as None",
    );
}

// ── B7: Multiple cells hit ────────────────────────────────────────

#[test]
fn multiple_cells_within_beam_all_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    let cell_a = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))))
        .id();
    let cell_b = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 150.0))))
        .id();
    let cell_c = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 300.0))))
        .id();

    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(2.0),
        width:       OrderedFloat(20.0),
    };
    config.fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 3, "all 3 cells in beam should be hit");

    let targets: std::collections::HashSet<Entity> = msgs.0.iter().map(|m| m.target).collect();
    assert!(targets.contains(&cell_a));
    assert!(targets.contains(&cell_b));
    assert!(targets.contains(&cell_c));

    for msg in &msgs.0 {
        assert!(
            (msg.amount - 20.0).abs() < f32::EPSILON,
            "each message should have amount 10.0 * 2.0 = 20.0, got {}",
            msg.amount,
        );
    }
}

#[test]
fn mixed_inside_outside_dead_only_alive_inside_hit() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // Inside beam, alive.
    let alive_inside = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))))
        .id();
    // Outside beam (perp > half_width).
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(50.0, 100.0))));
    // Dead cell inside beam.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 200.0)), Dead));

    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(2.0),
        width:       OrderedFloat(20.0),
    };
    config.fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 1, "only alive cell inside beam should be hit");
    assert_eq!(msgs.0[0].target, alive_inside);
}

// ── B8: No cells in range ─────────────────────────────────────────

#[test]
fn fire_with_no_cells_emits_nothing_and_does_not_panic() {
    let mut app = piercing_test_app();

    let source = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    // No Cell entities.
    let config = PiercingBeamConfig {
        damage_mult: OrderedFloat(2.0),
        width:       OrderedFloat(20.0),
    };
    config.fire(source, "beam", app.world_mut());
    app.update();

    let msgs = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(msgs.0.len(), 0, "no cells should mean no damage messages");
}
