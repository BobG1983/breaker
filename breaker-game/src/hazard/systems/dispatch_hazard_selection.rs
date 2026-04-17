//! System to dispatch `HazardSelected` messages into `ActiveHazards` via
//! `add_stack(kind)` and delegate to per-hazard `activate()` functions.

use bevy::prelude::*;

use crate::hazard::{
    hazards,
    messages::HazardSelected,
    resources::{ActiveHazards, HazardRegistry},
};

/// Dispatch selected hazards: increment per-kind stack counts in
/// [`ActiveHazards`], then call the hazard's `activate(tuning, commands)`
/// fan-out so per-kind config resources are inserted.
pub(crate) fn dispatch_hazard_selection(
    mut reader: MessageReader<HazardSelected>,
    mut active: ResMut<ActiveHazards>,
    registry: Res<HazardRegistry>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        active.add_stack(msg.kind);
        if let Some(def) = registry.get(msg.kind) {
            hazards::activate(msg.kind, &def.tuning, &mut commands);
        } else {
            warn!(
                "dispatch_hazard_selection: no definition for {:?} — skipping activate()",
                msg.kind
            );
        }
    }
}

#[cfg(test)]
mod tests;
