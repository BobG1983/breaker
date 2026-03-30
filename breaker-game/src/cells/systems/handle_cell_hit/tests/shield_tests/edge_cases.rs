//! Behaviors 10-12: Zero charges, two-hit dedup, `source_chip` absorption.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

// ── Behavior 10: ShieldActive with charges: 0 offers no protection ──

#[test]
fn shield_active_with_zero_charges_offers_no_protection() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 0 } (degenerate).
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // Then: Damage applies normally. Cell destroyed (10-10=0). RequestCellDestroyed sent.
    //       ShieldActive { charges: 0 } still present (no code path removes it in this case).
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: None,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "ShieldActive with charges: 0 should not protect — cell destroyed, RequestCellDestroyed sent"
    );

    // ShieldActive { charges: 0 } still present (degenerate state preserved)
    let shield = app.world().get::<ShieldActive>(cell).expect(
        "ShieldActive { charges: 0 } should still be present on entity (degenerate state not cleaned)"
    );
    assert_eq!(
        shield.charges, 0,
        "charges should remain 0 (no decrement on zero-charge path)"
    );
}

// ── Behavior 11: Two hits both absorbed, no dedup entry ──

#[test]
fn two_hits_both_absorbed_no_dedup_no_destruction() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 3 }.
    //        Two DamageCell messages, damage 10.0 each.
    // Then: Both absorbed. Health 10.0. Charges 3 to 1. No RequestCellDestroyed.
    let mut app = test_app();
    let cell = spawn_shielded_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell,
            damage: 10.0,
            source_chip: None,
        },
    ];
    app.add_systems(
        FixedUpdate,
        (
            enqueue_all.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "both hits absorbed, HP should remain 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "both hits absorbed, no RequestCellDestroyed"
    );
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 1,
        "two hits absorbed: charges should go from 3 to 1, got {}",
        shield.charges
    );
}

// ── Behavior 12: Shield absorbs hit from DamageCell with source_chip set ──

#[test]
fn shield_absorbs_hit_with_source_chip() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 2 }.
    //        DamageCell { cell, damage: 10.0, source_chip: Some("shockwave".into()) }.
    // Then: Damage absorbed. Health 10.0. Charges 2 to 1.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 2);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 10.0,
        source_chip: Some("shockwave".into()),
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_from_resource.before(handle_cell_hit),
            capture_destroyed.after(handle_cell_hit),
        ),
    );
    tick(&mut app);

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell should absorb hit with source_chip, HP should be 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed"
    );
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 1,
        "shield charges should decrement from 2 to 1, got {}",
        shield.charges
    );
}
