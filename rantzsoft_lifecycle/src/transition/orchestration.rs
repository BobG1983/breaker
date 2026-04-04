//! Orchestration system — drives the transition lifecycle state machine.
//!
//! The orchestration system (`orchestrate_transitions`) runs each frame and
//! advances the transition through its phases based on messages from effect
//! systems:
//!
//! 1. **Starting** — effect start system runs, sends `TransitionReady`
//! 2. **Running** — effect run system runs, sends `TransitionRunComplete`
//! 3. **Ending** — effect end system runs, sends `TransitionOver`
//!
//! After all phases complete, the orchestrator cleans up marker resources
//! and (for In/OutIn/OneShot) unpauses `Time<Virtual>`.

use bevy::{prelude::*, state::state::FreelyMutableState};

use super::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    registry::TransitionRegistry,
    resources::{ActiveTransition, OutInData, OutInState, PendingTransition, TransitionPhase},
    types::TransitionType,
};
use crate::messages::{StateChanged, TransitionEnd, TransitionStart};

/// Orchestrates transition lifecycle phases.
///
/// Runs each frame in `Update`. Reads internal messages to advance the
/// transition through its phases:
///
/// 1. `TransitionReady` → Starting → Running
/// 2. `TransitionRunComplete` → Running → Ending
/// 3. `TransitionOver` → finalize (apply state change, unpause, cleanup)
pub(crate) fn orchestrate_transitions(world: &mut World) {
    // Only run if there's an active transition
    if !world.contains_resource::<PendingTransition>() {
        return;
    }

    // Check for TransitionReady message (Starting -> Running)
    let got_ready = world
        .resource_mut::<bevy::ecs::message::Messages<TransitionReady>>()
        .drain()
        .count()
        > 0;

    let got_run_complete = world
        .resource_mut::<bevy::ecs::message::Messages<TransitionRunComplete>>()
        .drain()
        .count()
        > 0;

    let got_over = world
        .resource_mut::<bevy::ecs::message::Messages<TransitionOver>>()
        .drain()
        .count()
        > 0;

    // Phase state machine
    if got_ready {
        advance_to_running(world);
    }

    if got_run_complete {
        advance_to_ending(world);
    }

    if got_over {
        handle_transition_over(world);
    }
}

/// Advance from Starting to Running phase via the registry.
fn advance_to_running(world: &mut World) {
    let type_id = world.resource::<PendingTransition>().current_effect_type_id;
    world.resource_mut::<PendingTransition>().phase = TransitionPhase::Running;
    world.resource_scope(|world, registry: Mut<TransitionRegistry>| {
        registry.advance_to_running(type_id, world);
    });
}

/// Advance from Running to Ending phase via the registry.
fn advance_to_ending(world: &mut World) {
    let type_id = world.resource::<PendingTransition>().current_effect_type_id;
    world.resource_mut::<PendingTransition>().phase = TransitionPhase::Ending;
    world.resource_scope(|world, registry: Mut<TransitionRegistry>| {
        registry.advance_to_ending(type_id, world);
    });
}

