//! Protocol definition types — `ProtocolKind`, `ProtocolTuning`, `ProtocolDefinition`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::types::RootNode;

/// Identifies a protocol by its variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolKind {
    /// Deadline — node-timer protocol.
    Deadline,
    /// Ricochet — bolt-reflection protocol.
    Ricochet,
    /// Anchor — breaker-positioning protocol.
    Anchor,
    /// Kickstart — run-launch protocol.
    Kickstart,
    /// `DebtCollector` — bump-stack protocol.
    DebtCollector,
    /// `IronCurtain` — damage-fraction protocol.
    IronCurtain,
    /// `EchoStrike` — echo-damage protocol.
    EchoStrike,
    /// Siphon — kill-streak-time protocol.
    Siphon,
    /// Greed — rarity-weighted chip-offer protocol.
    Greed,
    /// `RecklessDash` — risky-zone damage protocol.
    RecklessDash,
    /// Burnout — heat accumulation protocol.
    Burnout,
    /// Conductor — primary-swap protocol.
    Conductor,
    /// Afterimage — phantom-bolt protocol.
    Afterimage,
    /// Fission — kill-split protocol.
    Fission,
    /// `TierRegression` — tier-back protocol.
    TierRegression,
}

impl ProtocolKind {
    /// Canonical slice of every variant.
    pub const ALL: &[Self] = &[
        Self::Deadline,
        Self::Ricochet,
        Self::Anchor,
        Self::Kickstart,
        Self::DebtCollector,
        Self::IronCurtain,
        Self::EchoStrike,
        Self::Siphon,
        Self::Greed,
        Self::RecklessDash,
        Self::Burnout,
        Self::Conductor,
        Self::Afterimage,
        Self::Fission,
        Self::TierRegression,
    ];
}

/// Per-protocol tuning data carried by [`ProtocolDefinition`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum ProtocolTuning {
    // ── Effect-tree protocols (effects() returns Some) ──
    Deadline {
        effects: Vec<RootNode>,
    },
    Ricochet {
        effects: Vec<RootNode>,
    },
    Anchor {
        effects: Vec<RootNode>,
    },
    Kickstart {
        effects: Vec<RootNode>,
    },

    // ── Custom-system protocols (effects() returns None) ──
    DebtCollector {
        stack_per_bump: f32,
    },
    IronCurtain {
        damage_fraction: f32,
        falloff_start:   f32,
    },
    EchoStrike {
        max_echoes:      u32,
        newest_fraction: f32,
        middle_fraction: f32,
        oldest_fraction: f32,
    },
    Siphon {
        streak_window: f32,
        time_per_kill: f32,
    },
    Greed {
        rarity_boost_per_skip: f32,
    },
    RecklessDash {
        risky_zone_start:  f32,
        damage_multiplier: f32,
        double_penalty:    bool,
    },
    Burnout {
        fill_duration:               f32,
        drain_duration:              f32,
        still_threshold:             f32,
        full_heat_damage_multiplier: f32,
        speed_boost_duration:        f32,
    },
    Conductor {
        primary_swap_window: f32,
    },
    Afterimage {
        phantom_duration:      f32,
        phantom_bolt_duration: f32,
    },
    Fission {
        kills_per_split: u32,
    },
    TierRegression {
        tiers_back: u32,
    },
}

impl ProtocolTuning {
    /// Returns the [`ProtocolKind`] associated with this tuning.
    #[must_use]
    pub(crate) const fn kind(&self) -> ProtocolKind {
        match self {
            Self::Deadline { .. } => ProtocolKind::Deadline,
            Self::Ricochet { .. } => ProtocolKind::Ricochet,
            Self::Anchor { .. } => ProtocolKind::Anchor,
            Self::Kickstart { .. } => ProtocolKind::Kickstart,
            Self::DebtCollector { .. } => ProtocolKind::DebtCollector,
            Self::IronCurtain { .. } => ProtocolKind::IronCurtain,
            Self::EchoStrike { .. } => ProtocolKind::EchoStrike,
            Self::Siphon { .. } => ProtocolKind::Siphon,
            Self::Greed { .. } => ProtocolKind::Greed,
            Self::RecklessDash { .. } => ProtocolKind::RecklessDash,
            Self::Burnout { .. } => ProtocolKind::Burnout,
            Self::Conductor { .. } => ProtocolKind::Conductor,
            Self::Afterimage { .. } => ProtocolKind::Afterimage,
            Self::Fission { .. } => ProtocolKind::Fission,
            Self::TierRegression { .. } => ProtocolKind::TierRegression,
        }
    }

