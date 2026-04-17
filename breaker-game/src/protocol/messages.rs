//! Protocol domain messages.

use bevy::prelude::*;

use super::definition::ProtocolKind;

/// Sent when the player picks a protocol. Consumed by the protocol dispatch
/// system to stamp the selected protocol's effect tree onto the breaker or
/// activate its custom-system runtime.
#[derive(Message, Clone, Debug)]
pub(crate) struct ProtocolSelected {
    /// The protocol kind the player selected.
    pub(crate) kind: ProtocolKind,
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;

    use super::*;

    // ── Behavior 14: ProtocolSelected is a readable Message ───────────────

    #[test]
    fn protocol_selected_can_be_sent_and_read_via_writer_reader() {
        // Writer system — sends one message.
        fn send_anchor(mut writer: MessageWriter<ProtocolSelected>) {
            writer.write(ProtocolSelected {
                kind: ProtocolKind::Anchor,
            });
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ProtocolSelected>();
        app.add_systems(Update, send_anchor);
        app.update();

        // Read directly from the Messages resource on the next frame to assert
        // exactly one message with the expected kind was observed.
        let messages = app.world().resource::<Messages<ProtocolSelected>>();
        let observed: Vec<ProtocolKind> = messages
            .iter_current_update_messages()
            .map(|m| m.kind)
            .collect();

        assert_eq!(
            observed.len(),
            1,
            "reader should observe exactly one ProtocolSelected message"
        );
        assert_eq!(observed[0], ProtocolKind::Anchor);
    }

    #[test]
    fn protocol_selected_clone_produces_equal_copy() {
        // Proves Clone derive exists. PartialEq is not required on the
        // message struct itself — we compare the `kind` field.
        let original = ProtocolSelected {
            kind: ProtocolKind::Greed,
        };
        let cloned = original.clone();
        assert_eq!(cloned.kind, original.kind);
    }
}