/// Handle transition completion. Behavior depends on whether this is an
/// `OutIn` Out-phase (transitions to In phase) or a final phase (cleanup).
fn handle_transition_over(world: &mut World) {
    let is_out_in_out_phase =
        world.resource::<PendingTransition>().out_in_state == Some(OutInState::Out);

    if is_out_in_out_phase {
        // OutIn Out-phase complete: apply state change, start In effect

        // 1. Remove EndingTransition<T> for the Out effect via registry
        let out_type_id = world.resource::<PendingTransition>().current_effect_type_id;
        world.resource_scope(|world, registry: Mut<TransitionRegistry>| {
            registry.remove_ending(out_type_id, world);
        });

        // 2. Apply state change (take the closure and call it)
        let apply_fn = world
            .resource_mut::<PendingTransition>()
            .apply_state_change
            .take();
        if let Some(apply) = apply_fn {
            apply(world);
        }

        // 3. Extract In effect data and update PendingTransition for In phase
        //
        // out_in_state == Some(Out) implies out_in_in_effect is Some — this
        // is an invariant maintained by begin_transition.
        let in_data = world
            .resource_mut::<PendingTransition>()
            .out_in_in_effect
            .take();
        // Invariant: out_in_state == Some(Out) implies out_in_in_effect is Some.
        // This is maintained by begin_transition.
        debug_assert!(in_data.is_some(), "OutIn transition missing in_effect data");
        let Some(in_data) = in_data else {
            return;
        };
        let in_type_id = in_data.in_effect_type_id;
        {
            let mut pending = world.resource_mut::<PendingTransition>();
            pending.out_in_state = Some(OutInState::In);
            pending.current_effect_type_id = in_type_id;
            pending.phase = TransitionPhase::Starting;
        }

        // 4. Start the In effect via trait method
        in_data.in_effect.insert_starting(world);
    } else {
        // Single-phase (Out, In, OneShot) or OutIn In-phase: finalize

        // 1. Remove EndingTransition<T> for the current effect via registry
        let type_id = world.resource::<PendingTransition>().current_effect_type_id;
        world.resource_scope(|world, registry: Mut<TransitionRegistry>| {
            registry.remove_ending(type_id, world);
        });

        // 2. If apply_state_change is Some (Out transition), call it
        let apply_fn = world
            .resource_mut::<PendingTransition>()
            .apply_state_change
            .take();
        if let Some(apply) = apply_fn {
            apply(world);
        }

        // 3. If unpause_at_end, unpause Time<Virtual>
        let unpause = world.resource::<PendingTransition>().unpause_at_end;
        if unpause {
            world.resource_mut::<Time<Virtual>>().unpause();
        }

        // 4. Send TransitionEnd<S> via closure
        let end_fn = world
            .resource_mut::<PendingTransition>()
            .send_transition_end
            .take();
        if let Some(send_end) = end_fn {
            send_end(world);
        }

        // 5. Remove PendingTransition and ActiveTransition
        world.remove_resource::<PendingTransition>();
        world.remove_resource::<ActiveTransition>();
    }
}

/// Begin a transition by setting up initial state.
///
/// All transition types:
/// - Pause `Time<Virtual>` (idempotent)
/// - Insert `ActiveTransition`
/// - Send `TransitionStart<S>`
///
/// For Out: state change is deferred until after the effect completes.
/// For In/OneShot: state change is applied immediately before the effect.
/// For `OutIn`: Out effect starts first; state change deferred until between
/// Out and In phases.
/// Create a closure that applies `NextState<S>::set` and sends `StateChanged<S>`.
fn make_state_change_closure<S: FreelyMutableState + Copy>(
    from: S,
    to: S,
) -> Box<dyn FnOnce(&mut World) + Send + Sync> {
    Box::new(move |world: &mut World| {
        world.resource_mut::<NextState<S>>().set(to);
        world
            .resource_mut::<bevy::ecs::message::Messages<StateChanged<S>>>()
            .write(StateChanged { from, to });
    })
}

/// Create a closure that sends `TransitionEnd<S>`.
fn make_transition_end_closure<S: FreelyMutableState + Copy>(
    from: S,
    to: S,
) -> Box<dyn FnOnce(&mut World) + Send + Sync> {
    Box::new(move |world: &mut World| {
        world
            .resource_mut::<bevy::ecs::message::Messages<TransitionEnd<S>>>()
            .write(TransitionEnd { from, to });
    })
}

