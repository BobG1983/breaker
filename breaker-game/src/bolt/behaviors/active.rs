//! `ActiveOverclocks` resource — runtime list of overclock trigger chains.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// All overclock trigger chains currently active for the run.
///
/// Populated by `handle_overclock` when a `ChipEffectApplied` with an
/// `Overclock` effect is observed.
#[derive(Resource, Debug, Default)]
pub struct ActiveOverclocks(pub Vec<TriggerChain>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_overclocks_default_is_empty() {
        let active = ActiveOverclocks::default();
        assert!(
            active.0.is_empty(),
            "ActiveOverclocks::default() should produce an empty vec"
        );
    }
}
