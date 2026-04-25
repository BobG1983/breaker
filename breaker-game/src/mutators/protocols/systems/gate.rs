//! Shared pre-gate drain state for protocol reader systems.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    mutators::protocols::{definition::ProtocolKind, resources::ActiveProtocols},
    prelude::*,
};

/// Shared pre-gate drain state for protocol reader systems.
///
/// Bundles the two resources every retrofitted reader system queries —
/// [`ActiveProtocols`] and `State<NodeState>` — behind a single
/// [`SystemParam`], keeping per-system signatures below the
/// `clippy::too_many_arguments` limit after the pre-gate-drain retrofit
/// added two new params to every reader system.
///
/// Usage:
///
/// ```ignore
/// fn foo_reader(mut reader: MessageReader<FooEvent>, gate: ProtocolGate, ...) {
///     if gate.is_closed_for(ProtocolKind::Foo) {
///         reader.clear();
///         return;
///     }
///     // ...
/// }
/// ```
#[derive(SystemParam)]
pub(crate) struct ProtocolGate<'w> {
    pub(crate) active_protocols: Option<Res<'w, ActiveProtocols>>,
    pub(crate) node_state:       Option<Res<'w, State<NodeState>>>,
}

impl ProtocolGate<'_> {
    /// Returns `true` when either the given protocol is NOT active OR
    /// the node is NOT in [`NodeState::Playing`]. Reader systems should
    /// call `reader.clear(); return;` when this returns `true` so
    /// buffered messages cannot leak retroactively when the protocol
    /// activates on a later frame.
    pub(crate) fn is_closed_for(&self, kind: ProtocolKind) -> bool {
        self.active_protocols
            .as_ref()
            .is_none_or(|ap| !ap.contains(kind))
            || self
                .node_state
                .as_ref()
                .is_none_or(|s| *s.get() != NodeState::Playing)
    }
}
