//! `EffectStack`<T> — generic stack component for passive effects.

use bevy::prelude::*;

use crate::effect_v3::traits::PassiveEffect;

/// Generic stack component for passive effects. Each entry is a
/// `(source, config)` pair. The source string identifies which chip
/// or definition added the entry.
///
/// Monomorphized per config type — `EffectStack<SpeedBoostConfig>` and
/// `EffectStack<DamageBoostConfig>` are independent Bevy components.
#[derive(Component)]
pub struct EffectStack<T: PassiveEffect> {
    entries: Vec<(String, T)>,
}

impl<T: PassiveEffect> Default for EffectStack<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T: PassiveEffect> EffectStack<T> {
    /// Append a `(source, config)` entry to the stack.
    ///
    /// Called by fire implementations.
    #[allow(
        clippy::needless_pass_by_ref_mut,
        reason = "stub — will mutate entries when implemented"
    )]
    pub fn push(&mut self, _source: String, _config: T) {
        todo!()
    }

    /// Find and remove the first entry matching `(source, config)` exactly.
    ///
    /// Called by reverse implementations. If no match is found, does nothing.
    #[allow(
        clippy::needless_pass_by_ref_mut,
        reason = "stub — will mutate entries when implemented"
    )]
    pub fn remove(&mut self, _source: &str, _config: &T) {
        todo!()
    }

    /// Compute the aggregated value from all stacked entries.
    ///
    /// Delegates to `T::aggregate(&self.entries)`. Returns the identity
    /// value (1.0 for multiplicative, 0 for additive) when the stack is empty.
    pub fn aggregate(&self) -> f32 {
        todo!()
    }
}
