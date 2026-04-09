use bevy::prelude::*;

use super::{
    super::*,
    helpers::{Counter, Order, first_system, increment, second_system},
};
use crate::state::types::NodeState;

// ════════════════════════════════════════════════════════════════════
// Section M: with_system()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 28: with_system() adds system to schedule ──

#[test]
fn with_system_adds_fixed_update_system() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Counter>()
        .with_system(FixedUpdate, increment)
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Counter>().0,
        1,
        "System added via with_system(FixedUpdate, ...) should run on tick()"
    );
}

#[test]
fn with_system_update_schedule() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Counter>()
        .with_system(Update, increment)
        .build();
    app.update();
    assert_eq!(
        app.world().resource::<Counter>().0,
        1,
        "System added to Update should run on app.update()"
    );
}

// ── Behavior 29: with_system() supports ordering ──

#[test]
fn with_system_ordering_after() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Order>()
        .with_system(
            FixedUpdate,
            (first_system, second_system.after(first_system)),
        )
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Order>().0,
        "first,second",
        "second_system should run after first_system"
    );
}

#[test]
fn with_system_ordering_reversed() {
    let mut app = TestAppBuilder::new()
        .with_resource::<Order>()
        .with_system(
            FixedUpdate,
            (first_system.after(second_system), second_system),
        )
        .build();
    tick(&mut app);
    assert_eq!(
        app.world().resource::<Order>().0,
        "second,first",
        "first_system.after(second_system) should reverse execution order"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section N: build()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 30: build() without state navigation doesn't run systems ──

#[test]
fn build_without_update_does_not_run_systems() {
    let app = TestAppBuilder::new().with_resource::<Counter>().build();
    assert_eq!(
        app.world().resource::<Counter>().0,
        0,
        "No systems should have run — counter should be 0"
    );
}

// ── Behavior 31: build() after state navigation has app in target state ──

#[test]
fn build_after_state_navigation_is_in_target_state_immediately() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    // Immediately after build — no additional app.update() or tick()
    assert_eq!(
        *app.world().resource::<State<NodeState>>().get(),
        NodeState::Playing,
        "After build(), app should already be in NodeState::Playing without extra update"
    );
}
