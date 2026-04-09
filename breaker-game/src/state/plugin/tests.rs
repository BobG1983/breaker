use bevy::prelude::*;

use super::system::*;
use crate::state::{
    run::resources::{NodeOutcome, NodeResult},
    types::RunState,
};

#[test]
fn plugin_builds() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::asset::AssetPlugin::default(),
        ))
        .add_plugins(StatePlugin)
        .update();
}

// ── Behavior 7a: Quit routes to RunState::Teardown ──────────────────

#[test]
fn resolve_node_next_state_quit_returns_teardown() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::Quit,
        node_index: 0,
        cleared_this_frame: false,
        ..default()
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(
        next,
        RunState::Teardown,
        "NodeResult::Quit should route to RunState::Teardown"
    );
}

#[test]
fn resolve_node_next_state_quit_ignores_node_index_and_cleared_this_frame() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::Quit,
        node_index: 99,
        cleared_this_frame: true,
        ..default()
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(
        next,
        RunState::Teardown,
        "NodeResult::Quit should route to Teardown regardless of node_index or cleared_this_frame"
    );
}

// ── Behavior 7b: InProgress routes to ChipSelect ────────────────────

#[test]
fn resolve_node_next_state_in_progress_returns_chip_select() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::InProgress,
        node_index: 0,
        cleared_this_frame: false,
        ..default()
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::ChipSelect);
}

// ── Behavior 7c: Won routes to RunEnd ───────────────────────────────

#[test]
fn resolve_node_next_state_won_returns_run_end() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::Won,
        node_index: 8,
        cleared_this_frame: false,
        ..default()
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::RunEnd);
}

// ── Behavior 7d: TimerExpired routes to RunEnd ──────────────────────

#[test]
fn resolve_node_next_state_timer_expired_returns_run_end() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::TimerExpired,
        node_index: 3,
        cleared_this_frame: false,
        ..default()
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::RunEnd);
}

// ── Behavior 7e: LivesDepleted routes to RunEnd ─────────────────────

#[test]
fn resolve_node_next_state_lives_depleted_returns_run_end() {
    let mut world = World::new();
    world.insert_resource(NodeOutcome {
        result: NodeResult::LivesDepleted,
        node_index: 1,
        cleared_this_frame: false,
        ..default()
    });

    let next = resolve_node_next_state(&world);
    assert_eq!(next, RunState::RunEnd);
}

// ── Quit teardown chain: MenuState → GameState → AppState ──────────

use crate::state::{
    menu::main::{MainMenuSelection, MenuItem},
    types::{AppState, GameState, MenuState},
};

fn send_test_app_exit(mut writer: MessageWriter<AppExit>) {
    writer.write(AppExit::Success);
}

/// Builds an app with the 3-level state hierarchy and quit-path routes.
fn quit_chain_app() -> App {
    use std::sync::Arc;

    use bevy::state::app::StatesPlugin;
    use rantzsoft_stateflow::{RantzStateflowPlugin, Route, RoutingTableAppExt, TransitionType};

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_sub_state::<MenuState>()
        .add_plugins(
            RantzStateflowPlugin::new()
                .register_state::<AppState>()
                .register_state::<GameState>()
                .register_state::<MenuState>(),
        )
        .insert_resource(MainMenuSelection {
            selected: MenuItem::Quit,
        })
        .add_message::<AppExit>();

    // AppState::Game → Teardown (watches GameState)
    app.add_route(
        Route::from(AppState::Game)
            .to(AppState::Teardown)
            .when(|world| {
                world
                    .get_resource::<State<GameState>>()
                    .is_some_and(|s| *s.get() == GameState::Teardown)
            }),
    );
    // GameState::Loading → Menu (Out FadeOut, matching real game)
    app.add_route(
        Route::from(GameState::Loading)
            .to(GameState::Menu)
            .with_transition(TransitionType::Out(Arc::new(
                rantzsoft_stateflow::FadeOut {
                    duration: 0.6,
                    color: Color::WHITE,
                },
            )))
            .when(|_| true),
    );
    // GameState::Menu → Teardown (TransitionType::None for Quit)
    app.add_route(
        Route::from(GameState::Menu)
            .to(GameState::Teardown)
            .with_transition(TransitionType::None)
            .when(|world| {
                world
                    .get_resource::<State<MenuState>>()
                    .is_some_and(|s| *s.get() == MenuState::Teardown)
            }),
    );
    // MenuState::Loading → Main (In FadeIn, matching real game)
    app.add_route(
        Route::from(MenuState::Loading)
            .to(MenuState::Main)
            .with_transition(TransitionType::In(Arc::new(rantzsoft_stateflow::FadeIn {
                duration: 0.6,
                color: Color::WHITE,
            })))
            .when(|_| true),
    );
    // MenuState::Main → Teardown (message-triggered, TransitionType::None)
    app.add_route(
        Route::from(MenuState::Main)
            .to(MenuState::Teardown)
            .with_transition(TransitionType::None),
    );

    app.add_systems(OnEnter(AppState::Teardown), send_test_app_exit);
    app
}

/// Drives the app to `MenuState::Main`, force-completing any transitions.
fn navigate_to_menu(app: &mut App) {
    use rantzsoft_stateflow::transition::effects::shared::TransitionProgress;

    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();

    for _ in 0..60 {
        if app.world().contains_resource::<TransitionProgress>() {
            let mut progress = app.world_mut().resource_mut::<TransitionProgress>();
            if !progress.completed {
                progress.elapsed = progress.duration + 0.1;
            }
        }
        app.update();
    }
}

#[test]
fn quit_teardown_chain_reaches_app_teardown() {
    use rantzsoft_stateflow::ChangeState;

    let mut app = quit_chain_app();
    navigate_to_menu(&mut app);

    assert_eq!(
        app.world()
            .get_resource::<State<GameState>>()
            .map(|s| *s.get()),
        Some(GameState::Menu),
    );
    assert_eq!(
        app.world()
            .get_resource::<State<MenuState>>()
            .map(|s| *s.get()),
        Some(MenuState::Main),
    );

    // Trigger Quit
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<MenuState>>>()
        .write(ChangeState::new());

    // Chain should complete within a few frames
    let mut reached = false;
    for _ in 0..10 {
        app.update();
        if *app.world().resource::<State<AppState>>().get() == AppState::Teardown {
            reached = true;
            break;
        }
    }

    assert!(reached, "AppState must reach Teardown after Quit");

    let msgs = app
        .world()
        .resource::<bevy::ecs::message::Messages<AppExit>>();
    assert!(
        msgs.iter_current_update_messages().count() > 0,
        "AppExit::Success should be sent on entering AppState::Teardown"
    );
}
