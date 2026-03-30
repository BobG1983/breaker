use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

// =========================================================================
// Shield Charge-Based Damage Absorption — handle_cell_hit shield behaviors
// =========================================================================

// ── Behavior 1: Shielded cell absorbs damage and decrements charges by 1 ──

#[test]
fn shielded_cell_absorbs_damage_and_decrements_one_charge() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 3 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // When: handle_cell_hit runs
    // Then: Cell health remains 10.0. No RequestCellDestroyed. ShieldActive.charges is 2.
    let mut app = test_app();
    let cell = spawn_shielded_cell(&mut app, 10.0);

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

    // Health should remain unchanged (damage absorbed)
    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 10.0, got {}",
        health.current
    );

    // No destruction
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed"
    );

    // Charges decremented from 3 to 2
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 2,
        "shield charges should decrement from 3 to 2 after absorbing one hit, got {}",
        shield.charges
    );
}

#[test]
fn shielded_cell_absorbs_massive_overkill_and_decrements_one_charge() {
    // Behavior 1 edge case: DamageCell with damage 999.0 (massive overkill).
    // Shield absorbs the hit regardless of damage amount.
    let mut app = test_app();
    let cell = spawn_shielded_cell(&mut app, 10.0);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 999.0,
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

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell should absorb 999.0 overkill damage, HP should be 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed even with overkill damage"
    );

    // Charges decremented from 3 to 2
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 2,
        "shield charges should decrement from 3 to 2 after absorbing overkill hit, got {}",
        shield.charges
    );
}

// ── Behavior 2: Shield absorbs damage regardless of damage amount ──

#[test]
fn shield_absorbs_damage_regardless_of_amount() {
    // Given: Cell with CellHealth::new(5.0) and ShieldActive { charges: 2 }.
    //        DamageCell { cell, damage: 50.0, source_chip: None }.
    // Then: Health remains 5.0. charges 2 to 1.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 5.0, 2);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 50.0,
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

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 5.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 5.0, got {}",
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

// ── Behavior 3: Last charge consumed removes ShieldActive component ──

#[test]
fn last_charge_consumed_removes_shield_active() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 1 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // Then: Health remains 10.0. ShieldActive removed (charges reached 0).
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);

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

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "shielded cell HP should remain 10.0 (last charge absorbs), got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "shielded cell should not produce RequestCellDestroyed"
    );

    // ShieldActive should be removed after charges reach 0
    assert!(
        app.world().get::<ShieldActive>(cell).is_none(),
        "ShieldActive should be removed when last charge is consumed"
    );
}

#[test]
fn last_charge_consumed_with_zero_damage() {
    // Edge case: ShieldActive { charges: 1 } with damage 0.0 — still consumes charge.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);

    app.init_resource::<CapturedDestroyed>();
    app.insert_resource(TestMessage(Some(DamageCell {
        cell,
        damage: 0.0,
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

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "zero-damage hit on shielded cell should keep HP at 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "zero-damage hit on shielded cell should not produce RequestCellDestroyed"
    );

    // Shield removed because the charge was consumed even with 0 damage
    assert!(
        app.world().get::<ShieldActive>(cell).is_none(),
        "ShieldActive should be removed when last charge consumed, even with zero damage"
    );
}

// ── Behavior 4: Multiple hits in same frame each consume one charge ──

#[test]
fn multiple_hits_same_frame_each_consume_one_charge() {
    // Given: Cell with CellHealth::new(30.0) and ShieldActive { charges: 2 }.
    //        Two DamageCell messages, each damage 10.0.
    // Then: Both absorbed. Health 30.0. Charges 2 to 0. ShieldActive removed.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 30.0, 2);

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
        (health.current - 30.0).abs() < f32::EPSILON,
        "both hits absorbed, HP should remain 30.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "both hits absorbed, no RequestCellDestroyed"
    );

    // ShieldActive removed (2 charges consumed, reached 0)
    assert!(
        app.world().get::<ShieldActive>(cell).is_none(),
        "ShieldActive should be removed after both charges consumed"
    );
}

// ── Behavior 5: More hits than charges — excess hits apply damage normally ──

#[test]
fn excess_hits_beyond_charges_apply_damage_normally() {
    // Given: Cell with CellHealth::new(30.0) and ShieldActive { charges: 1 }.
    //        Three DamageCell messages, each damage 10.0.
    // Then: First absorbed (charges 1 to 0, removal queued). Second: health 30->20.
    //       Third: health 20->10. Final HP = 10.0. ShieldActive absent.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 30.0, 1);

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
        "one hit absorbed + two applied: 30 - 10 - 10 = 10.0 HP, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "cell not destroyed (HP 10.0 > 0), no RequestCellDestroyed"
    );

    // ShieldActive absent (deferred removal applied)
    assert!(
        app.world().get::<ShieldActive>(cell).is_none(),
        "ShieldActive should be absent after charge depleted"
    );
}

// ── Behavior 6: More hits than charges — excess hit destroys cell ──

#[test]
fn excess_hit_beyond_charges_destroys_cell() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 1 }.
    //        Two DamageCell messages: first damage 5.0, second damage 10.0.
    // Then: First absorbed (charges 1 to 0). Second: take_damage(10.0) destroys cell.
    //       Exactly one RequestCellDestroyed.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 5.0,
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

    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "one RequestCellDestroyed expected after excess hit destroys cell"
    );
    assert_eq!(
        captured.0[0].cell, cell,
        "RequestCellDestroyed should reference the destroyed cell"
    );
}

