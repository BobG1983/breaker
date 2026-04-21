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
///
/// Fails closed on missing `HazardRegistry` definitions: a
/// `HazardSelected` for a kind without a registered definition logs a
/// `warn!` and does NOT increment `ActiveHazards` — previously the stack
/// was incremented even when activation was skipped, leaving a silent
/// half-applied hazard (stack count > 0 with no runtime effect).
pub(crate) fn dispatch_hazard_selection(
    mut reader: MessageReader<HazardSelected>,
    mut active: ResMut<ActiveHazards>,
    registry: Res<HazardRegistry>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Some(def) = registry.get(msg.kind) else {
            warn!(
                "dispatch_hazard_selection: no definition for {:?} — skipping (stack NOT incremented)",
                msg.kind
            );
            continue;
        };
        active.add_stack(msg.kind);
        hazards::activate(msg.kind, &def.tuning, &mut commands);
    }
}

#[cfg(test)]
mod tests;