    /// Returns the effect tree for effect-tree protocols; `None` for
    /// custom-system protocols.
    #[must_use]
    pub(crate) const fn effects(&self) -> Option<&[RootNode]> {
        match self {
            Self::Deadline { effects }
            | Self::Ricochet { effects }
            | Self::Anchor { effects }
            | Self::Kickstart { effects } => Some(effects.as_slice()),
            _ => None,
        }
    }
}

/// A complete protocol definition — the payload loaded from `.protocol.ron`
/// files and stored in [`crate::protocol::resources::ProtocolRegistry`].
#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ProtocolDefinition {
    /// Display name shown on the protocol card.
    pub(crate) name:        String,
    /// Flavor text shown below the name.
    pub(crate) description: String,
    /// Meta-progression unlock tier. Defaults to 0 (always unlocked).
    #[serde(default)]
    pub(crate) unlock_tier: u32,
    /// Per-protocol tuning data.
    pub(crate) tuning:      ProtocolTuning,
}

impl ProtocolDefinition {
    /// Returns the [`ProtocolKind`] of the inner tuning.
    #[must_use]
    pub(crate) const fn kind(&self) -> ProtocolKind {
        self.tuning.kind()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // ── Behavior 1: ProtocolKind::ALL has 15 entries, all unique ──────────

    #[test]
    fn protocol_kind_all_has_15_entries() {
        assert_eq!(
            ProtocolKind::ALL.len(),
            15,
            "ProtocolKind::ALL must contain all 15 variants"
        );
    }

    #[test]
    fn protocol_kind_all_entries_are_unique() {
        let set: HashSet<_> = ProtocolKind::ALL.iter().copied().collect();
        assert_eq!(
            set.len(),
            15,
            "ProtocolKind::ALL must have 15 unique entries"
        );
    }

    #[test]
    fn protocol_kind_all_contains_every_variant() {
        let set: HashSet<_> = ProtocolKind::ALL.iter().copied().collect();
        let expected: HashSet<ProtocolKind> = [
            ProtocolKind::Deadline,
            ProtocolKind::Ricochet,
            ProtocolKind::Anchor,
            ProtocolKind::Kickstart,
            ProtocolKind::DebtCollector,
            ProtocolKind::IronCurtain,
            ProtocolKind::EchoStrike,
            ProtocolKind::Siphon,
            ProtocolKind::Greed,
            ProtocolKind::RecklessDash,
            ProtocolKind::Burnout,
            ProtocolKind::Conductor,
            ProtocolKind::Afterimage,
            ProtocolKind::Fission,
            ProtocolKind::TierRegression,
        ]
        .into_iter()
        .collect();
        assert_eq!(
            set, expected,
            "ProtocolKind::ALL must contain every variant exactly once"
        );
    }

    // ── Behavior 2: ProtocolTuning::kind() roundtrips for every variant ───

    /// Returns the full list of `(ProtocolTuning, expected ProtocolKind)` pairs
    /// from the spec. Using the exact field names/values from spec §2.
    fn all_tuning_variants() -> Vec<(ProtocolTuning, ProtocolKind)> {
        vec![
            (
                ProtocolTuning::Deadline { effects: vec![] },
                ProtocolKind::Deadline,
            ),
            (
                ProtocolTuning::Ricochet { effects: vec![] },
                ProtocolKind::Ricochet,
            ),
            (
                ProtocolTuning::Anchor { effects: vec![] },
                ProtocolKind::Anchor,
            ),
            (
                ProtocolTuning::Kickstart { effects: vec![] },
                ProtocolKind::Kickstart,
            ),
            (
                ProtocolTuning::DebtCollector {
                    stack_per_bump: 0.1,
                },
                ProtocolKind::DebtCollector,
            ),
            (
                ProtocolTuning::IronCurtain {
                    damage_fraction: 0.25,
                    falloff_start:   0.5,
                },
                ProtocolKind::IronCurtain,
            ),
            (
                ProtocolTuning::EchoStrike {
                    max_echoes:      3,
                    newest_fraction: 0.5,
                    middle_fraction: 0.25,
                    oldest_fraction: 0.125,
                },
                ProtocolKind::EchoStrike,
            ),
            (
                ProtocolTuning::Siphon {
                    streak_window: 2.0,
                    time_per_kill: 0.25,
                },
                ProtocolKind::Siphon,
            ),
            (
                ProtocolTuning::Greed {
                    rarity_boost_per_skip: 0.05,
                },
                ProtocolKind::Greed,
            ),
            (
                ProtocolTuning::RecklessDash {
                    risky_zone_start:  0.3,
                    damage_multiplier: 4.0,
                    double_penalty:    true,
                },
                ProtocolKind::RecklessDash,
            ),
            (
                ProtocolTuning::Burnout {
                    fill_duration:               3.0,
                    drain_duration:              5.0,
                    still_threshold:             0.25,
                    full_heat_damage_multiplier: 2.0,
                    speed_boost_duration:        1.0,
                },
                ProtocolKind::Burnout,
            ),
            (
                ProtocolTuning::Conductor {
                    primary_swap_window: 0.2,
                },
                ProtocolKind::Conductor,
            ),
            (
                ProtocolTuning::Afterimage {
                    phantom_duration:      1.5,
                    phantom_bolt_duration: 0.75,
                },
                ProtocolKind::Afterimage,
            ),
            (
                ProtocolTuning::Fission {
                    kills_per_split: 10,
                },
                ProtocolKind::Fission,
            ),
            (
                ProtocolTuning::TierRegression { tiers_back: 1 },
                ProtocolKind::TierRegression,
            ),
        ]
    }

    #[test]
    fn protocol_tuning_kind_roundtrips_for_every_variant() {
        for (tuning, expected) in all_tuning_variants() {
            assert_eq!(
                tuning.kind(),
                expected,
                "ProtocolTuning::{tuning:?}.kind() should be {expected:?}"
            );
        }
    }

    // ── Behavior 3: effects() returns Some(&[]) for 4 effect-tree protocols,
    //                None for 11 custom-system protocols ────────────────────

    #[test]
    fn protocol_tuning_effects_returns_some_empty_for_effect_tree_protocols() {
        let effect_tree: [ProtocolTuning; 4] = [
            ProtocolTuning::Deadline { effects: vec![] },
            ProtocolTuning::Ricochet { effects: vec![] },
            ProtocolTuning::Anchor { effects: vec![] },
            ProtocolTuning::Kickstart { effects: vec![] },
        ];

        for tuning in &effect_tree {
            let slice = tuning.effects();
            assert!(
                slice.is_some(),
                "effect-tree tuning {tuning:?} should return Some from effects()"
            );
            assert_eq!(
                slice.map(<[RootNode]>::len),
                Some(0),
                "effect-tree tuning {tuning:?} constructed with empty vec should report len 0"
            );
        }
    }

    #[test]
    fn protocol_tuning_effects_returns_none_for_custom_system_protocols() {
        let custom: Vec<ProtocolTuning> = vec![
            ProtocolTuning::DebtCollector {
                stack_per_bump: 0.1,
            },
            ProtocolTuning::IronCurtain {
                damage_fraction: 0.25,
                falloff_start:   0.5,
            },
            ProtocolTuning::EchoStrike {
                max_echoes:      3,
                newest_fraction: 0.5,
                middle_fraction: 0.25,
                oldest_fraction: 0.125,
            },
            ProtocolTuning::Siphon {
                streak_window: 2.0,
                time_per_kill: 0.25,
            },
            ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
            ProtocolTuning::RecklessDash {
                risky_zone_start:  0.3,
                damage_multiplier: 4.0,
                double_penalty:    true,
            },
            ProtocolTuning::Burnout {
                fill_duration:               3.0,
                drain_duration:              5.0,
                still_threshold:             0.25,
                full_heat_damage_multiplier: 2.0,
                speed_boost_duration:        1.0,
            },
            ProtocolTuning::Conductor {
                primary_swap_window: 0.2,
            },
            ProtocolTuning::Afterimage {
                phantom_duration:      1.5,
                phantom_bolt_duration: 0.75,
            },
            ProtocolTuning::Fission {
                kills_per_split: 10,
            },
            ProtocolTuning::TierRegression { tiers_back: 1 },
        ];
        assert_eq!(custom.len(), 11, "should be 11 custom-system protocols");

        for tuning in &custom {
            assert!(
                tuning.effects().is_none(),
                "custom-system tuning {tuning:?} should return None from effects()"
            );
        }
    }

    #[test]
    fn protocol_tuning_effects_reflects_stored_vec_contents() {
        // Edge case: proves the slice returned is actually the stored vec,
        // not always-empty.
        use crate::effect_v3::types::{EntityKind, RootNode, Tree};

        // A minimal RootNode: Spawn on Bolts with an empty Sequence tree.
        // Since we only care about `.len() == 1`, the internals are irrelevant.
        let stub_root = RootNode::Spawn(EntityKind::Bolt, Tree::Sequence(vec![]));
        let tuning = ProtocolTuning::Deadline {
            effects: vec![stub_root],
        };

        assert_eq!(
            tuning.effects().map(<[RootNode]>::len),
            Some(1),
            "effects() must return the stored vec, not always-empty"
        );
    }

    // ── Behavior 4: ProtocolDefinition::kind() delegates to tuning.kind() ─

    #[test]
    fn protocol_definition_kind_delegates_to_tuning() {
        let def = ProtocolDefinition {
            name:        "X".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        };
        assert_eq!(
            def.kind(),
            ProtocolKind::Greed,
            "ProtocolDefinition::kind() must delegate to tuning.kind()"
        );
    }
}
