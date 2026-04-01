//! Tests for `effective_damage_multiplier` scaling in `tick_tether_beam`.

use super::super::helpers::*;

#[test]
fn tick_tether_beam_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    // Tether beam with damage_mult=2.0, effective_damage_multiplier=1.5
    let (_bolt_a, _bolt_b, _beam) = spawn_tether_beam_with_edm(
        &mut app,
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        2.0,
        1.5,
    );
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = BASE_BOLT_DAMAGE * damage_mult * effective_damage_multiplier
    //        = 10.0 * 2.0 * 1.5 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 2.0 * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 2.0 * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

#[test]
fn tick_tether_beam_damage_zero_edm_produces_zero() {
    let mut app = damage_test_app();

    // EDM = 0.0 should produce zero damage
    let (_bolt_a, _bolt_b, _beam) = spawn_tether_beam_with_edm(
        &mut app,
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        2.0,
        0.0,
    );
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell even with zero EDM"
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = 10.0 * 2.0 * 0.0 = 0.0
    assert!(
        (collector.0[0].damage - 0.0).abs() < f32::EPSILON,
        "zero EDM should produce zero damage, got {}",
        collector.0[0].damage
    );
}
