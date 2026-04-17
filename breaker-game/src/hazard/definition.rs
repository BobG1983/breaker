//! Hazard definition types — `HazardKind`, `HazardTuning`, `HazardDefinition`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Identifies a hazard by its variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HazardKind {
    /// Decay — cell-HP loss over time.
    Decay,
    /// Drift — periodic lateral force on bolts.
    Drift,
    /// Haste — cell-timer acceleration.
    Haste,
    /// `EchoCells` — delayed-respawn cells.
    EchoCells,
    /// Erosion — breaker shrinks on whiff.
    Erosion,
    /// Cascade — per-clear cell-heal.
    Cascade,
    /// Fracture — per-level cell splits.
    Fracture,
    /// Renewal — periodic node refresh.
    Renewal,
    /// Volatility — cell HP grows over time.
    Volatility,
    /// `GravitySurge` — periodic gravity on bolts.
    GravitySurge,
    /// Overcharge — bolt-damage boost and penalty.
    Overcharge,
    /// Resonance — bump-window narrowing.
    Resonance,
    /// Diffusion — shared cell damage.
    Diffusion,
    /// Tether — paired cell damage sharing.
    Tether,
    /// Momentum — required hits per cell.
    Momentum,
    /// Sympathy — paired cell heal sharing.
    Sympathy,
}

impl HazardKind {
    /// Canonical slice of every variant.
    pub const ALL: &[Self] = &[
        Self::Decay,
        Self::Drift,
        Self::Haste,
        Self::EchoCells,
        Self::Erosion,
        Self::Cascade,
        Self::Fracture,
        Self::Renewal,
        Self::Volatility,
        Self::GravitySurge,
        Self::Overcharge,
        Self::Resonance,
        Self::Diffusion,
        Self::Tether,
        Self::Momentum,
        Self::Sympathy,
    ];
}

/// Per-hazard tuning data carried by [`HazardDefinition`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum HazardTuning {
    Decay {
        base_percent:      f32,
        per_level_percent: f32,
    },
    Drift {
        force:           f32,
        period_secs:     f32,
        per_level_force: f32,
    },
    Haste {
        base_percent:      f32,
        per_level_percent: f32,
    },
    EchoCells {
        delay_secs:           f32,
        base_hp:              f32,
        per_level_multiplier: f32,
    },
    Erosion {
        shrink_rate:      f32,
        min_width_frac:   f32,
        restore_nonwhiff: f32,
        restore_perfect:  f32,
    },
    Cascade {
        base_heal:      f32,
        per_level_heal: f32,
    },
    Fracture {
        base_splits:      u32,
        per_level_splits: u32,
    },
    Renewal {
        base_period_secs:         f32,
        per_level_reduction_frac: f32,
    },
    Volatility {
        hp_per_interval: f32,
        interval_secs:   f32,
        max_multiplier:  f32,
    },
    GravitySurge {
        base_duration_secs:      f32,
        per_level_duration_secs: f32,
        base_strength:           f32,
        per_level_strength_frac: f32,
    },
    Overcharge {
        base_frac:      f32,
        per_level_frac: f32,
    },
    Resonance {
        base_window_secs:      f32,
        per_level_window_secs: f32,
    },
    Diffusion {
        base_share_frac:      f32,
        per_level_share_frac: f32,
        depth_every_levels:   u32,
    },
    Tether {
        base_share_frac:         f32,
        per_level_share_frac:    f32,
        base_coverage_frac:      f32,
        per_level_coverage_frac: f32,
    },
    Momentum {
        base_hp_per_hit:      f32,
        per_level_hp_per_hit: f32,
    },
    Sympathy {
        base_heal_frac:      f32,
        per_level_heal_frac: f32,
        depth_every_levels:  u32,
    },
}

impl HazardTuning {
    /// Returns the [`HazardKind`] associated with this tuning.
    #[must_use]
    pub(crate) const fn kind(&self) -> HazardKind {
        match self {
            Self::Decay { .. } => HazardKind::Decay,
            Self::Drift { .. } => HazardKind::Drift,
            Self::Haste { .. } => HazardKind::Haste,
            Self::EchoCells { .. } => HazardKind::EchoCells,
            Self::Erosion { .. } => HazardKind::Erosion,
            Self::Cascade { .. } => HazardKind::Cascade,
            Self::Fracture { .. } => HazardKind::Fracture,
            Self::Renewal { .. } => HazardKind::Renewal,
            Self::Volatility { .. } => HazardKind::Volatility,
            Self::GravitySurge { .. } => HazardKind::GravitySurge,
            Self::Overcharge { .. } => HazardKind::Overcharge,
            Self::Resonance { .. } => HazardKind::Resonance,
            Self::Diffusion { .. } => HazardKind::Diffusion,
            Self::Tether { .. } => HazardKind::Tether,
            Self::Momentum { .. } => HazardKind::Momentum,
            Self::Sympathy { .. } => HazardKind::Sympathy,
        }
    }
}

/// A complete hazard definition — the payload loaded from `.hazard.ron`
/// (future commit) and stored in [`crate::hazard::resources::HazardRegistry`].
#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct HazardDefinition {
    /// Display name shown on the hazard card.
    pub(crate) name:        String,
    /// Flavor text shown below the name.
    pub(crate) description: String,
    /// Meta-progression unlock tier. Defaults to 0 (always unlocked).
    #[serde(default)]
    pub(crate) unlock_tier: u32,
    /// Per-hazard tuning data.
    pub(crate) tuning:      HazardTuning,
}

