//! Group F — cross-node persistence and `MenuState::Main` cleanup
//! (Behaviors 24-27).
//!
//! Pins that `FissionCounter` persists across `OnExit(NodeState::Playing)`
//! and across `OnExit(RunState::Node)`; that
//! `OnExit(MenuState::Main)` removes both `FissionConfig` and
//! `FissionCounter`; and that cleanup does NOT fire on
//! `OnExit(NodeState::Playing)`.

use bevy::prelude::*;

use super::{
    super::system::{FissionConfig, FissionCounter, register},
    helpers::{
        build_fission_app, install_fission_counter, seed_active_protocols_with_fission,
        spawn_bolt_at_with_velocity, write_destroyed_cell,
    },
};
use crate::{mutators::protocols::resources::ActiveProtocols, prelude::*};

// ── Behavior 24 — FissionCounter persists across OnExit(NodeState::Playing) ─

#[test]
fn fission_counter_persists_across_node_state_exit() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 5);

    // Exit: Playing → AnimateOut.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    // Re-enter: AnimateOut → Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 5 },
        "counter must persist across NodeState exit/entry; got {counter:?}"
    );
    let cfg = *app
        .world()
        .get_resource::<FissionConfig>()
        .expect("FissionConfig must persist across node transition");
    assert_eq!(
        cfg.kills_per_split, 8,
        "FissionConfig must persist unchanged; got {}",
        cfg.kills_per_split
    );
}

// ── Behavior 24 (edge case) — counter at 7 triggers split in new node ──────-

#[test]
fn counter_at_seven_triggers_split_in_new_node_after_transition() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    // Transition out and back in.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "counter must still be 7 after transition; got {counter:?}"
    );

    // Spawn a bolt and trigger the Nth kill.
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));
    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "subsequent kill must trigger split (counter resets to 0); got {counter:?}"
    );
}

// ── Behavior 25 — FissionCounter persists across OnExit(RunState::Node) ────-

#[test]
fn fission_counter_persists_across_run_state_exit() {
    let mut app = build_fission_app();
    install_fission_counter(&mut app, 4);

    // Transition out of RunState::Node.
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::ChipSelect);
    app.update();

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 4 },
        "counter must persist across RunState exit; got {counter:?}"
    );
}

// ── Behavior 25 (edge case) — round-trip ChipSelect ↔ Node preserves counter

#[test]
fn fission_counter_persists_across_chip_select_round_trip() {
    let mut app = build_fission_app();
    install_fission_counter(&mut app, 4);

    // Node → ChipSelect.
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::ChipSelect);
    app.update();
    // ChipSelect → Node.
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 4 },
        "counter must persist across ChipSelect round-trip; got {counter:?}"
    );
}

// ── Behavior 26 — OnExit(MenuState::Main) removes BOTH resources ───────────-

#[test]
fn on_exit_menu_state_main_removes_fission_config_and_counter() {
    // Fresh app in MenuState::Main with both resources installed.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .build();

    // Drive to MenuState::Main.
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Menu);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Main);
    app.update();

    register(&mut app);
    app.world_mut()
        .insert_resource(FissionConfig { kills_per_split: 8 });
    app.world_mut().insert_resource(FissionCounter { kills: 5 });

    // Trigger OnExit(MenuState::Main).
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Teardown);
    app.update();

    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "FissionConfig must be removed on OnExit(MenuState::Main)"
    );
    assert!(
        app.world().get_resource::<FissionCounter>().is_none(),
        "FissionCounter must be removed on OnExit(MenuState::Main)"
    );
}

// ── Behavior 26 (edge case) — cleanup runs even when Fission NOT active ────-

#[test]
fn cleanup_runs_even_when_fission_not_in_active_protocols() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveProtocols>()
        .with_resource::<FissionCounter>()
        .with_message::<Destroyed<Cell>>()
        .build();

    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Menu);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Main);
    app.update();

    register(&mut app);
    // Do NOT seed Fission in ActiveProtocols.
    app.world_mut()
        .insert_resource(FissionConfig { kills_per_split: 8 });
    app.world_mut().insert_resource(FissionCounter { kills: 5 });

    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Teardown);
    app.update();

    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "cleanup must run unconditionally — FissionConfig removed"
    );
    assert!(
        app.world().get_resource::<FissionCounter>().is_none(),
        "cleanup must run unconditionally — FissionCounter removed"
    );
}

// ── Behavior 26 (edge case) — cleanup is idempotent (no panic on second run)

#[test]
fn cleanup_is_idempotent_when_resources_already_absent() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveProtocols>()
        .with_message::<Destroyed<Cell>>()
        .build();

    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Menu);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Main);
    app.update();

    register(&mut app);
    // Resources intentionally NOT installed.

    // First OnExit(MenuState::Main) — resources already absent.
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Teardown);
    app.update();
    assert!(app.world().get_resource::<FissionConfig>().is_none());
    assert!(app.world().get_resource::<FissionCounter>().is_none());

    // Return to MenuState::Main, then exit again — must still not panic.
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Main);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<MenuState>>()
        .set(MenuState::Teardown);
    app.update();

    // Reaching here without panic is the primary assertion.
    assert!(app.world().get_resource::<FissionConfig>().is_none());
    assert!(app.world().get_resource::<FissionCounter>().is_none());
}

// ── Behavior 27 — cleanup does NOT fire on OnExit(NodeState::Playing) ──────-

#[test]
fn cleanup_does_not_fire_on_node_state_playing_exit() {
    let mut app = build_fission_app();
    install_fission_counter(&mut app, 5);

    // Transition Playing → AnimateOut.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 5 },
        "FissionCounter must NOT be removed on NodeState::Playing exit; got {counter:?}"
    );
    let cfg = *app
        .world()
        .get_resource::<FissionConfig>()
        .expect("FissionConfig must NOT be removed on NodeState::Playing exit");
    assert_eq!(cfg.kills_per_split, 8);
}
