//! UI domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the UI domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum UiSystems {
    /// The `spawn_timer_hud` system — spawns the timer display on node entry.
    SpawnTimerHud,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_timer_hud_variant_exists() {
        // Ensures UiSystems::SpawnTimerHud is a valid, distinct variant
        let set = UiSystems::SpawnTimerHud;
        assert_eq!(set, UiSystems::SpawnTimerHud);
    }
}
