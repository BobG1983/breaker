//! Group A — Component attachment via builder.
//!
//! Pure builder-terminal assertions. No plugin, no state navigation — just
//! `spawn_cell_in_world` + `Cell::builder()` variants. Mirrors the pattern in
//! `breaker-game/src/cells/builder/tests/optional_tests/volatile.rs`.

use bevy::prelude::*;

use crate::{
    cells::{
        components::{SequenceActive, SequenceCell, SequenceGroup, SequencePosition, VolatileCell},
        definition::CellBehavior,
        test_utils::{spawn_cell_in_world, test_cell_definition},
    },
    prelude::*,
};

// ── Behavior 1 ─────────────────────────────────────────────────────────────

#[test]
fn sequence_sugar_inserts_sequence_cell_group_and_position_markers() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .sequence(5, 2)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<SequenceCell>(entity).is_some(),
        "entity should have SequenceCell marker"
    );
    let group = world
        .get::<SequenceGroup>(entity)
        .expect("entity should have SequenceGroup");
    assert_eq!(group.0, 5);
    let position = world
        .get::<SequencePosition>(entity)
        .expect("entity should have SequencePosition");
    assert_eq!(position.0, 2);
    assert!(
        world.get::<SequenceActive>(entity).is_none(),
        "activation is the job of init_sequence_groups — not spawn"
    );
}

// ── Behavior 1 edge: .sequence(0, 0) is valid — zero is a real group/position
#[test]
fn sequence_sugar_zero_group_zero_position_is_valid() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .sequence(0, 0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SequenceCell>(entity).is_some());
    let group = world
        .get::<SequenceGroup>(entity)
        .expect("SequenceGroup(0) is a valid group id");
    assert_eq!(group.0, 0);
    let position = world
        .get::<SequencePosition>(entity)
        .expect("SequencePosition(0) is a valid position");
    assert_eq!(position.0, 0);
    assert!(
        world.get::<SequenceActive>(entity).is_none(),
        "SequenceActive is inserted by init_sequence_groups, not by the builder"
    );
}

// ── Behavior 2 ─────────────────────────────────────────────────────────────

#[test]
fn cell_without_sequence_sugar_has_no_sequence_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SequenceCell>(entity).is_none());
    assert!(world.get::<SequenceGroup>(entity).is_none());
    assert!(world.get::<SequencePosition>(entity).is_none());
    assert!(world.get::<SequenceActive>(entity).is_none());

    // Guard: prove the builder actually ran so this test does not false-pass
    // against a no-op stub further down the chain.
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!((hp.current - 20.0).abs() < f32::EPSILON);
}

// ── Behavior 3 ─────────────────────────────────────────────────────────────

#[test]
fn sequence_through_cell_behavior_on_definition_inserts_markers() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Sequence {
        group:    7,
        position: 3,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SequenceCell>(entity).is_some());
    let group = world
        .get::<SequenceGroup>(entity)
        .expect("definition-sourced Sequence should insert SequenceGroup");
    assert_eq!(group.0, 7);
    let position = world
        .get::<SequencePosition>(entity)
        .expect("definition-sourced Sequence should insert SequencePosition");
    assert_eq!(position.0, 3);
    assert!(world.get::<SequenceActive>(entity).is_none());
}

// ── Behavior 3 edge: two Sequence entries in one definition — last write wins
#[test]
fn duplicate_sequence_behaviors_on_definition_overwrite_group_and_position() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![
        CellBehavior::Sequence {
            group:    1,
            position: 0,
        },
        CellBehavior::Sequence {
            group:    2,
            position: 5,
        },
    ]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    // Both match-arm iterations run via `entity.insert(...)`, so the second
    // iteration overwrites the first. This is the natural
    // append-without-dedup semantic already used by the other behaviors.
    let group = world
        .get::<SequenceGroup>(entity)
        .expect("entity should have SequenceGroup");
    assert_eq!(
        group.0, 2,
        "second CellBehavior::Sequence must overwrite the first"
    );
    let position = world
        .get::<SequencePosition>(entity)
        .expect("entity should have SequencePosition");
    assert_eq!(
        position.0, 5,
        "second CellBehavior::Sequence must overwrite the first"
    );
    assert!(world.get::<SequenceCell>(entity).is_some());
}

// ── Behavior 4 ─────────────────────────────────────────────────────────────

#[test]
fn sequence_and_volatile_on_same_cell_insert_both_markers_and_one_bound_effect() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .volatile(25.0, 40.0)
            .sequence(3, 1)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<VolatileCell>(entity).is_some());
    assert!(world.get::<SequenceCell>(entity).is_some());
    let group = world
        .get::<SequenceGroup>(entity)
        .expect("entity should have SequenceGroup");
    assert_eq!(group.0, 3);
    let position = world
        .get::<SequencePosition>(entity)
        .expect("entity should have SequencePosition");
    assert_eq!(position.0, 1);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("volatile should stamp BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "only volatile should contribute a BoundEffects entry (sequence does not stamp)"
    );

    assert!(world.get::<SequenceActive>(entity).is_none());
}

// ── Behavior 4 edge: reversed order produces the same component set
#[test]
fn sequence_then_volatile_produces_same_components_and_bound_effects_as_volatile_then_sequence() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .sequence(3, 1)
            .volatile(25.0, 40.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<VolatileCell>(entity).is_some());
    assert!(world.get::<SequenceCell>(entity).is_some());
    let group = world
        .get::<SequenceGroup>(entity)
        .expect("entity should have SequenceGroup");
    assert_eq!(group.0, 3);
    let position = world
        .get::<SequencePosition>(entity)
        .expect("entity should have SequencePosition");
    assert_eq!(position.0, 1);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("volatile should stamp BoundEffects regardless of order");
    assert_eq!(bound.0.len(), 1);

    assert!(world.get::<SequenceActive>(entity).is_none());
}
