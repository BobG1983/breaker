//! `ActiveChains` resource — runtime list of active trigger chains.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// All trigger chains currently active for the run.
///
/// Populated by `init_archetype` from the archetype definition and by
/// `handle_overclock` when a `ChipEffectApplied` with an `Overclock`
/// effect is observed.
#[derive(Resource, Debug, Default)]
pub struct ActiveChains(pub Vec<TriggerChain>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_chains_default_is_empty() {
        let active = ActiveChains::default();
        assert!(
            active.0.is_empty(),
            "ActiveChains::default() should produce an empty vec"
        );
    }
}