// ── Behavior 6b: Three-message dedup — shield absorb + destroy + dedup ──

#[test]
fn three_message_dedup_shield_absorb_destroy_dedup() {
    // Given: Cell with CellHealth::new(10.0) and ShieldActive { charges: 1 }.
    //        Three DamageCell messages: damage 5.0, 10.0, 10.0.
    // Then: First absorbed. Second destroys cell. Third deduped. One RequestCellDestroyed.
    let mut app = test_app();
    let cell = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell,
            damage: 5.0,
            source_chip: None,
        },
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

    let captured = app.world().resource::<CapturedDestroyed>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one RequestCellDestroyed (third message deduped)"
    );
}

// ── Behavior 7: Locked cell is checked before shield ──

#[test]
fn locked_cell_checked_before_shield_charges_not_consumed() {
    // Given: Cell with CellHealth::new(10.0), Locked component, AND ShieldActive { charges: 3 }.
    //        DamageCell { cell, damage: 10.0, source_chip: None }.
    // Then: Locked guard fires first. Health 10.0. Charges remain 3.
    let mut app = test_app();
    let cell = spawn_locked_cell(&mut app, 10.0);
    app.world_mut()
        .entity_mut(cell)
        .insert(ShieldActive { charges: 3 });

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

    let health = app.world().get::<CellHealth>(cell).unwrap();
    assert!(
        (health.current - 10.0).abs() < f32::EPSILON,
        "locked+shielded cell HP should remain 10.0, got {}",
        health.current
    );
    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "locked+shielded cell should not produce RequestCellDestroyed"
    );

    // Shield charges must remain unchanged (Locked guard fires first)
    let shield = app.world().get::<ShieldActive>(cell).unwrap();
    assert_eq!(
        shield.charges, 3,
        "locked guard should fire before shield — charges should remain 3, got {}",
        shield.charges
    );
}

// ── Behavior 8: Shielded cell absorbs but unshielded cell takes damage ──

#[test]
fn shielded_cell_absorbs_but_unshielded_cell_takes_damage() {
    // Given: Cell A with CellHealth::new(10.0) and ShieldActive { charges: 3 }.
    //        Cell B with CellHealth::new(30.0) and no ShieldActive.
    //        DamageCell for each cell, damage 10.0.
    // Then: Cell A health 10.0, charges 3 to 2. Cell B health 20.0 (30-10).
    let mut app = test_app();
    let cell_a = spawn_shielded_cell(&mut app, 10.0);
    let cell_b = spawn_cell(&mut app, 30.0);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell: cell_a,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell: cell_b,
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

    let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
    assert!(
        (health_a.current - 10.0).abs() < f32::EPSILON,
        "shielded cell A HP should remain 10.0, got {}",
        health_a.current
    );
    let shield_a = app.world().get::<ShieldActive>(cell_a).unwrap();
    assert_eq!(
        shield_a.charges, 2,
        "cell A shield charges should decrement from 3 to 2, got {}",
        shield_a.charges
    );

    let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
    assert!(
        (health_b.current - 20.0).abs() < f32::EPSILON,
        "unshielded cell B HP should be 30.0 - 10.0 = 20.0, got {}",
        health_b.current
    );

    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(
        captured.0.is_empty(),
        "neither cell should produce RequestCellDestroyed"
    );
}

// ── Behavior 9: Both cells shielded — both absorb independently ──

#[test]
fn both_shielded_cells_absorb_independently() {
    // Given: Cell A with CellHealth::new(10.0) and ShieldActive { charges: 1 }.
    //        Cell B with CellHealth::new(30.0) and ShieldActive { charges: 2 }.
    //        DamageCell for each, damage 10.0.
    // Then: Cell A health 10.0, ShieldActive removed (1 to 0).
    //       Cell B health 30.0, charges 2 to 1.
    let mut app = test_app();
    let cell_a = spawn_shielded_cell_with_charges(&mut app, 10.0, 1);
    let cell_b = spawn_shielded_cell_with_charges(&mut app, 30.0, 2);

    app.init_resource::<CapturedDestroyed>();
    app.init_resource::<TestMessages>();
    app.world_mut().resource_mut::<TestMessages>().0 = vec![
        DamageCell {
            cell: cell_a,
            damage: 10.0,
            source_chip: None,
        },
        DamageCell {
            cell: cell_b,
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

    // Cell A: health unchanged, shield removed
    let health_a = app.world().get::<CellHealth>(cell_a).unwrap();
    assert!(
        (health_a.current - 10.0).abs() < f32::EPSILON,
        "cell A HP should remain 10.0, got {}",
        health_a.current
    );
    assert!(
        app.world().get::<ShieldActive>(cell_a).is_none(),
        "cell A ShieldActive should be removed (charges 1 to 0)"
    );

    // Cell B: health unchanged, charges decremented
    let health_b = app.world().get::<CellHealth>(cell_b).unwrap();
    assert!(
        (health_b.current - 30.0).abs() < f32::EPSILON,
        "cell B HP should remain 30.0, got {}",
        health_b.current
    );
    let shield_b = app.world().get::<ShieldActive>(cell_b).unwrap();
    assert_eq!(
        shield_b.charges, 1,
        "cell B shield charges should decrement from 2 to 1, got {}",
        shield_b.charges
    );

    let captured = app.world().resource::<CapturedDestroyed>();
    assert!(captured.0.is_empty(), "no cells should be destroyed");
}

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
