//! `ArmedEffects` component for the unified effect system.

use bevy::prelude::*;

use crate::chips::definition::TriggerChain;

/// Partially-resolved trigger chains attached to a specific bolt entity.
///
/// When a trigger chain matches but the inner chain is not a leaf,
/// the inner chain is pushed onto this component's list. Subsequent
/// triggers evaluate against these armed chains.
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// archetype-originating chains and `Some(name)` for chip/evolution chains.
#[derive(Component, Debug, Default)]
pub(crate) struct ArmedEffects(pub Vec<(Option<String>, TriggerChain)>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armed_effects_default_is_empty() {
        let armed = ArmedEffects::default();
        assert!(
            armed.0.is_empty(),
            "ArmedEffects::default() should produce an empty vec"
        );
    }
}
