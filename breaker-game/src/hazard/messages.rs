//! Hazard domain messages.

use bevy::prelude::*;

use super::definition::HazardKind;

/// Sent when the player picks a hazard. Consumed by the hazard dispatch
/// system to increment `ActiveHazards` stacks and activate the hazard's
/// runtime.
#[derive(Message, Clone, Debug)]
pub(crate) struct HazardSelected {
    /// The hazard kind the player selected.
    pub(crate) kind: HazardKind,
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;

    use super::*;

    // ── Behavior 26: HazardSelected is a readable Message ─────────────────

    #[test]
    fn hazard_selected_can_be_sent_and_read_via_writer_reader() {
        fn send_drift(mut writer: MessageWriter<HazardSelected>) {
            writer.write(HazardSelected {
                kind: HazardKind::Drift,
            });
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<HazardSelected>();
        app.add_systems(Update, send_drift);
        app.update();

        let messages = app.world().resource::<Messages<HazardSelected>>();
        let observed: Vec<HazardKind> = messages
            .iter_current_update_messages()
            .map(|m| m.kind)
            .collect();

        assert_eq!(
            observed.len(),
            1,
            "reader should observe exactly one HazardSelected message"
        );
        assert_eq!(observed[0], HazardKind::Drift);
    }

    #[test]
    fn hazard_selected_clone_produces_equal_copy() {
        let original = HazardSelected {
            kind: HazardKind::Sympathy,
        };
        let cloned = original.clone();
        assert_eq!(cloned.kind, original.kind);
    }
}
