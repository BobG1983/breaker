//! `ArmedTriggers` component — per-bolt list of partially-resolved trigger chains.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// Partially-resolved trigger chains attached to a specific bolt entity.
///
/// When a trigger chain matches but the inner chain is not a leaf,
/// the inner chain is pushed onto this component's list. Subsequent
/// triggers evaluate against these armed chains.
#[derive(Component, Debug, Default)]
pub(crate) struct ArmedTriggers(pub Vec<TriggerChain>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armed_triggers_default_is_empty() {
        let armed = ArmedTriggers::default();
        assert!(
            armed.0.is_empty(),
            "ArmedTriggers::default() should produce an empty vec"
        );
    }
}
