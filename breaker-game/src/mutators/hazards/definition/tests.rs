use std::collections::HashSet;

use super::types::*;

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
                kills_to_trigger:      2,
                base_window:           0.5,
                window_per_level:      0.3,
                wave_speed:            200.0,
                base_slow_duration:    1.5,
                base_slow_strength:    0.5,
                slow_duration_scaling: 0.2,
                slow_strength_scaling: 0.15,
                contact_threshold:     16.0,
                wave_max_lifetime:     10.0,
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

// ── A3.1–A3.17: HazardKind::kind_slug() pins per variant ─────────────

#[test]
fn kind_slug_decay_is_decay() {
    assert_eq!(HazardKind::Decay.kind_slug(), "decay");
}

#[test]
fn kind_slug_drift_is_drift() {
    assert_eq!(HazardKind::Drift.kind_slug(), "drift");
}

#[test]
fn kind_slug_haste_is_haste() {
    assert_eq!(HazardKind::Haste.kind_slug(), "haste");
}

#[test]
fn kind_slug_echo_cells_is_echo_cells() {
    assert_eq!(HazardKind::EchoCells.kind_slug(), "echo_cells");
}

#[test]
fn kind_slug_erosion_is_erosion() {
    assert_eq!(HazardKind::Erosion.kind_slug(), "erosion");
}

#[test]
fn kind_slug_cascade_is_cascade() {
    assert_eq!(HazardKind::Cascade.kind_slug(), "cascade");
}

#[test]
fn kind_slug_fracture_is_fracture() {
    assert_eq!(HazardKind::Fracture.kind_slug(), "fracture");
}

#[test]
fn kind_slug_renewal_is_renewal() {
    assert_eq!(HazardKind::Renewal.kind_slug(), "renewal");
}

#[test]
fn kind_slug_volatility_is_volatility() {
    assert_eq!(HazardKind::Volatility.kind_slug(), "volatility");
}

#[test]
fn kind_slug_gravity_surge_is_gravity_surge() {
    assert_eq!(HazardKind::GravitySurge.kind_slug(), "gravity_surge");
}

#[test]
fn kind_slug_overcharge_is_overcharge() {
    assert_eq!(HazardKind::Overcharge.kind_slug(), "overcharge");
}

#[test]
fn kind_slug_resonance_is_resonance() {
    assert_eq!(HazardKind::Resonance.kind_slug(), "resonance");
}

#[test]
fn kind_slug_diffusion_is_diffusion() {
    assert_eq!(HazardKind::Diffusion.kind_slug(), "diffusion");
}

#[test]
fn kind_slug_tether_is_tether() {
    assert_eq!(HazardKind::Tether.kind_slug(), "tether");
}

#[test]
fn kind_slug_momentum_is_momentum() {
    assert_eq!(HazardKind::Momentum.kind_slug(), "momentum");
}

#[test]
fn kind_slug_sympathy_is_sympathy() {
    assert_eq!(HazardKind::Sympathy.kind_slug(), "sympathy");
}

#[test]
fn kind_slug_exhaustiveness_is_unique_and_nonempty() {
    let slugs: HashSet<&'static str> = HazardKind::ALL.iter().map(HazardKind::kind_slug).collect();
    assert_eq!(
        slugs.len(),
        HazardKind::ALL.len(),
        "every HazardKind variant must have a unique slug"
    );
    for kind in HazardKind::ALL {
        assert!(
            !kind.kind_slug().is_empty(),
            "kind_slug for {kind:?} must be non-empty"
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
