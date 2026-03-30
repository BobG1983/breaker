//! Tests for explode damage scaling by `EffectiveDamageMultiplier`.

use bevy::prelude::*;

use super::{super::effect::*, helpers::*};
use crate::bolt::BASE_BOLT_DAMAGE;

// -- Damage scaling: Explode damage scales by source entity's EffectiveDamageMultiplier ──

#[test]
fn explode_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // fire() on an entity with EDM(1.5), damage_mult=2.0
    // Expected: DamageCell.damage = 10.0 * 2.0 * 1.5 = 30.0
    let source = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::EffectiveDamageMultiplier(1.5),
        ))
        .id();

    fire(source, 50.0, 2.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = BASE_BOLT_DAMAGE * 2.0 * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 2.0 * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

#[test]
fn explode_damage_with_edm_and_unit_damage_mult() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // damage_mult=1.0, EDM=2.0 => damage = 10.0 * 1.0 * 2.0 = 20.0
    let source = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::effect::EffectiveDamageMultiplier(2.0),
        ))
        .id();

    fire(source, 50.0, 1.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = BASE_BOLT_DAMAGE * 1.0 * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 1.0 * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
}
