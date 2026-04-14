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

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{
            BumpForceConfig, DamageBoostConfig, PiercingConfig, QuickStopConfig,
            RampingDamageConfig, SizeBoostConfig, SpeedBoostConfig, VulnerableConfig,
        },
        traits::PassiveEffect,
    };

    fn assert_f32_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "expected {expected}, got {actual}"
        );
    }

    // ---------------------------------------------------------------
    // Push tests (behaviors 1-3)
    // ---------------------------------------------------------------

    #[test]
    fn push_adds_single_entry_to_empty_stack() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".to_string(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());
    }

    #[test]
    fn push_succeeds_with_empty_source_name() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            String::new(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn push_appends_multiple_entries_preserving_insertion_order() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "feedback_loop".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        assert_eq!(stack.len(), 3);

        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "amp");
        assert_eq!(entries[0].1.multiplier, OrderedFloat(2.0));
        assert_eq!(entries[1].0, "feedback_loop");
        assert_eq!(entries[1].1.multiplier, OrderedFloat(1.5));
        assert_eq!(entries[2].0, "amp");
        assert_eq!(entries[2].1.multiplier, OrderedFloat(2.0));
    }

    #[test]
    fn push_allows_duplicate_source_config_pairs() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        stack.push("amp".into(), config.clone());
        stack.push("amp".into(), config);
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn push_allows_same_source_with_different_configs() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.2),
            },
        );
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        assert_eq!(stack.len(), 2);

        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].1.multiplier, OrderedFloat(1.2));
        assert_eq!(entries[1].1.multiplier, OrderedFloat(1.5));
    }

    // ---------------------------------------------------------------
    // Remove tests (behaviors 4-8)
    // ---------------------------------------------------------------

    #[test]
    fn remove_finds_and_removes_first_exact_match() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "feedback_loop".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        stack.remove(
            "amp",
            &DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        assert_eq!(stack.len(), 2);

        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "feedback_loop");
        assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
        assert_eq!(entries[1].0, "amp");
        assert_eq!(entries[1].1.multiplier, OrderedFloat(2.0));
    }

    #[test]
    fn remove_second_call_removes_remaining_match() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "feedback_loop".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        let target = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        stack.remove("amp", &target);
        stack.remove("amp", &target);

        assert_eq!(stack.len(), 1);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "feedback_loop");
    }

    #[test]
    fn remove_does_nothing_when_source_matches_but_config_differs() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        stack.remove(
            "amp",
            &DamageBoostConfig {
                multiplier: OrderedFloat(3.0),
            },
        );

        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn remove_does_nothing_when_config_matches_but_source_differs() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        stack.remove(
            "overclock",
            &DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn remove_on_empty_stack_is_no_op() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.remove(
            "anything",
            &SpeedBoostConfig {
                multiplier: OrderedFloat(1.0),
            },
        );
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn remove_only_removes_first_match_when_multiple_identical_entries_exist() {
        let mut stack = EffectStack::<PiercingConfig>::default();
        let config = PiercingConfig { charges: 2 };
        stack.push("amp".into(), config.clone());
        stack.push("amp".into(), config.clone());
        stack.push("amp".into(), config);

        stack.remove("amp", &PiercingConfig { charges: 2 });

        assert_eq!(stack.len(), 2);
    }

    // ---------------------------------------------------------------
    // is_empty / len / Default tests (behaviors 9-10)
    // ---------------------------------------------------------------

    #[test]
    fn default_stack_is_empty() {
        let stack = EffectStack::<SpeedBoostConfig>::default();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn is_empty_returns_false_after_push_true_after_removing_all() {
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push("splinter".into(), PiercingConfig { charges: 1 });
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);

        stack.remove("splinter", &PiercingConfig { charges: 1 });
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    // ---------------------------------------------------------------
    // Iteration tests (behavior 11)
    // ---------------------------------------------------------------

    #[test]
    fn iter_yields_references_in_insertion_order() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "feedback_loop".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0],
            &(
                "overclock".to_string(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }
            )
        );
        assert_eq!(
            entries[1],
            &(
                "feedback_loop".to_string(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(2.0),
                }
            )
        );
    }

    #[test]
    fn iter_on_empty_stack_yields_zero_items() {
        let stack = EffectStack::<SpeedBoostConfig>::default();
        assert_eq!(stack.iter().count(), 0);
    }

    // ---------------------------------------------------------------
    // EffectStack aggregate delegation tests (behaviors 12-17)
    // ---------------------------------------------------------------

    #[test]
    fn aggregate_on_empty_multiplicative_stack_returns_identity_one() {
        let stack = EffectStack::<SpeedBoostConfig>::default();
        assert_f32_eq(stack.aggregate(), 1.0);
    }

    #[test]
    fn aggregate_on_empty_additive_stack_returns_identity_zero() {
        let stack = EffectStack::<PiercingConfig>::default();
        assert_f32_eq(stack.aggregate(), 0.0);
    }

    #[test]
    fn aggregate_delegates_to_passive_effect_and_returns_product() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "augment".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        assert_f32_eq(stack.aggregate(), 3.0);
    }

    #[test]
    fn aggregate_delegates_to_passive_effect_and_returns_sum() {
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push("splinter".into(), PiercingConfig { charges: 3 });
        stack.push("piercing_bolt".into(), PiercingConfig { charges: 2 });
        assert_f32_eq(stack.aggregate(), 5.0);
    }

    #[test]
    fn removing_entry_from_multiplicative_stack_updates_aggregate() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "feedback_loop".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        assert_f32_eq(stack.aggregate(), 6.0);

        stack.remove(
            "amp",
            &DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );

        assert_f32_eq(stack.aggregate(), 3.0);
    }

    #[test]
    fn removing_all_multiplicative_entries_returns_aggregate_to_identity() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        let config = DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        };
        stack.push("amp".into(), config.clone());
        stack.remove("amp", &config);
        assert_f32_eq(stack.aggregate(), 1.0);
    }

    #[test]
    fn removing_entry_from_additive_stack_updates_aggregate() {
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push("splinter".into(), PiercingConfig { charges: 3 });
        stack.push("piercing_bolt".into(), PiercingConfig { charges: 2 });

        assert_f32_eq(stack.aggregate(), 5.0);

        stack.remove("splinter", &PiercingConfig { charges: 3 });

        assert_f32_eq(stack.aggregate(), 2.0);
    }

    // ---------------------------------------------------------------
    // Multiplicative PassiveEffect::aggregate tests (behaviors 18-25)
    // ---------------------------------------------------------------

    #[test]
    fn speed_boost_aggregate_empty_returns_one() {
        let entries: &[(String, SpeedBoostConfig)] = &[];
        assert_f32_eq(SpeedBoostConfig::aggregate(entries), 1.0);
    }

    #[test]
    fn speed_boost_aggregate_single_entry_returns_multiplier() {
        let entries = [(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        )];
        assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 1.5);
    }

    #[test]
    fn speed_boost_aggregate_two_entries_returns_product() {
        let entries = [
            (
                "overclock".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ),
            (
                "feedback_loop".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
        ];
        assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 3.0);
    }

    #[test]
    fn speed_boost_aggregate_three_entries_returns_product() {
        let entries = [
            (
                "a".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.25),
                },
            ),
            (
                "b".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ),
            (
                "c".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
        ];
        assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 3.75);
    }

    #[test]
    fn speed_boost_aggregate_identity_multiplier_does_not_change_product() {
        let entries = [
            (
                "a".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(1.0),
                },
            ),
            (
                "b".into(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
        ];
        assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 2.0);
    }

    #[test]
    fn size_boost_aggregate_empty_returns_one() {
        let entries: &[(String, SizeBoostConfig)] = &[];
        assert_f32_eq(SizeBoostConfig::aggregate(entries), 1.0);
    }

    #[test]
    fn size_boost_aggregate_two_entries_returns_product() {
        let entries = [
            (
                "augment".into(),
                SizeBoostConfig {
                    multiplier: OrderedFloat(1.2),
                },
            ),
            (
                "augment".into(),
                SizeBoostConfig {
                    multiplier: OrderedFloat(1.3),
                },
            ),
        ];
        assert_f32_eq(SizeBoostConfig::aggregate(&entries), 1.56);
    }

    #[test]
    fn size_boost_aggregate_identity_multiplier_unchanged() {
        let entries = [
            (
                "a".into(),
                SizeBoostConfig {
                    multiplier: OrderedFloat(1.0),
                },
            ),
            (
                "b".into(),
                SizeBoostConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ),
        ];
        assert_f32_eq(SizeBoostConfig::aggregate(&entries), 1.5);
    }

    #[test]
    fn damage_boost_aggregate_empty_returns_one() {
        let entries: &[(String, DamageBoostConfig)] = &[];
        assert_f32_eq(DamageBoostConfig::aggregate(entries), 1.0);
    }

    #[test]
    fn damage_boost_aggregate_two_entries_returns_product() {
        let entries = [
            (
                "amp".into(),
                DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
            (
                "amp".into(),
                DamageBoostConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
        ];
        assert_f32_eq(DamageBoostConfig::aggregate(&entries), 4.0);
    }

    #[test]
    fn damage_boost_aggregate_single_half_multiplier() {
        let entries = [(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(0.5),
            },
        )];
        assert_f32_eq(DamageBoostConfig::aggregate(&entries), 0.5);
    }

    #[test]
    fn bump_force_aggregate_empty_returns_one() {
        let entries: &[(String, BumpForceConfig)] = &[];
        assert_f32_eq(BumpForceConfig::aggregate(entries), 1.0);
    }

    #[test]
    fn bump_force_aggregate_two_entries_returns_product() {
        let entries = [
            (
                "augment".into(),
                BumpForceConfig {
                    multiplier: OrderedFloat(1.25),
                },
            ),
            (
                "augment".into(),
                BumpForceConfig {
                    multiplier: OrderedFloat(1.25),
                },
            ),
        ];
        assert_f32_eq(BumpForceConfig::aggregate(&entries), 1.5625);
    }

    #[test]
    fn quick_stop_aggregate_empty_returns_one() {
        let entries: &[(String, QuickStopConfig)] = &[];
        assert_f32_eq(QuickStopConfig::aggregate(entries), 1.0);
    }

    #[test]
    fn quick_stop_aggregate_two_entries_returns_product() {
        let entries = [
            (
                "chrono_passive".into(),
                QuickStopConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
            (
                "chrono_passive".into(),
                QuickStopConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ),
        ];
        assert_f32_eq(QuickStopConfig::aggregate(&entries), 3.0);
    }

    #[test]
    fn vulnerable_aggregate_empty_returns_one() {
        let entries: &[(String, VulnerableConfig)] = &[];
        assert_f32_eq(VulnerableConfig::aggregate(entries), 1.0);
    }

    #[test]
    fn vulnerable_aggregate_two_entries_returns_product() {
        let entries = [
            (
                "decay".into(),
                VulnerableConfig {
                    multiplier: OrderedFloat(1.5),
                },
            ),
            (
                "decay".into(),
                VulnerableConfig {
                    multiplier: OrderedFloat(2.0),
                },
            ),
        ];
        assert_f32_eq(VulnerableConfig::aggregate(&entries), 3.0);
    }

    #[test]
    fn vulnerable_aggregate_below_one_reduces() {
        let entries = [(
            "shield_effect".into(),
            VulnerableConfig {
                multiplier: OrderedFloat(0.5),
            },
        )];
        assert_f32_eq(VulnerableConfig::aggregate(&entries), 0.5);
    }

    // ---------------------------------------------------------------
    // Additive PassiveEffect::aggregate tests (behaviors 26-31)
    // ---------------------------------------------------------------

    #[test]
    fn piercing_aggregate_empty_returns_zero() {
        let entries: &[(String, PiercingConfig)] = &[];
        assert_f32_eq(PiercingConfig::aggregate(entries), 0.0);
    }

    #[test]
    fn piercing_aggregate_single_entry_returns_charges_as_f32() {
        let entries = [("splinter".into(), PiercingConfig { charges: 3 })];
        assert_f32_eq(PiercingConfig::aggregate(&entries), 3.0);
    }

    #[test]
    fn piercing_aggregate_two_entries_returns_sum() {
        let entries = [
            ("splinter".into(), PiercingConfig { charges: 3 }),
            ("piercing_bolt".into(), PiercingConfig { charges: 2 }),
        ];
        assert_f32_eq(PiercingConfig::aggregate(&entries), 5.0);
    }

    #[test]
    fn piercing_aggregate_zero_charges_does_not_change_sum() {
        let entries = [
            ("a".into(), PiercingConfig { charges: 0 }),
            ("b".into(), PiercingConfig { charges: 3 }),
        ];
        assert_f32_eq(PiercingConfig::aggregate(&entries), 3.0);
    }

    #[test]
    fn piercing_aggregate_all_zero_charges_returns_zero() {
        let entries = [
            ("a".into(), PiercingConfig { charges: 0 }),
            ("b".into(), PiercingConfig { charges: 0 }),
        ];
        assert_f32_eq(PiercingConfig::aggregate(&entries), 0.0);
    }

    #[test]
    fn ramping_damage_aggregate_empty_returns_zero() {
        let entries: &[(String, RampingDamageConfig)] = &[];
        assert_f32_eq(RampingDamageConfig::aggregate(entries), 0.0);
    }

    #[test]
    fn ramping_damage_aggregate_single_entry_returns_increment() {
        let entries = [(
            "amp".into(),
            RampingDamageConfig {
                increment: OrderedFloat(0.5),
            },
        )];
        assert_f32_eq(RampingDamageConfig::aggregate(&entries), 0.5);
    }

    #[test]
    fn ramping_damage_aggregate_two_entries_returns_sum() {
        let entries = [
            (
                "amp".into(),
                RampingDamageConfig {
                    increment: OrderedFloat(0.5),
                },
            ),
            (
                "amp".into(),
                RampingDamageConfig {
                    increment: OrderedFloat(0.25),
                },
            ),
        ];
        assert_f32_eq(RampingDamageConfig::aggregate(&entries), 0.75);
    }

    #[test]
    fn ramping_damage_aggregate_three_entries_returns_sum() {
        let entries = [
            (
                "amp".into(),
                RampingDamageConfig {
                    increment: OrderedFloat(0.5),
                },
            ),
            (
                "amp".into(),
                RampingDamageConfig {
                    increment: OrderedFloat(0.25),
                },
            ),
            (
                "amp".into(),
                RampingDamageConfig {
                    increment: OrderedFloat(1.0),
                },
            ),
        ];
        assert_f32_eq(RampingDamageConfig::aggregate(&entries), 1.75);
    }

    // ---------------------------------------------------------------
    // Mixed-source integration tests (behaviors 32-33)
    // ---------------------------------------------------------------

    #[test]
    fn multiplicative_aggregate_is_source_agnostic() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "chrono_passive".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.2),
            },
        );
        stack.push(
            "feedback_loop".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        assert_f32_eq(stack.aggregate(), 3.6);
    }

    // ---------------------------------------------------------------
    // retain_by_source tests (behaviors 1-6)
    // ---------------------------------------------------------------

    #[test]
    fn retain_by_source_removes_all_entries_matching_source() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "feedback_loop".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.3),
            },
        );

        stack.retain_by_source("overclock");

        assert_eq!(stack.len(), 1);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "feedback_loop");
        assert_f32_eq(stack.aggregate(), 2.0);
    }

    #[test]
    fn retain_by_source_with_no_matching_source_is_noop() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "feedback_loop".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );

        stack.retain_by_source("nonexistent");

        assert_eq!(stack.len(), 2);
        assert_f32_eq(stack.aggregate(), 3.0);
    }

    #[test]
    fn retain_by_source_on_empty_stack_is_noop() {
        let mut stack = EffectStack::<PiercingConfig>::default();

        stack.retain_by_source("anything");

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn retain_by_source_removes_all_entries_when_all_share_same_source() {
        let mut stack = EffectStack::<SpeedBoostConfig>::default();
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.3),
            },
        );

        stack.retain_by_source("overclock");

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_f32_eq(stack.aggregate(), 1.0);
    }

    #[test]
    fn retain_by_source_preserves_insertion_order_of_surviving_entries() {
        let mut stack = EffectStack::<DamageBoostConfig>::default();
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        );
        stack.push(
            "overclock".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        stack.push(
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(3.0),
            },
        );
        stack.push(
            "feedback_loop".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(1.2),
            },
        );

        stack.retain_by_source("amp");

        assert_eq!(stack.len(), 2);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "overclock");
        assert_eq!(entries[0].1.multiplier, OrderedFloat(1.5));
        assert_eq!(entries[1].0, "feedback_loop");
        assert_eq!(entries[1].1.multiplier, OrderedFloat(1.2));
    }

    #[test]
    fn retain_by_source_with_empty_string_matches_empty_string_entries() {
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push(String::new(), PiercingConfig { charges: 1 });
        stack.push("splinter".into(), PiercingConfig { charges: 3 });

        stack.retain_by_source("");

        assert_eq!(stack.len(), 1);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "splinter");
        assert_eq!(entries[0].1.charges, 3);
    }

    // ---------------------------------------------------------------
    // Mixed-source integration tests (behaviors 32-33)
    // ---------------------------------------------------------------

    #[test]
    fn additive_aggregate_is_source_agnostic() {
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push("splinter".into(), PiercingConfig { charges: 2 });
        stack.push("piercing_bolt".into(), PiercingConfig { charges: 5 });
        assert_f32_eq(stack.aggregate(), 7.0);
    }
}
