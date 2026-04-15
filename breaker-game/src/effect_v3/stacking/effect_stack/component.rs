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
    pub fn push(&mut self, source: String, config: T) {
        self.entries.push((source, config));
    }

    /// Find and remove the first entry matching `(source, config)` exactly.
    ///
    /// Called by reverse implementations. If no match is found, does nothing.
    pub fn remove(&mut self, source: &str, config: &T) {
        if let Some(index) = self
            .entries
            .iter()
            .position(|(s, c)| s == source && c == config)
        {
            self.entries.remove(index);
        }
    }

    /// Compute the aggregated value from all stacked entries.
    ///
    /// Delegates to `T::aggregate(&self.entries)`. Returns the identity
    /// value (1.0 for multiplicative, 0 for additive) when the stack is empty.
    #[must_use]
    pub fn aggregate(&self) -> f32 {
        T::aggregate(&self.entries)
    }

    /// Returns `true` if the stack contains no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of entries in the stack.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Remove all entries whose source matches the given `source` string.
    ///
    /// Retains only entries whose source does NOT match. If no entries match,
    /// this is a no-op.
    pub fn retain_by_source(&mut self, source: &str) {
        self.entries.retain(|(s, _)| s != source);
    }

    /// Iterates over all `(source, config)` entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &(String, T)> {
        self.entries.iter()
    }
}
