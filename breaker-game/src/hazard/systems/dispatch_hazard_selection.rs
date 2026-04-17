//! System to dispatch `HazardSelected` messages into `ActiveHazards` via
//! `add_stack(kind)`.

use bevy::prelude::*;

use crate::hazard::{messages::HazardSelected, resources::ActiveHazards};

/// Dispatch selected hazards: increment per-kind stack counts in
/// [`ActiveHazards`].
///
/// Future extension: call per-kind `hazards::activate(kind, tuning,
/// commands)` after the stack increment once per-hazard runtime systems land.
pub(crate) fn dispatch_hazard_selection(
    mut reader: MessageReader<HazardSelected>,
    mut active: ResMut<ActiveHazards>,
) {
    for msg in reader.read() {
        active.add_stack(msg.kind);
    }
}

#[cfg(test)]
mod tests;
