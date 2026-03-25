//! `ActiveEffects` resource — runtime list of active trigger chains.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// All trigger chains currently active for the run.
///
/// Populated by `init_archetype` from the archetype definition and by
/// `apply_chip_effect` when a chip with a triggered chain is selected.
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// archetype-originating chains and `Some(name)` for chip/evolution chains.
#[derive(Resource, Debug, Default)]
pub struct ActiveEffects(pub Vec<(Option<String>, TriggerChain)>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_effects_default_is_empty() {
        let active = ActiveEffects::default();
        assert!(
            active.0.is_empty(),
            "ActiveEffects::default() should produce an empty vec"
        );
    }
}
