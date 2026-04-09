//! Section J: `CellBehavior::Guarded` Insertion in `spawn_inner`

use bevy::prelude::*;

use super::helpers::*;
use crate::cells::{
    components::{Cell, CellHealth, GuardedCell, GuardianCell},
    definition::{CellBehavior, GuardedBehavior},
};

// Behavior 38: CellBehavior::Guarded inserts GuardedCell marker
#[test]
fn behavior_guarded_inserts_guarded_cell_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Guarded(GuardedBehavior {
                guardian_hp_fraction: 0.5,
                guardian_color_rgb: [0.5, 0.8, 1.0],
                slide_speed: 30.0,
            }))
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GuardedCell>(entity).is_some(),
        "entity with CellBehavior::Guarded should have GuardedCell component"
    );
}

// Behavior 38 edge case: behavior does NOT spawn guardian children
#[test]
fn behavior_guarded_does_not_spawn_children() {
    let mut world = World::new();
    let _entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Guarded(GuardedBehavior {
                guardian_hp_fraction: 0.5,
                guardian_color_rgb: [0.5, 0.8, 1.0],
                slide_speed: 30.0,
            }))
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let guardian_count = world.query::<&GuardianCell>().iter(&world).count();
    assert_eq!(
        guardian_count, 0,
        "behavior alone should NOT spawn guardian children"
    );
}

// Behavior 40: Cell without Guarded behavior has no GuardedCell component
#[test]
fn cell_without_guarded_behavior_has_no_guarded_cell() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: builder ran
    assert!(world.get::<CellHealth>(entity).is_some());
    assert!(
        world.get::<GuardedCell>(entity).is_none(),
        "cell without Guarded behavior should NOT have GuardedCell"
    );
}

// Behavior 40 edge case: Regen only has no GuardedCell
#[test]
fn cell_with_regen_only_has_no_guarded_cell() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Regen { rate: 2.0 })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<GuardedCell>(entity).is_none(),
        "cell with only Regen behavior should NOT have GuardedCell"
    );
}
