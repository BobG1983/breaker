//! Behaviors 4-6b: Multiple hits per frame, excess hits apply damage, dedup.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

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
