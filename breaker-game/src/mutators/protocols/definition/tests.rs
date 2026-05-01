use std::collections::HashSet;

use super::types::*;

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
                risky_zone_start:  0.7,
                damage_multiplier: 4.0,
                double_penalty:    true,
            },
            ProtocolKind::RecklessDash,
        ),
        (
            ProtocolTuning::Burnout {
                fill_duration:               4.0,
                drain_duration:              2.0,
                still_threshold:             1.5,
                full_heat_damage_multiplier: 4.0,
                speed_boost_duration:        2.0,
                shockwave_base_range:        0.0,
                shockwave_range_per_level:   0.0,
                shockwave_stacks:            0,
                shockwave_speed:             0.0,
            },
            ProtocolKind::Burnout,
        ),
        (ProtocolTuning::Conductor, ProtocolKind::Conductor),
        (
            ProtocolTuning::Afterimage {
                phantom_duration:      1.5,
                phantom_bolt_duration: 0.75,
            },
            ProtocolKind::Afterimage,
        ),
        (
            ProtocolTuning::Fission {
                kills_per_split:      10,
                divergence_angle_rad: 0.0,
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
            slice.map(<[crate::effect_v3::types::RootNode]>::len),
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
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    true,
        },
        ProtocolTuning::Burnout {
            fill_duration:               4.0,
            drain_duration:              2.0,
            still_threshold:             1.5,
            full_heat_damage_multiplier: 4.0,
            speed_boost_duration:        2.0,
            shockwave_base_range:        0.0,
            shockwave_range_per_level:   0.0,
            shockwave_stacks:            0,
            shockwave_speed:             0.0,
        },
        ProtocolTuning::Conductor,
        ProtocolTuning::Afterimage {
            phantom_duration:      1.5,
            phantom_bolt_duration: 0.75,
        },
        ProtocolTuning::Fission {
            kills_per_split:      10,
            divergence_angle_rad: 0.0,
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

// ── A2.1–A2.16: ProtocolKind::kind_slug() pins per variant ───────────

#[test]
fn kind_slug_deadline_is_deadline() {
    assert_eq!(ProtocolKind::Deadline.kind_slug(), "deadline");
}

#[test]
fn kind_slug_ricochet_is_ricochet() {
    assert_eq!(ProtocolKind::Ricochet.kind_slug(), "ricochet");
}

#[test]
fn kind_slug_anchor_is_anchor() {
    assert_eq!(ProtocolKind::Anchor.kind_slug(), "anchor");
}

#[test]
fn kind_slug_kickstart_is_kickstart() {
    assert_eq!(ProtocolKind::Kickstart.kind_slug(), "kickstart");
}

#[test]
fn kind_slug_debt_collector_is_debt_collector() {
    assert_eq!(ProtocolKind::DebtCollector.kind_slug(), "debt_collector");
}

#[test]
fn kind_slug_iron_curtain_is_iron_curtain() {
    assert_eq!(ProtocolKind::IronCurtain.kind_slug(), "iron_curtain");
}

#[test]
fn kind_slug_echo_strike_is_echo_strike() {
    assert_eq!(ProtocolKind::EchoStrike.kind_slug(), "echo_strike");
}

#[test]
fn kind_slug_siphon_is_siphon() {
    assert_eq!(ProtocolKind::Siphon.kind_slug(), "siphon");
}

#[test]
fn kind_slug_greed_is_greed() {
    assert_eq!(ProtocolKind::Greed.kind_slug(), "greed");
}

#[test]
fn kind_slug_reckless_dash_is_reckless_dash() {
    assert_eq!(ProtocolKind::RecklessDash.kind_slug(), "reckless_dash");
}

#[test]
fn kind_slug_burnout_is_burnout() {
    assert_eq!(ProtocolKind::Burnout.kind_slug(), "burnout");
}

#[test]
fn kind_slug_conductor_is_conductor() {
    assert_eq!(ProtocolKind::Conductor.kind_slug(), "conductor");
}

#[test]
fn kind_slug_afterimage_is_afterimage() {
    assert_eq!(ProtocolKind::Afterimage.kind_slug(), "afterimage");
}

#[test]
fn kind_slug_fission_is_fission() {
    assert_eq!(ProtocolKind::Fission.kind_slug(), "fission");
}

#[test]
fn kind_slug_tier_regression_is_tier_regression() {
    assert_eq!(ProtocolKind::TierRegression.kind_slug(), "tier_regression");
}

#[test]
fn kind_slug_exhaustiveness_is_unique_and_nonempty() {
    let slugs: HashSet<&'static str> = ProtocolKind::ALL
        .iter()
        .map(ProtocolKind::kind_slug)
        .collect();
    assert_eq!(
        slugs.len(),
        ProtocolKind::ALL.len(),
        "every ProtocolKind variant must have a unique slug"
    );
    for kind in ProtocolKind::ALL {
        assert!(
            !kind.kind_slug().is_empty(),
            "kind_slug for {kind:?} must be non-empty"
        );
    }
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
