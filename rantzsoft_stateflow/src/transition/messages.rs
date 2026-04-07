//! Internal transition lifecycle messages.
//!
//! These messages are `pub(crate)` — effect systems within the crate send
//! them, and the orchestration system consumes them. Game code cannot send
//! these directly.

use bevy::prelude::*;

/// Sent by effect start systems to signal that the start phase is complete
/// and the transition can advance to the Running phase.
#[derive(Message, Clone)]
pub(crate) struct TransitionReady;

/// Sent by effect run systems to signal that the main run phase is complete
/// and the transition can advance to the Ending phase.
#[derive(Message, Clone)]
pub(crate) struct TransitionRunComplete;

/// Sent by effect end systems to signal that the ending phase is complete
/// and the transition lifecycle can be finalized.
#[derive(Message, Clone)]
pub(crate) struct TransitionOver;

#[cfg(test)]
mod tests {
    use super::*;

    // --- Section D behavior 1: TransitionReady can be written and read ---

    #[test]
    fn transition_ready_can_be_written_and_read_as_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TransitionReady>();

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TransitionReady>>()
            .write(TransitionReady);

        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        let count = msgs.iter_current_update_messages().count();
        assert_eq!(count, 1, "expected exactly 1 TransitionReady message");
    }

    // --- Section D behavior 2: TransitionRunComplete can be written and read ---

    #[test]
    fn transition_run_complete_can_be_written_and_read_as_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TransitionRunComplete>();

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TransitionRunComplete>>()
            .write(TransitionRunComplete);

        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        let count = msgs.iter_current_update_messages().count();
        assert_eq!(count, 1, "expected exactly 1 TransitionRunComplete message");
    }

    #[test]
    fn transition_run_complete_multiple_writes_yield_multiple_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TransitionRunComplete>();

        {
            let mut msgs = app
                .world_mut()
                .resource_mut::<bevy::ecs::message::Messages<TransitionRunComplete>>();
            msgs.write(TransitionRunComplete);
            msgs.write(TransitionRunComplete);
        }

        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        let count = msgs.iter_current_update_messages().count();
        assert_eq!(count, 2, "expected 2 TransitionRunComplete messages");
    }

    // --- Section D behavior 3: TransitionOver can be written and read ---

    #[test]
    fn transition_over_can_be_written_and_read_as_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TransitionOver>();

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<TransitionOver>>()
            .write(TransitionOver);

        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionOver>>();
        let count = msgs.iter_current_update_messages().count();
        assert_eq!(count, 1, "expected exactly 1 TransitionOver message");
    }
}
