//! Section G: .locked(entities) Optional Method

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::components::{Cell, Locked, Locks},
    shared::death_pipeline::hp::Hp,
};

// Behavior 31: .locked(entities) stores lock data for spawn
#[test]
fn locked_inserts_locked_and_lock_adjacents() {
    let mut world = World::new();
    let e1 = world.spawn_empty().id();
    let e2 = world.spawn_empty().id();

    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .locked(vec![e1, e2])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Locked>(entity).is_some(),
        "entity should have Locked marker"
    );
    let adjacents = world
        .get::<Locks>(entity)
        .expect("entity should have Locks");
    assert_eq!(adjacents.0.len(), 2, "Locks should have 2 entities");
    assert_eq!(adjacents.0[0], e1);
    assert_eq!(adjacents.0[1], e2);
}

// Behavior 31 edge case: locked with empty vec
#[test]
fn locked_empty_vec_still_inserts_markers() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .locked(vec![])
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<Locked>(entity).is_some(),
        "entity should have Locked marker even with empty vec"
    );
    let adjacents = world
        .get::<Locks>(entity)
        .expect("entity should have Locks");
    assert!(adjacents.0.is_empty(), "Locks should be empty");
}

// Behavior 32: .locked() is available in any typestate (compile test)
#[test]
fn locked_available_in_any_typestate() {
    let _builder = Cell::builder().locked(vec![]);
    // Compiles — that is the assertion.
}

// Behavior 33: Cell without .locked() has no lock components
#[test]
fn no_locked_has_no_lock_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    // Guard: non-#[require] component ensures builder actually ran
    assert!(
        world.get::<Hp>(entity).is_some(),
        "entity should have Hp from builder"
    );

    assert!(
        world.get::<Locked>(entity).is_none(),
        "entity should NOT have Locked without .locked()"
    );
    assert!(
        world.get::<Locks>(entity).is_none(),
        "entity should NOT have Locks without .locked()"
    );
}