impl HazardDefinition {
    /// Returns the [`HazardKind`] of the inner tuning.
    #[must_use]
    pub(crate) const fn kind(&self) -> HazardKind {
        self.tuning.kind()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // ── Behavior 15: HazardKind::ALL has 16 entries, all unique ───────────

    #[test]
    fn hazard_kind_all_has_16_entries() {
        assert_eq!(
            HazardKind::ALL.len(),
            16,
            "HazardKind::ALL must contain all 16 variants"
        );
    }

    #[test]
    fn hazard_kind_all_entries_are_unique() {
        let set: HashSet<_> = HazardKind::ALL.iter().copied().collect();
        assert_eq!(set.len(), 16, "HazardKind::ALL must have 16 unique entries");
    }

    #[test]
    fn hazard_kind_all_contains_every_variant() {
        let set: HashSet<_> = HazardKind::ALL.iter().copied().collect();
        let expected: HashSet<HazardKind> = [
            HazardKind::Decay,
            HazardKind::Drift,
            HazardKind::Haste,
            HazardKind::EchoCells,
            HazardKind::Erosion,
            HazardKind::Cascade,
            HazardKind::Fracture,
            HazardKind::Renewal,
            HazardKind::Volatility,
            HazardKind::GravitySurge,
            HazardKind::Overcharge,
            HazardKind::Resonance,
            HazardKind::Diffusion,
            HazardKind::Tether,
            HazardKind::Momentum,
            HazardKind::Sympathy,
        ]
        .into_iter()
        .collect();
        assert_eq!(
            set, expected,
            "HazardKind::ALL must contain every variant exactly once"
        );
    }

    // ── Behavior 16: HazardTuning::kind() roundtrips for every variant ────

    fn tuning_variants_first_half() -> Vec<(HazardTuning, HazardKind)> {
        vec![
            (
                HazardTuning::Decay {
                    base_percent:      0.05,
                    per_level_percent: 0.03,
                },
                HazardKind::Decay,
            ),
            (
                HazardTuning::Drift {
                    force:           100.0,
                    period_secs:     8.0,
                    per_level_force: 33.3,
                },
                HazardKind::Drift,
            ),
            (
                HazardTuning::Haste {
                    base_percent:      0.1,
                    per_level_percent: 0.05,
                },
                HazardKind::Haste,
            ),
            (
                HazardTuning::EchoCells {
                    delay_secs:           1.5,
                    base_hp:              1.0,
                    per_level_multiplier: 2.0,
                },
                HazardKind::EchoCells,
            ),
            (
                HazardTuning::Erosion {
                    shrink_rate:      0.05,
                    min_width_frac:   0.35,
                    restore_nonwhiff: 0.25,
                    restore_perfect:  0.5,
                },
                HazardKind::Erosion,
            ),
            (
                HazardTuning::Cascade {
                    base_heal:      1.0,
                    per_level_heal: 0.5,
                },
                HazardKind::Cascade,
            ),
            (
                HazardTuning::Fracture {
                    base_splits:      1,
                    per_level_splits: 1,
                },
                HazardKind::Fracture,
            ),
            (
                HazardTuning::Renewal {
                    base_period_secs:         10.0,
                    per_level_reduction_frac: 0.2,
                },
                HazardKind::Renewal,
            ),
        ]
    }

    fn tuning_variants_second_half() -> Vec<(HazardTuning, HazardKind)> {
        vec![
            (
                HazardTuning::Volatility {
                    hp_per_interval: 1.0,
                    interval_secs:   5.0,
                    max_multiplier:  2.0,
                },
                HazardKind::Volatility,
            ),
            (
                HazardTuning::GravitySurge {
                    base_duration_secs:      2.0,
                    per_level_duration_secs: 1.0,
                    base_strength:           100.0,
                    per_level_strength_frac: 0.2,
                },
                HazardKind::GravitySurge,
            ),
            (
                HazardTuning::Overcharge {
                    base_frac:      0.05,
                    per_level_frac: 0.03,
                },
                HazardKind::Overcharge,
            ),
            (
                HazardTuning::Resonance {
                    base_window_secs:      0.5,
                    per_level_window_secs: 0.3,
                },
                HazardKind::Resonance,
            ),
            (
                HazardTuning::Diffusion {
                    base_share_frac:      0.2,
                    per_level_share_frac: 0.1,
                    depth_every_levels:   5,
                },
                HazardKind::Diffusion,
            ),
            (
                HazardTuning::Tether {
                    base_share_frac:         0.25,
                    per_level_share_frac:    0.1,
                    base_coverage_frac:      0.4,
                    per_level_coverage_frac: 0.1,
                },
                HazardKind::Tether,
            ),
            (
                HazardTuning::Momentum {
                    base_hp_per_hit:      10.0,
                    per_level_hp_per_hit: 10.0,
                },
                HazardKind::Momentum,
            ),
            (
                HazardTuning::Sympathy {
                    base_heal_frac:      0.25,
                    per_level_heal_frac: 0.05,
                    depth_every_levels:  5,
                },
                HazardKind::Sympathy,
            ),
        ]
    }

    fn all_tuning_variants() -> Vec<(HazardTuning, HazardKind)> {
        let mut variants = tuning_variants_first_half();
        variants.extend(tuning_variants_second_half());
        variants
    }

    #[test]
    fn hazard_tuning_kind_roundtrips_for_every_variant() {
        for (tuning, expected) in all_tuning_variants() {
            assert_eq!(
                tuning.kind(),
                expected,
                "HazardTuning::{tuning:?}.kind() should be {expected:?}"
            );
        }
    }

    // ── Behavior 17: HazardDefinition::kind() delegates to tuning.kind() ──

    #[test]
    fn hazard_definition_kind_delegates_to_tuning() {
        let def = HazardDefinition {
            name:        "Decay".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
        };
        assert_eq!(
            def.kind(),
            HazardKind::Decay,
            "HazardDefinition::kind() must delegate to tuning.kind()"
        );
    }
}
