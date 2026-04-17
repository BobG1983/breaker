//! Cross-domain ordering anchors for the hazard-select screen.
//!
//! Other domains (e.g. `hazard`) order their own systems against these sets
//! instead of referencing bare hazard-select function symbols, keeping the
//! coupling surface stable if the underlying systems are renamed or split.

use bevy::prelude::*;

/// Ordering anchors exposed by the hazard-select domain.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HazardSelectSystems {
    /// Tags `generate_hazard_offerings`.
    GenerateOfferings,
    /// Tags `spawn_hazard_select`.
    SpawnScreen,
    /// Tags `handle_hazard_input`.
    HandleInput,
    /// Tags `tick_hazard_timer` — emits `HazardSelected` on auto-pick. The
    /// hazard dispatcher must order `.after` this set to avoid dropping the
    /// expiry message across a state transition.
    TickTimer,
}
