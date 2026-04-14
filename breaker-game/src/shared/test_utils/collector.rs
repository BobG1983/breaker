//! Generic message collector resource and supporting systems for test assertions.

use bevy::prelude::*;

// ── MessageCollector ───────────────────────────────────────────────────────

/// Generic message collector resource. Captures messages for test assertions.
#[derive(Resource)]
pub(crate) struct MessageCollector<M: Message>(pub Vec<M>);

impl<M: Message> Default for MessageCollector<M> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<M: Message> MessageCollector<M> {
    /// Manually clears collected messages.
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

// ── Clear and collect systems ──────────────────────────────────────────────

/// Clears the `MessageCollector<M>` at the start of each update cycle.
pub(super) fn clear_messages<M: Message>(mut collector: ResMut<MessageCollector<M>>) {
    collector.0.clear();
}

/// Reads messages from `MessageReader<M>` and pushes clones into `MessageCollector<M>`.
pub(super) fn collect_messages<M: Message + Clone>(
    mut reader: MessageReader<M>,
    mut collector: ResMut<MessageCollector<M>>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

/// Attaches a `MessageCollector<M>` to an existing `App`. Mirrors
/// `TestAppBuilder::with_message_capture`, but operates on a mutable `App`
/// reference — use this when the message has already been registered by a
/// plugin after the builder has been finalized. Idempotent.
pub(crate) fn attach_message_capture<M: Message + Clone>(app: &mut App) {
    if app.world().contains_resource::<MessageCollector<M>>() {
        return;
    }
    app.add_message::<M>();
    app.init_resource::<MessageCollector<M>>();
    app.add_systems(First, clear_messages::<M>);
    app.add_systems(Last, collect_messages::<M>);
}
