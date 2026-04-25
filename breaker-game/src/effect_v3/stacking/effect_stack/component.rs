//! `EffectStack`<T> — generic stack component for passive effects.

use bevy::prelude::*;

use crate::{effect_v3::traits::PassiveEffect, prelude::SourceId};

/// Generic stack component for passive effects. Each entry is a
/// `(source, config)` pair. The `SourceId` identifies which chip
/// or definition added the entry.
///
/// Monomorphized per config type — `EffectStack<SpeedBoostConfig>` and
/// `EffectStack<DamageBoostConfig>` are independent Bevy components.
#[derive(Component)]
pub struct EffectStack<T: PassiveEffect> {
    entries: Vec<(SourceId, T)>,
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
    pub fn push(&mut self, source: SourceId, config: T) {
        self.entries.push((source, config));
    }

    /// Find and remove the first entry matching `(source, config)` exactly.
    ///
    /// Called by reverse implementations. If no match is found, does nothing.
    pub fn remove(&mut self, source: &SourceId, config: &T) {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|(s, c)| s == source && c == config)
        {
            self.entries.remove(pos);
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

    /// Remove all entries whose source matches the given `source`.
    ///
    /// Retains only entries whose source does NOT match. If no entries match,
    /// this is a no-op.
    pub fn retain_by_source(&mut self, source: &SourceId) {
        self.entries.retain(|(s, _)| s != source);
    }

    /// Iterates over all `(source, config)` entries in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &(SourceId, T)> {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        chips::definition::Rarity, effect_v3::effects::SpeedBoostConfig, prelude::SourceIdExt,
    };

    fn cfg(mult: f32) -> SpeedBoostConfig {
        SpeedBoostConfig {
            multiplier: OrderedFloat(mult),
        }
    }

    /// Builder-format `SourceId` for the canonical "Pulse" chip used by
    /// these mechanics tests in place of arbitrary string fixtures.
    fn pulse_source() -> SourceId {
        SourceId::chip("Pulse").rarity(Rarity::Common).build()
    }

    /// Distinct builder-format chip source — used as the "other" entry in
    /// retain/remove tests.
    fn other_source() -> SourceId {
        SourceId::chip("Other").rarity(Rarity::Common).build()
    }

    /// Three distinct builder-format chip sources — used by ordering tests.
    fn alpha_source() -> SourceId {
        SourceId::chip("Alpha").rarity(Rarity::Common).build()
    }

    fn beta_source() -> SourceId {
        SourceId::chip("Beta").rarity(Rarity::Common).build()
    }

    fn gamma_source() -> SourceId {
        SourceId::chip("Gamma").rarity(Rarity::Common).build()
    }

    // ── B59a — push stores the (SourceId, T) entry ──

    #[test]
    fn push_stores_entry() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let source = pulse_source();
        let config = cfg(1.5);

        stack.push(source.clone(), config.clone());

        assert_eq!(stack.len(), 1, "push must store exactly one entry");
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(
            entries[0].0, source,
            "stored source must match the pushed SourceId"
        );
        assert_eq!(
            entries[0].1, config,
            "stored config must match the pushed config"
        );
    }

    #[test]
    fn push_multiple_appends_in_order() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let s1 = alpha_source();
        let s2 = beta_source();

        stack.push(s1.clone(), cfg(1.5));
        stack.push(s2.clone(), cfg(2.0));

        assert_eq!(stack.len(), 2);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, s1, "first entry must be the first push");
        assert_eq!(entries[1].0, s2, "second entry must be the second push");
    }

    // ── B59b — remove(&SourceId, &T) removes a matching entry ──

    #[test]
    fn remove_removes_first_matching_entry() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let source = pulse_source();
        let config = cfg(1.5);
        stack.push(source.clone(), config.clone());

        stack.remove(&source, &config);

        assert_eq!(stack.len(), 0, "remove must drop the matching entry");
    }

    #[test]
    fn remove_with_non_matching_source_is_noop() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let source = alpha_source();
        let other = beta_source();
        let config = cfg(1.5);
        stack.push(source, config.clone());

        stack.remove(&other, &config);

        assert_eq!(
            stack.len(),
            1,
            "non-matching source must not remove anything"
        );
    }

    #[test]
    fn remove_with_non_matching_config_is_noop() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let source = alpha_source();
        stack.push(source.clone(), cfg(1.5));

        stack.remove(&source, &cfg(2.0));

        assert_eq!(
            stack.len(),
            1,
            "non-matching config must not remove anything"
        );
    }

    // ── B59c — retain_by_source removes all entries with matching source ──

    #[test]
    fn retain_by_source_removes_all_entries_with_matching_source() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let target = pulse_source();
        let other = other_source();
        stack.push(target.clone(), cfg(1.5));
        stack.push(other.clone(), cfg(2.0));
        stack.push(target.clone(), cfg(2.5));

        stack.retain_by_source(&target);

        assert_eq!(
            stack.len(),
            1,
            "retain_by_source must remove all matching-source entries"
        );
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(
            entries[0].0, other,
            "the only surviving entry must be the non-matching source"
        );
    }

    #[test]
    fn retain_by_source_with_no_matches_is_noop() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let kept = alpha_source();
        let target = beta_source();
        stack.push(kept, cfg(1.5));

        stack.retain_by_source(&target);

        assert_eq!(stack.len(), 1, "no-match retain must be a no-op");
    }

    // ── B59d — iter() yields &(SourceId, T) tuples in insertion order ──

    #[test]
    fn iter_yields_source_id_tuples_in_insertion_order() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        let s1 = alpha_source();
        let s2 = beta_source();
        let s3 = gamma_source();
        stack.push(s1.clone(), cfg(1.0));
        stack.push(s2.clone(), cfg(2.0));
        stack.push(s3.clone(), cfg(3.0));

        let collected: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0].0, s1);
        assert_eq!(collected[1].0, s2);
        assert_eq!(collected[2].0, s3);
    }

    // ── B59e — Cow equality across Borrowed/Owned variants ──
    //
    // SourceId wraps Cow<'static, str>. Equality on Cow is by string content,
    // so a `Borrowed("X")` and an `Owned(String::from("X"))` must compare
    // equal — which means a remove() call passing the Owned variant must
    // remove an entry pushed with the Borrowed variant (and vice versa).

    #[test]
    fn remove_matches_across_cow_borrowed_and_owned_variants() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();

        // Push using a `&'static str` (likely Cow::Borrowed).
        let pushed = SourceId::from("X");
        let config = cfg(1.5);
        stack.push(pushed, config.clone());
        assert_eq!(stack.len(), 1);

        // Remove using an Owned String (Cow::Owned). Equality must hold.
        let removed = SourceId::from("X".to_owned());
        stack.remove(&removed, &config);

        assert_eq!(
            stack.len(),
            0,
            "remove must succeed across Cow::Borrowed / Cow::Owned variants \
             when the underlying string content is equal"
        );
    }

    #[test]
    fn retain_by_source_matches_across_cow_variants() {
        let mut stack: EffectStack<SpeedBoostConfig> = EffectStack::default();
        stack.push(SourceId::from("X"), cfg(1.0));
        stack.push(SourceId::from("X".to_owned()), cfg(2.0));
        stack.push(SourceId::from("Y"), cfg(3.0));

        // Strip both X-sourced entries using an Owned variant.
        stack.retain_by_source(&SourceId::from("X".to_owned()));

        assert_eq!(stack.len(), 1, "both X variants must be removed");
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, SourceId::from("Y"));
    }
}
