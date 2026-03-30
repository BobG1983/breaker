//! Tests for `reverse()`: no-op with or without `EntropyEngineState`.

use bevy::prelude::*;

use super::super::effect::*;

// -- Behavior 19: reverse() is a no-op (entity with state) --

#[test]
fn reverse_with_state_is_noop() {
    let mut world = World::new();
    let entity = world.spawn(EntropyEngineState { cells_destroyed: 5 }).id();

    reverse(entity, "", &mut world);

    let state = world.get::<EntropyEngineState>(entity).unwrap();
    assert_eq!(
        state.cells_destroyed, 5,
        "reverse should not modify cells_destroyed"
    );
}

// -- Behavior 20: reverse() on entity without state is a no-op --

#[test]
fn reverse_without_state_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Should not panic
    reverse(entity, "", &mut world);

    assert!(
        world.get::<EntropyEngineState>(entity).is_none(),
        "no EntropyEngineState should be inserted by reverse"
    );
}
