//! Tests for shockwave damage scaling by `ShockwaveDamageMultiplier`: standard,
//! high, and zero multiplier values.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::super::*;
use crate::bolt::BASE_BOLT_DAMAGE;

// -- Damage scaling: Shockwave damage scales by ShockwaveDamageMultiplier ──

#[test]
fn shockwave_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    // Spawn shockwave with ShockwaveDamageMultiplier(2.0)
    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        ShockwaveDamageMultiplier(2.0),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = BASE_BOLT_DAMAGE * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (BASE_BOLT_DAMAGE * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

// -- Damage scaling: High multiplier across multiple cells ──

#[test]
fn shockwave_damage_scales_with_high_multiplier_across_multiple_cells() {
    let mut app = damage_test_app();

    let cell1 = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell2 = spawn_test_cell(&mut app, 0.0, 15.0);
    let cell3 = spawn_test_cell(&mut app, -20.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        ShockwaveDamageMultiplier(3.5),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageCell messages, got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell1), "cell1 should be damaged");
    assert!(damaged_cells.contains(&cell2), "cell2 should be damaged");
    assert!(damaged_cells.contains(&cell3), "cell3 should be damaged");

    let expected_damage = BASE_BOLT_DAMAGE * 3.5;
    for msg in &collector.0 {
        assert!(
            (msg.damage - expected_damage).abs() < f32::EPSILON,
            "each cell damage should be BASE_BOLT_DAMAGE * 3.5 = {}, got {}",
            expected_damage,
            msg.damage
        );
    }
}

#[test]
fn shockwave_damage_zero_multiplier_produces_zero_damage() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource,
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        ShockwaveDamageMultiplier(0.0),
        Position2D(Vec2::new(0.0, 0.0)),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell even with zero multiplier"
    );
    assert_eq!(collector.0[0].cell, cell);
    assert!(
        (collector.0[0].damage - 0.0).abs() < f32::EPSILON,
        "zero multiplier should produce zero damage, got {}",
        collector.0[0].damage
    );
}
