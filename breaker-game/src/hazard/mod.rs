//! Hazard domain — negative stackable hazards chosen during infinite play
//! at tier 9 and above.

pub mod definition;
pub(crate) mod hazards;
pub(crate) mod messages;
pub(crate) mod plugin;
pub mod resources;
pub(crate) mod systems;

use bevy::prelude::*;

use self::{definition::HazardKind, resources::HazardRegistry};

/// Activate a hazard using the registry-held tuning. Inserts the per-kind
/// config resource so the hazard's runtime systems see fresh values.
///
/// Returns `false` when no definition exists for `kind` — caller decides
/// how to surface that (scenario runner logs a warning and skips).
///
/// Exposed for the scenario runner's `InjectHazardStack` mutation so
/// hazard-runtime scenarios can install a hazard without going through the
/// `HazardSelect` UI. Production code should route selection through
/// `dispatch_hazard_selection` instead.
pub fn activate_from_registry(
    registry: &HazardRegistry,
    kind: HazardKind,
    commands: &mut Commands,
) -> bool {
    let Some(def) = registry.get(kind) else {
        return false;
    };
    hazards::activate(kind, &def.tuning, commands);
    true
}
