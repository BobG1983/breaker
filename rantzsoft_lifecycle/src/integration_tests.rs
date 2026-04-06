use std::sync::Arc;

use bevy::{prelude::*, state::app::StatesPlugin};

use crate::{
    ChangeState, RantzLifecyclePlugin, Route, RoutingTableAppExt,
    transition::{
        effects::shared::TransitionProgress,
        resources::{ActiveTransition, PendingTransition},
    },
};

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum TestState {
    #[default]
    A,
    B,
}

/// Build a minimal app with the lifecycle plugin and a registered test state.
fn integration_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<TestState>()
        .add_plugins(RantzLifecyclePlugin::new().register_state::<TestState>());
    app
}

/// Drive the app forward until the transition completes, by force-completing
/// the progress each frame. Returns the number of updates run.
fn drive_transition_to_completion(app: &mut App, max_frames: usize) -> usize {
    for frame in 0..max_frames {
        // If there is a TransitionProgress resource, force it past duration
        if app.world().contains_resource::<TransitionProgress>() {
            let mut progress = app.world_mut().resource_mut::<TransitionProgress>();
            if !progress.completed {
                progress.elapsed = progress.duration + 0.1;
            }
        }
        app.update();

        // Check if transition has completed
        if !app.world().contains_resource::<ActiveTransition>()
            && !app.world().contains_resource::<PendingTransition>()
        {
            return frame + 1;
        }
    }
    max_frames
}

// --- Test 1: Route with FadeIn transition completes full lifecycle ---

#[test]
fn route_with_fade_in_transition_completes_full_lifecycle() {
    use crate::FadeIn;

    let mut app = integration_app();

    app.add_route(
        Route::from(TestState::A)
            .to(TestState::B)
            .with_transition(crate::TransitionType::In(Arc::new(FadeIn::default())))
            .when(|_| true),
    );

    // Initial update — state is A, condition should fire
    app.update();

    // Drive transition to completion
    let frames = drive_transition_to_completion(&mut app, 20);
    assert!(frames < 20, "transition should complete within 20 frames");

    // One more update to apply the state change
    app.update();

    // Assert: State<TestState> == B
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "state should be B after transition completes"
    );

    // Assert: ActiveTransition removed
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after transition completes"
    );

    // Assert: Time<Virtual> unpaused
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after transition completes"
    );
}

// --- Test 2: Route with OutIn transition completes full lifecycle ---

#[test]
fn route_with_out_in_transition_completes_full_lifecycle() {
    use crate::{FadeIn, FadeOut};

    let mut app = integration_app();

    app.add_route(
        Route::from(TestState::A)
            .to(TestState::B)
            .with_transition(crate::TransitionType::OutIn {
                out_e: Arc::new(FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
            })
            .when(|_| true),
    );

    // Initial update
    app.update();

    // Drive transition to completion (OutIn has two phases)
    let frames = drive_transition_to_completion(&mut app, 30);
    assert!(
        frames < 30,
        "OutIn transition should complete within 30 frames"
    );

    // One more update to ensure state is applied
    app.update();

    // Assert: State<TestState> == B
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "state should be B after OutIn transition completes"
    );

    // Assert: ActiveTransition removed
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "ActiveTransition should be removed after OutIn transition completes"
    );

    // Assert: Time<Virtual> unpaused
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "Time<Virtual> should be unpaused after OutIn transition completes"
    );
}

// --- Test 3: Plain route (no transition) still changes state ---

#[test]
fn plain_route_without_transition_changes_state() {
    let mut app = integration_app();

    // Route without .with_transition()
    app.add_route(Route::from(TestState::A).to(TestState::B));

    app.update();

    // Trigger via ChangeState message
    app.world_mut()
        .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
        .write(ChangeState::new());
    app.update();
    app.update();

    // Assert: State changed
    assert_eq!(
        **app.world().resource::<State<TestState>>(),
        TestState::B,
        "plain route should still change state"
    );

    // Assert: No ActiveTransition was inserted
    assert!(
        !app.world().contains_resource::<ActiveTransition>(),
        "no ActiveTransition should be inserted for plain routes"
    );

    // Assert: No TransitionStart sent (check current frame messages)
    // Note: messages may have been consumed by now, but ActiveTransition
    // absence confirms no transition was started.
}

