//! Behaviors 8-9: Shielded vs unshielded cells, both shielded independently.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    cells::{components::CellHealth, messages::DamageCell},
    effect::effects::shield::ShieldActive,
};

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