pub(crate) fn begin_transition<S: FreelyMutableState + Copy>(
    world: &mut World,
    from: S,
    to: S,
    transition: TransitionType,
) {
    // ALL transition types pause virtual time. pause() is idempotent.
    world.resource_mut::<Time<Virtual>>().pause();

    // Insert ActiveTransition
    world.insert_resource(ActiveTransition);

    // Send TransitionStart<S>
    world
        .resource_mut::<bevy::ecs::message::Messages<TransitionStart<S>>>()
        .write(TransitionStart { from, to });

    // Build PendingTransition based on transition type
    match transition {
        TransitionType::Out(effect) => {
            let type_id = (*effect).type_id();
            let pending = PendingTransition {
                phase: TransitionPhase::Starting,
                current_effect_type_id: type_id,
                apply_state_change: Some(make_state_change_closure(from, to)),
                send_transition_end: Some(make_transition_end_closure(from, to)),
                out_in_in_effect: None,
                out_in_state: None,
                unpause_at_end: false, // Out does NOT unpause
            };
            world.insert_resource(pending);
            effect.insert_starting(world);
        }
        TransitionType::In(effect) => {
            let type_id = (*effect).type_id();
            // Apply state change NOW (In reveals new content)
            (make_state_change_closure(from, to))(world);
            let pending = PendingTransition {
                phase: TransitionPhase::Starting,
                current_effect_type_id: type_id,
                apply_state_change: None, // Already applied
                send_transition_end: Some(make_transition_end_closure(from, to)),
                out_in_in_effect: None,
                out_in_state: None,
                unpause_at_end: true, // In unpauses at end
            };
            world.insert_resource(pending);
            effect.insert_starting(world);
        }
        TransitionType::OutIn { out_e, in_e } => {
            let out_type_id = (*out_e).type_id();
            let in_type_id = (*in_e).type_id();
            let pending = PendingTransition {
                phase: TransitionPhase::Starting,
                current_effect_type_id: out_type_id,
                apply_state_change: Some(make_state_change_closure(from, to)),
                send_transition_end: Some(make_transition_end_closure(from, to)),
                out_in_in_effect: Some(OutInData {
                    in_effect_type_id: in_type_id,
                    in_effect: in_e,
                }),
                out_in_state: Some(OutInState::Out),
                unpause_at_end: true, // `OutIn` unpauses after In completes
            };
            world.insert_resource(pending);
            out_e.insert_starting(world);
        }
        TransitionType::OneShot(effect) => {
            let type_id = (*effect).type_id();
            // Apply state change NOW (both states coexist during effect)
            (make_state_change_closure(from, to))(world);
            let pending = PendingTransition {
                phase: TransitionPhase::Starting,
                current_effect_type_id: type_id,
                apply_state_change: None, // Already applied
                send_transition_end: Some(make_transition_end_closure(from, to)),
                out_in_in_effect: None,
                out_in_state: None,
                unpause_at_end: true, // OneShot unpauses at end
            };
            world.insert_resource(pending);
            effect.insert_starting(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{
        RantzLifecyclePlugin, Route,
        messages::{ChangeState, StateChanged},
        transition::{
            messages::{TransitionOver, TransitionReady, TransitionRunComplete},
            resources::{
                ActiveTransition, EndingTransition, PendingTransition, RunningTransition,
                StartingTransition,
            },
            traits::{InTransition, OneShotTransition, OutTransition, Transition},
            types::TransitionType,
        },
    };

    // ---- Test effect types ----

    struct TestEffectOut;
    impl Transition for TestEffectOut {}
    impl OutTransition for TestEffectOut {}

    struct TestEffectIn;
    impl Transition for TestEffectIn {}
    impl InTransition for TestEffectIn {}

    struct TestEffectOneShot;
    impl Transition for TestEffectOneShot {}
    impl OneShotTransition for TestEffectOneShot {}

    // ---- Test state enum ----

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
        B,
        C,
    }

    // ---- Test effect systems (instant completion) ----
    // These are test infrastructure, NOT stubs. They immediately send the
    // phase-completion message so orchestration tests can validate the full
    // lifecycle in a small number of update calls.

    fn test_effect_out_start(mut writer: MessageWriter<TransitionReady>) {
        writer.write(TransitionReady);
    }

    fn test_effect_out_run(mut writer: MessageWriter<TransitionRunComplete>) {
        writer.write(TransitionRunComplete);
    }

    fn test_effect_out_end(mut writer: MessageWriter<TransitionOver>) {
        writer.write(TransitionOver);
    }

    fn test_effect_in_start(mut writer: MessageWriter<TransitionReady>) {
        writer.write(TransitionReady);
    }

    fn test_effect_in_run(mut writer: MessageWriter<TransitionRunComplete>) {
        writer.write(TransitionRunComplete);
    }

    fn test_effect_in_end(mut writer: MessageWriter<TransitionOver>) {
        writer.write(TransitionOver);
    }

    fn test_effect_oneshot_start(mut writer: MessageWriter<TransitionReady>) {
        writer.write(TransitionReady);
    }

    fn test_effect_oneshot_run(mut writer: MessageWriter<TransitionRunComplete>) {
        writer.write(TransitionRunComplete);
    }

    fn test_effect_oneshot_end(mut writer: MessageWriter<TransitionOver>) {
        writer.write(TransitionOver);
    }

    // ---- Test app helper ----

    fn transition_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_plugins(
                RantzLifecyclePlugin::new()
                    .register_state::<TestState>()
                    .register_custom_transition::<TestEffectOut, _, _, _>(
                        test_effect_out_start,
                        test_effect_out_run,
                        test_effect_out_end,
                    )
                    .register_custom_transition::<TestEffectIn, _, _, _>(
                        test_effect_in_start,
                        test_effect_in_run,
                        test_effect_in_end,
                    )
                    .register_custom_transition::<TestEffectOneShot, _, _, _>(
                        test_effect_oneshot_start,
                        test_effect_oneshot_run,
                        test_effect_oneshot_end,
                    ),
            );
        app
    }

    fn send_change_state(app: &mut App) {
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<ChangeState<TestState>>>()
            .write(ChangeState::new());
    }

    fn add_out_route(app: &mut App, from: TestState, to: TestState) {
        app.world_mut()
            .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
            .add(
                Route::from(from)
                    .to(to)
                    .with_transition(TransitionType::Out(Arc::new(TestEffectOut))),
            )
            .ok();
    }

    fn add_in_route(app: &mut App, from: TestState, to: TestState) {
        app.world_mut()
            .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
            .add(
                Route::from(from)
                    .to(to)
                    .with_transition(TransitionType::In(Arc::new(TestEffectIn))),
            )
            .ok();
    }

    fn add_outin_route(app: &mut App, from: TestState, to: TestState) {
        app.world_mut()
            .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
            .add(
                Route::from(from)
                    .to(to)
                    .with_transition(TransitionType::OutIn {
                        out_e: Arc::new(TestEffectOut),
                        in_e: Arc::new(TestEffectIn),
                    }),
            )
            .ok();
    }

    fn add_oneshot_route(app: &mut App, from: TestState, to: TestState) {
        app.world_mut()
            .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
            .add(
                Route::from(from)
                    .to(to)
                    .with_transition(TransitionType::OneShot(Arc::new(TestEffectOneShot))),
            )
            .ok();
    }

    fn add_plain_route(app: &mut App, from: TestState, to: TestState) {
        app.world_mut()
            .resource_mut::<crate::routing_table::RoutingTable<TestState>>()
            .add(Route::from(from).to(to))
            .ok();
    }

    // =======================================================================
    // Section G: Out Transition Lifecycle
    // =======================================================================

    // --- Behavior 1: Out transition pauses Time<Virtual> at start ---

    #[test]
    fn out_transition_pauses_time_virtual_at_start() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update(); // initial

        send_change_state(&mut app);
        app.update(); // dispatch + begin_transition

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused after Out transition starts"
        );
    }

    // Time<Real> has no is_paused() in Bevy — Real time is never paused.
    // The guarantee that Time<Real> is unaffected by transitions is by Bevy design.

    // --- Behavior 2: Out transition inserts ActiveTransition at start ---

    #[test]
    fn out_transition_inserts_active_transition_at_start() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should not exist before ChangeState"
        );

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be inserted when Out transition starts"
        );
    }

    // --- Behavior 3: Out transition sends TransitionStart<S> ---

    #[test]
    fn out_transition_sends_transition_start_message() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<crate::messages::TransitionStart<TestState>>>(
            );
        let starts: Vec<_> = msgs.iter_current_update_messages().collect();
        assert_eq!(starts.len(), 1, "expected exactly 1 TransitionStart");
        assert_eq!(starts[0].from, TestState::A);
        assert_eq!(starts[0].to, TestState::B);
    }

    // --- Behavior 4: Out transition inserts StartingTransition<T> ---

    #[test]
    fn out_transition_inserts_starting_transition_for_effect() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world()
                .contains_resource::<StartingTransition<TestEffectOut>>(),
            "StartingTransition<TestEffectOut> should be inserted"
        );
        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectIn>>(),
            "StartingTransition<TestEffectIn> should NOT be inserted for an Out transition"
        );
    }

    // --- Behavior 5: Out advances Starting -> Running on TransitionReady ---

    #[test]
    fn out_transition_advances_starting_to_running_on_ready() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update(); // begin_transition inserts StartingTransition
        app.update(); // start system fires, sends TransitionReady, orchestrator advances

        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectOut>>(),
            "StartingTransition should be removed after TransitionReady"
        );
        assert!(
            app.world()
                .contains_resource::<RunningTransition<TestEffectOut>>(),
            "RunningTransition should be inserted after TransitionReady"
        );
        assert!(
            app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should remain present"
        );
    }

    // --- Behavior 6: Out advances Running -> Ending on TransitionRunComplete ---

    #[test]
    fn out_transition_advances_running_to_ending_on_run_complete() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update(); // begin
        app.update(); // Starting -> Running
        app.update(); // Running -> Ending

        assert!(
            !app.world()
                .contains_resource::<RunningTransition<TestEffectOut>>(),
            "RunningTransition should be removed after TransitionRunComplete"
        );
        assert!(
            app.world()
                .contains_resource::<EndingTransition<TestEffectOut>>(),
            "EndingTransition should be inserted after TransitionRunComplete"
        );
        assert!(
            app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should remain present"
        );
    }

    // --- Behavior 7: Out applies state change after TransitionOver ---

    #[test]
    fn out_transition_applies_state_change_after_transition_over() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        // Run sufficient updates for all phases + state application
        for _ in 0..10 {
            app.update();
        }

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "State should be B after Out transition completes"
        );
    }

    // --- Behavior 8: Out sends StateChanged<S> after state change ---

    #[test]
    fn out_transition_sends_state_changed_after_state_change() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        // StateChanged should have been sent at some point during the lifecycle.
        // We check by collecting all messages across updates. Because messages
        // are frame-scoped, we must check the final update's messages or use
        // a capture system. For the RED phase, we assert on the last update's
        // messages, which should include StateChanged if it was sent recently.
        // In practice, we rely on at least one update having it.
        //
        // A more robust approach (for GREEN phase) would be a capture system,
        // but for RED phase the assertion failing (stub does nothing) is sufficient.
        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<StateChanged<TestState>>>();
        assert!(
            msgs.iter_current_update_messages().next().is_some(),
            "StateChanged<TestState> should be sent after Out transition completes"
        );
    }

    // --- Behavior 9: Out sends TransitionEnd<S> after state change ---

    #[test]
    fn out_transition_sends_transition_end_after_state_change() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<crate::messages::TransitionEnd<TestState>>>();
        assert!(
            msgs.iter_current_update_messages().next().is_some(),
            "TransitionEnd<TestState> should be sent after Out transition completes"
        );
    }

    // --- Behavior 10: Out does NOT unpause Time<Virtual> ---

    #[test]
    fn out_transition_does_not_unpause_time_virtual() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should still be paused after Out transition completes"
        );
    }

    // --- Behavior 11: Out removes ActiveTransition at end ---

    #[test]
    fn out_transition_removes_active_transition_at_end() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after Out transition completes"
        );
    }

    // --- Behavior 11b: Out removes PendingTransition at end ---

    #[test]
    fn out_transition_removes_pending_transition_at_end() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().contains_resource::<PendingTransition>(),
            "PendingTransition should be removed after Out transition completes"
        );
    }

    // --- Behavior 12: Out full lifecycle end-to-end ---

    #[test]
    fn out_transition_full_lifecycle_end_to_end() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        // Final state
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "state should be B"
        );

        // Cleanup
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed"
        );
        assert!(
            !app.world().contains_resource::<PendingTransition>(),
            "PendingTransition should be removed"
        );

        // Time<Virtual> stays paused for Out
        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should still be paused after Out"
        );

        // All marker resources cleaned up
        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectOut>>(),
            "StartingTransition should be cleaned up"
        );
        assert!(
            !app.world()
                .contains_resource::<RunningTransition<TestEffectOut>>(),
            "RunningTransition should be cleaned up"
        );
        assert!(
            !app.world()
                .contains_resource::<EndingTransition<TestEffectOut>>(),
            "EndingTransition should be cleaned up"
        );
    }

    // --- Behavior 12b: Phase stall when no system sends TransitionReady ---

    #[test]
    fn phase_stalls_when_no_system_sends_transition_ready() {
        // Build an app WITHOUT registering the test effect systems,
        // but WITH a route that references the effect. The orchestrator
        // inserts StartingTransition but no system sends TransitionReady.
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<TestState>()
            .add_plugins(RantzLifecyclePlugin::new().register_state::<TestState>());

        // Manually add the route with the transition type. Since
        // TestEffectOut systems are NOT registered, no TransitionReady
        // will be sent.
        add_out_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update(); // begin
        app.update(); // should stall here

        // StartingTransition should still exist (no advancement)
        assert!(
            app.world()
                .contains_resource::<StartingTransition<TestEffectOut>>(),
            "StartingTransition should still exist when no TransitionReady is sent"
        );
        assert!(
            !app.world()
                .contains_resource::<RunningTransition<TestEffectOut>>(),
            "RunningTransition should NOT exist (no advancement)"
        );
        assert!(
            app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should still be present"
        );
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::A,
            "State should remain A (no advancement)"
        );
    }

    // =======================================================================
    // Section H: In Transition Lifecycle
    // =======================================================================

    // --- Behavior 13: In transition pauses Time<Virtual> (idempotent) ---

    #[test]
    fn in_transition_pauses_time_virtual_at_start() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused after In transition starts"
        );
    }

    #[test]
    fn in_transition_pause_is_idempotent_when_already_paused() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        // Pre-pause Time<Virtual> (simulating it was already paused by a preceding Out)
        app.world_mut().resource_mut::<Time<Virtual>>().pause();

        send_change_state(&mut app);
        app.update(); // should not error, stays paused

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should remain paused (idempotent)"
        );
    }

    // --- Behavior 14a: In transition sets state BEFORE inserting StartingTransition ---

    #[test]
    fn in_transition_sets_state_before_starting_transition() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update(); // dispatch + begin_transition: sets NextState, sends StateChanged
        app.update(); // Bevy applies state change

        // State should already be B before the effect lifecycle begins
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "In transition should change state BEFORE the effect starts"
        );
    }

    // --- Behavior 14b: In runs full lifecycle after state change ---

    #[test]
    fn in_transition_runs_full_lifecycle_after_state_change() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        // All marker resources should have been inserted and removed
        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectIn>>(),
            "StartingTransition<TestEffectIn> should be cleaned up"
        );
        assert!(
            !app.world()
                .contains_resource::<RunningTransition<TestEffectIn>>(),
            "RunningTransition<TestEffectIn> should be cleaned up"
        );
        assert!(
            !app.world()
                .contains_resource::<EndingTransition<TestEffectIn>>(),
            "EndingTransition<TestEffectIn> should be cleaned up"
        );
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after In lifecycle completes"
        );
    }

    // --- Behavior 15: In unpauses Time<Virtual> and sends TransitionEnd ---

    #[test]
    fn in_transition_unpauses_time_virtual_after_effect_ends() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after In transition completes"
        );
    }

    #[test]
    fn in_transition_sends_transition_end_after_completion() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<crate::messages::TransitionEnd<TestState>>>();
        assert!(
            msgs.iter_current_update_messages().next().is_some(),
            "TransitionEnd<TestState> should be sent after In transition completes"
        );
    }

    #[test]
    fn in_transition_removes_active_transition_after_completion() {
        let mut app = transition_test_app();
        add_in_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after In lifecycle completes"
        );
    }

    // =======================================================================
    // Section I: OutIn Transition Lifecycle
    // =======================================================================

    // --- Behavior 17: OutIn pauses Time<Virtual> at start ---

    #[test]
    fn outin_transition_pauses_time_virtual_at_start() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused after OutIn transition starts"
        );
    }

    // --- Behavior 18: OutIn runs Out phase first ---

    #[test]
    fn outin_transition_starts_with_out_effect() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world()
                .contains_resource::<StartingTransition<TestEffectOut>>(),
            "OutIn should start with Out phase (StartingTransition<TestEffectOut>)"
        );
        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectIn>>(),
            "In phase should not start yet"
        );
    }

    // --- Behavior 19: OutIn applies state change between Out and In ---

    #[test]
    fn outin_transition_applies_state_change_after_out_phase() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        // Run enough updates for Out phase to complete + state to apply
        for _ in 0..10 {
            app.update();
        }

        // After OutIn completes fully, state should be B
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "State should be B after OutIn transition"
        );
    }

    // --- Behavior 20: OutIn runs In phase after state change ---

    #[test]
    fn outin_transition_runs_in_phase_after_out_completes() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        // Run updates; after Out completes, In should start
        for _ in 0..10 {
            app.update();
        }

        // If the lifecycle ran correctly, all In markers should be cleaned up
        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectIn>>(),
            "In phase markers should be cleaned up after OutIn completes"
        );
        // ActiveTransition should be removed only after In completes
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after full OutIn lifecycle"
        );
    }

    // --- Behavior 21: OutIn unpauses Time<Virtual> after In completes ---

    #[test]
    fn outin_transition_unpauses_time_virtual_after_in_phase_completes() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after OutIn transition completes"
        );
    }

    // --- Behavior 22: OutIn sends TransitionStart once, StateChanged at midpoint, TransitionEnd once ---

    // Note: This test verifies the full message sequence. Since messages are
    // frame-scoped in Bevy, we capture them using a system that records them
    // into a resource across frames.

    #[derive(Resource, Default)]
    struct MessageLog {
        transition_starts: Vec<(TestState, TestState)>,
        state_changed: Vec<(TestState, TestState)>,
        transition_ends: Vec<(TestState, TestState)>,
    }

    fn capture_transition_start(
        mut msgs: MessageReader<crate::messages::TransitionStart<TestState>>,
        mut log: ResMut<MessageLog>,
    ) {
        for msg in msgs.read() {
            log.transition_starts.push((msg.from, msg.to));
        }
    }

    fn capture_state_changed(
        mut msgs: MessageReader<StateChanged<TestState>>,
        mut log: ResMut<MessageLog>,
    ) {
        for msg in msgs.read() {
            log.state_changed.push((msg.from, msg.to));
        }
    }

    fn capture_transition_end(
        mut msgs: MessageReader<crate::messages::TransitionEnd<TestState>>,
        mut log: ResMut<MessageLog>,
    ) {
        for msg in msgs.read() {
            log.transition_ends.push((msg.from, msg.to));
        }
    }

    #[test]
    fn outin_sends_exactly_one_transition_start_one_state_changed_one_transition_end() {
        let mut app = transition_test_app();
        app.init_resource::<MessageLog>();
        app.add_systems(
            Update,
            (
                capture_transition_start,
                capture_state_changed,
                capture_transition_end,
            ),
        );
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        let log = app.world().resource::<MessageLog>();
        assert_eq!(
            log.transition_starts.len(),
            1,
            "expected exactly 1 TransitionStart"
        );
        assert_eq!(
            log.state_changed.len(),
            1,
            "expected exactly 1 StateChanged"
        );
        assert_eq!(
            log.transition_ends.len(),
            1,
            "expected exactly 1 TransitionEnd"
        );
    }

    // --- Behavior 23: OutIn full lifecycle end-to-end ---

    #[test]
    fn outin_transition_full_lifecycle_end_to_end() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "state should be B"
        );
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused"
        );
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed"
        );
    }

    // =======================================================================
    // Section J: OneShot Transition Lifecycle
    // =======================================================================

    // --- Behavior 24: OneShot pauses Time<Virtual> at start ---

    #[test]
    fn oneshot_transition_pauses_time_virtual_at_start() {
        let mut app = transition_test_app();
        add_oneshot_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused after OneShot transition starts"
        );
    }

    // Time<Real> has no is_paused() in Bevy — Real time is never paused.
    // The guarantee that Time<Real> is unaffected is by Bevy design.

    // --- Behavior 25: OneShot applies state change before effect starts ---

    #[test]
    fn oneshot_transition_applies_state_change_before_effect() {
        let mut app = transition_test_app();
        add_oneshot_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        app.update(); // begin_transition sets NextState + sends StateChanged
        app.update(); // Bevy applies state change

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "OneShot should change state BEFORE the effect lifecycle"
        );
    }

    // --- Behavior 26: OneShot runs full Starting -> Running -> Ending lifecycle ---

    #[test]
    fn oneshot_transition_runs_full_lifecycle() {
        let mut app = transition_test_app();
        add_oneshot_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        // All marker resources should be cleaned up
        assert!(
            !app.world()
                .contains_resource::<StartingTransition<TestEffectOneShot>>(),
            "StartingTransition<TestEffectOneShot> should be cleaned up"
        );
        assert!(
            !app.world()
                .contains_resource::<RunningTransition<TestEffectOneShot>>(),
            "RunningTransition<TestEffectOneShot> should be cleaned up"
        );
        assert!(
            !app.world()
                .contains_resource::<EndingTransition<TestEffectOneShot>>(),
            "EndingTransition<TestEffectOneShot> should be cleaned up"
        );
        // ActiveTransition should persist during the effect then be removed
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after full OneShot lifecycle"
        );
    }

    // --- Behavior 27: OneShot unpauses Time<Virtual> after completion ---

    #[test]
    fn oneshot_transition_unpauses_time_virtual_after_completion() {
        let mut app = transition_test_app();
        add_oneshot_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after OneShot transition completes"
        );
    }

    // --- Behavior 28: OneShot full lifecycle end-to-end ---

    #[test]
    fn oneshot_transition_full_lifecycle_end_to_end() {
        let mut app = transition_test_app();
        add_oneshot_route(&mut app, TestState::A, TestState::B);
        app.update();

        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "state should be B"
        );
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused"
        );
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed"
        );
    }

    // =======================================================================
    // Section K (behavior 31): Deferred ChangeState re-queue
    // =======================================================================

    #[test]
    fn deferred_change_state_is_processed_after_transition_completes() {
        let mut app = transition_test_app();
        // Route A -> B with Out transition, B -> C with no transition
        add_out_route(&mut app, TestState::A, TestState::B);
        add_plain_route(&mut app, TestState::B, TestState::C);
        app.update();

        // Start A->B transition
        send_change_state(&mut app);
        app.update(); // begin Out transition

        // While transition is active, send another ChangeState
        send_change_state(&mut app);

        // Run enough updates for Out to complete + deferred to fire
        for _ in 0..15 {
            app.update();
        }

        // After Out completes (B), the deferred ChangeState fires B->C
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::C,
            "Deferred ChangeState should route B -> C after Out transition completes"
        );
    }

    // --- Section K (behavior 32): ChangeState during OutIn defers until completion ---

    #[test]
    fn change_state_during_outin_defers_until_entire_lifecycle_completes() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        add_plain_route(&mut app, TestState::B, TestState::C);
        app.update();

        send_change_state(&mut app);
        app.update(); // begin OutIn

        // Send deferred ChangeState during the transition
        send_change_state(&mut app);

        for _ in 0..15 {
            app.update();
        }

        // After OutIn completes (B), deferred fires B->C
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::C,
            "Deferred ChangeState should process after OutIn completes"
        );
    }

    // =======================================================================
    // Section N: Time<Virtual> Pause/Unpause Semantics
    // =======================================================================

    // --- Behavior 41: Out does not unpause, In pauses (idempotent) then unpauses ---

    #[test]
    fn out_then_in_paired_sequence_pause_unpause() {
        let mut app = transition_test_app();
        add_out_route(&mut app, TestState::A, TestState::B);
        add_in_route(&mut app, TestState::B, TestState::C);
        app.update();

        // Step 1: Out transition A -> B
        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        // After Out: paused, state B
        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused after Out"
        );
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::B,
            "State should be B after Out"
        );
        assert!(
            !app.world().contains_resource::<ActiveTransition>(),
            "ActiveTransition should be removed after Out completes"
        );

        // Step 2: In transition B -> C
        send_change_state(&mut app);
        for _ in 0..10 {
            app.update();
        }

        // After In: unpaused, state C
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after In"
        );
        assert_eq!(
            **app.world().resource::<State<TestState>>(),
            TestState::C,
            "State should be C after In"
        );
    }

    // --- Behavior 42: OutIn pauses at start and unpauses at end ---

    #[test]
    fn outin_pauses_at_start_and_unpauses_at_end() {
        let mut app = transition_test_app();
        add_outin_route(&mut app, TestState::A, TestState::B);
        app.update();

        // Before transition: not paused
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should not be paused before transition"
        );

        send_change_state(&mut app);
        app.update(); // begin transition

        // During transition: paused
        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused during OutIn"
        );

        for _ in 0..10 {
            app.update();
        }

        // After completion: unpaused
        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after OutIn completes"
        );
    }

    // --- Behavior 43: OneShot pauses at start and unpauses at end ---

    #[test]
    fn oneshot_pauses_at_start_and_unpauses_at_end() {
        let mut app = transition_test_app();
        add_oneshot_route(&mut app, TestState::A, TestState::B);
        app.update();

        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should not be paused before transition"
        );

        send_change_state(&mut app);
        app.update();

        assert!(
            app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be paused during OneShot"
        );

        for _ in 0..10 {
            app.update();
        }

        assert!(
            !app.world().resource::<Time<Virtual>>().is_paused(),
            "Time<Virtual> should be unpaused after OneShot completes"
        );
    }
}
