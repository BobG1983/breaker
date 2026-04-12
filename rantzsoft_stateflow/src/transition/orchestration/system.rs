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

use super::super::{
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
    // None: instant state change — no pause, no overlay, no transition lifecycle
    if matches!(transition, TransitionType::None) {
        (make_state_change_closure(from, to))(world);
        return;
    }

    // ALL non-None transition types pause virtual time. pause() is idempotent.
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
                phase:                  TransitionPhase::Starting,
                current_effect_type_id: type_id,
                apply_state_change:     Some(make_state_change_closure(from, to)),
                send_transition_end:    Some(make_transition_end_closure(from, to)),
                out_in_in_effect:       None,
                out_in_state:           None,
                unpause_at_end:         false, // Out does NOT unpause
            };
            world.insert_resource(pending);
            effect.insert_starting(world);
        }
        TransitionType::In(effect) => {
            let type_id = (*effect).type_id();
            // Apply state change NOW (In reveals new content)
            (make_state_change_closure(from, to))(world);
            let pending = PendingTransition {
                phase:                  TransitionPhase::Starting,
                current_effect_type_id: type_id,
                apply_state_change:     None, // Already applied
                send_transition_end:    Some(make_transition_end_closure(from, to)),
                out_in_in_effect:       None,
                out_in_state:           None,
                unpause_at_end:         true, // In unpauses at end
            };
            world.insert_resource(pending);
            effect.insert_starting(world);
        }
        TransitionType::OutIn { out_e, in_e } => {
            let out_type_id = (*out_e).type_id();
            let in_type_id = (*in_e).type_id();
            let pending = PendingTransition {
                phase:                  TransitionPhase::Starting,
                current_effect_type_id: out_type_id,
                apply_state_change:     Some(make_state_change_closure(from, to)),
                send_transition_end:    Some(make_transition_end_closure(from, to)),
                out_in_in_effect:       Some(OutInData {
                    in_effect_type_id: in_type_id,
                    in_effect:         in_e,
                }),
                out_in_state:           Some(OutInState::Out),
                unpause_at_end:         true, // `OutIn` unpauses after In completes
            };
            world.insert_resource(pending);
            out_e.insert_starting(world);
        }
        TransitionType::OneShot(effect) => {
            let type_id = (*effect).type_id();
            // Apply state change NOW (both states coexist during effect)
            (make_state_change_closure(from, to))(world);
            let pending = PendingTransition {
                phase:                  TransitionPhase::Starting,
                current_effect_type_id: type_id,
                apply_state_change:     None, // Already applied
                send_transition_end:    Some(make_transition_end_closure(from, to)),
                out_in_in_effect:       None,
                out_in_state:           None,
                unpause_at_end:         true, // OneShot unpauses at end
            };
            world.insert_resource(pending);
            effect.insert_starting(world);
        }
        TransitionType::None => unreachable!("handled by early return above"),
    }
}