// --- Test 4: FixedUpdate ChangeState WITHOUT transitions ---
//
// Same route chain as Test 5 but NO transitions. If this passes and
// Test 5 fails, transitions are the blocker.

#[test]
fn fixed_update_change_state_works_without_transitions() {
    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Parent {
        #[default]
        Setup,
        Node,
    }

    #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[source(Parent = Parent::Node)]
    enum Child {
        #[default]
        Loading,
        AnimateIn,
        Playing,
    }

    fn trigger_child(
        mut writer: MessageWriter<ChangeState<Child>>,
        mut fired: Local<bool>,
        state: Option<Res<State<Child>>>,
    ) {
        if *fired {
            return;
        }
        if let Some(s) = state
            && *s.get() == Child::Loading
        {
            writer.write(ChangeState::new());
            *fired = true;
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Parent>()
        .add_sub_state::<Child>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(20),
        ))
        .add_plugins(
            RantzLifecyclePlugin::new()
                .register_state::<Parent>()
                .register_state::<Child>(),
        );

    // Parent: Setup → Node (condition, NO transition)
    app.add_route(Route::from(Parent::Setup).to(Parent::Node).when(|_| true));

    // Child: Loading → AnimateIn (message-triggered, no transition)
    app.add_route(Route::from(Child::Loading).to(Child::AnimateIn));

    // Child: AnimateIn → Playing (condition, no transition)
    app.add_route(
        Route::from(Child::AnimateIn)
            .to(Child::Playing)
            .when(|_| true),
    );

    app.add_systems(FixedUpdate, trigger_child);

    for _ in 0..20 {
        app.update();
    }

    let child_state = app.world().get_resource::<State<Child>>().map(|s| *s.get());

    assert_eq!(
        child_state,
        Some(Child::Playing),
        "Without transitions, Child must reach Playing"
    );
}

// --- Test 5a: Full 3-level hierarchy with OutIn + In transitions ---
//
// Mirrors the actual game: Grand → Parent → Child
//   Grand: Menu → Run (OutIn FadeOut+FadeIn, condition-triggered)
//   Parent: Setup → Node (FadeIn, condition-triggered)
//   Child: Loading → AnimateIn (message-triggered) → Playing (condition)
//
// FixedUpdate system writes ChangeState<Child> when Child is Loading.

