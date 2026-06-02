pub(super) fn make_full_hazard_registry() -> crate::mutators::hazards::resources::HazardRegistry {
    use crate::mutators::hazards::{
        definition::{HazardDefinition, HazardKind, HazardTuning},
        resources::HazardRegistry,
    };
    let mut registry = HazardRegistry::default();
    for kind in HazardKind::ALL {
        let tuning = match kind {
            HazardKind::Decay => HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            HazardKind::Drift => HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
            HazardKind::Haste => HazardTuning::Haste {
                base_percent:      0.1,
                per_level_percent: 0.05,
            },
            HazardKind::EchoCells => HazardTuning::EchoCells {
                delay_secs:           1.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
            HazardKind::Erosion => HazardTuning::Erosion {
                shrink_rate:      0.05,
                min_width_frac:   0.35,
                restore_nonwhiff: 0.25,
                restore_perfect:  0.5,
            },
            HazardKind::Cascade => HazardTuning::Cascade {
                base_heal:      1.0,
                per_level_heal: 0.5,
            },
            HazardKind::Fracture => HazardTuning::Fracture {
                base_splits:      1,
                per_level_splits: 1,
            },
            HazardKind::Renewal => HazardTuning::Renewal {
                base_period_secs:         10.0,
                per_level_reduction_frac: 0.2,
            },
            HazardKind::Volatility => HazardTuning::Volatility {
                hp_per_interval: 1.0,
                interval_secs:   5.0,
                max_multiplier:  2.0,
            },
            HazardKind::GravitySurge => HazardTuning::GravitySurge {
                base_duration_secs:      2.0,
                per_level_duration_secs: 1.0,
                base_strength:           100.0,
                per_level_strength_frac: 0.2,
            },
            HazardKind::Overcharge => HazardTuning::Overcharge {
                base_frac:      0.05,
                per_level_frac: 0.03,
            },
            HazardKind::Resonance => HazardTuning::Resonance {
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
            HazardKind::Diffusion => HazardTuning::Diffusion {
                base_share_frac:      0.2,
                per_level_share_frac: 0.1,
                depth_every_levels:   5,
            },
            HazardKind::Tether => HazardTuning::Tether {
                base_share_frac:         0.25,
                per_level_share_frac:    0.1,
                base_coverage_frac:      0.4,
                per_level_coverage_frac: 0.1,
            },
            HazardKind::Momentum => HazardTuning::Momentum {
                base_hp_per_hit:      10.0,
                per_level_hp_per_hit: 10.0,
            },
            HazardKind::Sympathy => HazardTuning::Sympathy {
                base_heal_frac:      0.25,
                per_level_heal_frac: 0.05,
                depth_every_levels:  5,
            },
        };
        registry.insert(HazardDefinition {
            name: format!("{kind:?}"),
            description: String::new(),
            unlock_tier: 0,
            tuning,
        });
    }
    registry
}