#[test]
fn full_hierarchy_with_outin_and_in_transitions() {
    use crate::FadeIn;

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Grand {
        #[default]
        Menu,
        Run,
    }

    #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[source(Grand = Grand::Run)]
    enum Parent {
        #[default]
        Setup,
        Node,
    }

    #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[source(Parent = Parent::Node)]
    enum Child {
        #[default]
        Loading,
        AnimateIn,
        Playing,
    }

    fn trigger_child(
        mut writer: MessageWriter<ChangeState<Child>>,
        mut fired: Local<bool>,
        state: Option<Res<State<Child>>>,
    ) {
        if *fired {
            return;
        }
        if let Some(s) = state
            && *s.get() == Child::Loading
        {
            writer.write(ChangeState::new());
            *fired = true;
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Grand>()
        .add_sub_state::<Parent>()
        .add_sub_state::<Child>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(20),
        ))
        .add_plugins(
            RantzLifecyclePlugin::new()
                .register_state::<Grand>()
                .register_state::<Parent>()
                .register_state::<Child>(),
        );

    // Grand: Menu → Run with OutIn transition
    app.add_route(
        Route::from(Grand::Menu)
            .to(Grand::Run)
            .with_transition(crate::TransitionType::OutIn {
                out_e: Arc::new(crate::FadeOut::default()),
                in_e: Arc::new(FadeIn::default()),
            })
            .when(|_| true),
    );

    // Parent: Setup → Node with FadeIn
    app.add_route(
        Route::from(Parent::Setup)
            .to(Parent::Node)
            .with_transition(crate::TransitionType::In(Arc::new(FadeIn::default())))
            .when(|_| true),
    );

    // Child: Loading → AnimateIn (message-triggered)
    app.add_route(Route::from(Child::Loading).to(Child::AnimateIn));

    // Child: AnimateIn → Playing (condition)
    app.add_route(
        Route::from(Child::AnimateIn)
            .to(Child::Playing)
            .when(|_| true),
    );

    app.add_systems(FixedUpdate, trigger_child);

    // Drive transitions to completion (OutIn = ~38 frames, In = ~19 frames)
    let transition_frames = drive_transition_to_completion(&mut app, 80);

    // Extra frames for child state chain (needs enough frames for
    // Child transitions to settle — Parent's FadeIn transition also
    // needs to complete before Child routes can dispatch)
    let child_frames = drive_transition_to_completion(&mut app, 80);

    for _ in 0..20 {
        app.update();
    }

    let child_state = app.world().get_resource::<State<Child>>().map(|s| *s.get());

    assert_eq!(
        child_state,
        Some(Child::Playing),
        "Child must reach Playing through full OutIn + In transition chain \
         (transition took {transition_frames} frames, child took {child_frames} frames)"
    );
}

// --- Test 5b: FixedUpdate ChangeState through transition chain ---
//
// Mirrors the game's NodeState lifecycle:
//   Parent: condition-triggered route with FadeIn transition (Setup → Node)
//   Child:  message-triggered route (Loading → AnimateIn)
//           condition pass-through (AnimateIn → Playing)
//
// A system in FixedUpdate writes ChangeState<Child> after the parent
// transition completes. The child must reach Playing.

#[test]
fn fixed_update_change_state_reaches_playing_through_transition_chain() {
    use crate::FadeIn;

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Parent {
        #[default]
        Setup,
        Node,
    }

    #[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[source(Parent = Parent::Node)]
    enum Child {
        #[default]
        Loading,
        AnimateIn,
        Playing,
    }

    // System in FixedUpdate that writes ChangeState<Child> once
    fn trigger_child_transition(
        mut writer: MessageWriter<ChangeState<Child>>,
        mut fired: Local<bool>,
        state: Option<Res<State<Child>>>,
    ) {
        if *fired {
            return;
        }
        if let Some(s) = state
            && *s.get() == Child::Loading
        {
            writer.write(ChangeState::new());
            *fired = true;
        }
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Parent>()
        .add_sub_state::<Child>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(20),
        ))
        .add_plugins(
            RantzLifecyclePlugin::new()
                .register_state::<Parent>()
                .register_state::<Child>(),
        );

    // Parent: Setup → Node with FadeIn transition (condition: always)
    app.add_route(
        Route::from(Parent::Setup)
            .to(Parent::Node)
            .with_transition(crate::TransitionType::In(Arc::new(FadeIn::default())))
            .when(|_| true),
    );

    // Child: Loading → AnimateIn (message-triggered)
    app.add_route(Route::from(Child::Loading).to(Child::AnimateIn));

    // Child: AnimateIn → Playing (condition: always)
    app.add_route(
        Route::from(Child::AnimateIn)
            .to(Child::Playing)
            .when(|_| true),
    );

    app.add_systems(FixedUpdate, trigger_child_transition);

    // Drive until the parent transition completes and child reaches Playing
    let frames = drive_transition_to_completion(&mut app, 30);

    // Give extra frames for child state to advance
    for _ in 0..10 {
        app.update();
    }

    let child_state = app.world().get_resource::<State<Child>>().map(|s| *s.get());

    assert_eq!(
        child_state,
        Some(Child::Playing),
        "Child must reach Playing after parent transition completes \
         and FixedUpdate writes ChangeState<Child> (took {frames} frames for transition)"
    );
}
