use super::types::*;
use crate::state::run::resources::DifficultyCurveDefaults;

// -- NodeType deserialization --

#[test]
fn node_type_passive_deserializes_from_ron() {
    let result: NodeType = ron::de::from_str("Passive").expect("Passive should deserialize");
    assert_eq!(result, NodeType::Passive);
}

#[test]
fn node_type_active_deserializes_from_ron() {
    let result: NodeType = ron::de::from_str("Active").expect("Active should deserialize");
    assert_eq!(result, NodeType::Active);
}

#[test]
fn node_type_boss_deserializes_from_ron() {
    let result: NodeType = ron::de::from_str("Boss").expect("Boss should deserialize");
    assert_eq!(result, NodeType::Boss);
}

// -- TierNodeCount deserialization --

#[test]
fn tier_node_count_fixed_deserializes_from_ron() {
    let result: TierNodeCount = ron::de::from_str("Fixed(5)").expect("Fixed(5) should deserialize");
    assert_eq!(result, TierNodeCount::Fixed(5));
}

#[test]
fn tier_node_count_fixed_zero_deserializes_from_ron() {
    let result: TierNodeCount = ron::de::from_str("Fixed(0)").expect("Fixed(0) should deserialize");
    assert_eq!(result, TierNodeCount::Fixed(0));
}

#[test]
fn tier_node_count_range_deserializes_from_ron() {
    let result: TierNodeCount =
        ron::de::from_str("Range(4, 6)").expect("Range(4, 6) should deserialize");
    assert_eq!(result, TierNodeCount::Range(4, 6));
}

#[test]
fn tier_node_count_range_min_equals_max_deserializes_from_ron() {
    let result: TierNodeCount =
        ron::de::from_str("Range(5, 5)").expect("Range(5, 5) should deserialize");
    assert_eq!(result, TierNodeCount::Range(5, 5));
}

// -- TierNodeCount::validate --

#[test]
fn validate_accepts_fixed_count() {
    assert!(TierNodeCount::Fixed(5).validate().is_ok());
}

#[test]
fn validate_accepts_fixed_zero() {
    assert!(TierNodeCount::Fixed(0).validate().is_ok());
}

#[test]
fn validate_accepts_valid_range() {
    assert!(TierNodeCount::Range(4, 6).validate().is_ok());
}

#[test]
fn validate_accepts_range_min_equals_max() {
    assert!(TierNodeCount::Range(5, 5).validate().is_ok());
}

#[test]
fn validate_rejects_range_min_greater_than_max() {
    let result = TierNodeCount::Range(6, 4).validate();
    assert!(result.is_err(), "Range(6, 4) should be rejected");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("min") || msg.contains("max"),
        "error message should mention min/max, got: {msg}"
    );
}

// -- TierDefinition deserialization --

#[test]
fn tier_definition_deserializes_from_ron() {
    let ron_str =
        "(nodes: Range(4, 6), active_ratio: 0.2, timer_mult: 0.9, introduced_cells: ['T'])";
    let tier: TierDefinition =
        ron::de::from_str(ron_str).expect("TierDefinition should deserialize");
    assert_eq!(tier.nodes, TierNodeCount::Range(4, 6));
    assert!((tier.active_ratio - 0.2).abs() < f32::EPSILON);
    assert!((tier.timer_mult - 0.9).abs() < f32::EPSILON);
    assert_eq!(tier.introduced_cells, vec!['T']);
}

#[test]
fn tier_definition_empty_introduced_cells_deserializes() {
    let ron_str = "(nodes: Fixed(3), active_ratio: 0.0, timer_mult: 1.0, introduced_cells: [])";
    let tier: TierDefinition =
        ron::de::from_str(ron_str).expect("empty introduced_cells should deserialize");
    assert!(tier.introduced_cells.is_empty());
}

// -- DifficultyCurveDefaults deserialization --

#[test]
fn difficulty_curve_defaults_deserializes_from_ron() {
    let ron_str = "
(
    tiers: [
        (nodes: Fixed(3), active_ratio: 0.0, timer_mult: 1.0, introduced_cells: []),
        (nodes: Range(4, 6), active_ratio: 0.5, timer_mult: 0.8, introduced_cells: ['T']),
    ],
    timer_reduction_per_boss: 0.1,
)";
    let defaults: DifficultyCurveDefaults =
        ron::de::from_str(ron_str).expect("DifficultyCurveDefaults should deserialize");
    assert_eq!(defaults.tiers.len(), 2);
    assert!((defaults.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON);
    assert_eq!(defaults.tiers[0].nodes, TierNodeCount::Fixed(3));
    assert!((defaults.tiers[0].active_ratio - 0.0).abs() < f32::EPSILON);
}

#[test]
fn difficulty_curve_defaults_empty_tiers_deserializes() {
    let ron_str = "
(
    tiers: [],
    timer_reduction_per_boss: 0.05,
)";
    let defaults: DifficultyCurveDefaults =
        ron::de::from_str(ron_str).expect("empty tiers should deserialize");
    assert!(defaults.tiers.is_empty());
}

// -- difficulty.ron file parse --

#[test]
fn difficulty_ron_file_parses() {
    let ron_str = include_str!("../../../../assets/config/defaults.difficulty.ron");
    let defaults: DifficultyCurveDefaults =
        ron::de::from_str(ron_str).expect("difficulty.ron should parse as DifficultyCurveDefaults");
    assert_eq!(defaults.tiers.len(), 5);
    assert!((defaults.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON);
}

// -- HighlightDefaults deserialization --

#[test]
fn highlight_defaults_deserializes_all_34_fields_from_ron() {
    let ron_str = "
(
    clutch_clear_secs: 4.0,
    fast_clear_fraction: 0.4,
    perfect_streak_count: 6,
    mass_destruction_count: 12,
    mass_destruction_window_secs: 1.5,
    combo_king_cells: 10,
    pinball_wizard_bounces: 15,
    speed_demon_secs: 6.0,
    close_save_pixels: 25.0,
    comeback_bolts_lost: 4,
    nail_biter_pixels: 35.0,
    untouchable_nodes: 3,
    highlight_cap: 7,
    popup_max_visible: 5,
    popup_fade_duration_secs: 1.0,
    popup_overshoot_duration_secs: 0.2,
    popup_overshoot_scale: 1.25,
    popup_base_y: 120.0,
    popup_vertical_spacing: 60.0,
    popup_jitter_min_x: -15.0,
    popup_jitter_max_x: 15.0,
    popup_cascade_stagger_secs: 0.15,
    diversity_penalty: 0.6,
    max_expected_clutch_clear: 12.0,
    max_expected_speed_demon: 12.0,
    max_expected_close_save: 12.0,
    max_expected_nail_biter: 12.0,
    max_expected_mass_destruction: 6.0,
    max_expected_perfect_streak: 5.0,
    max_expected_combo_king: 5.0,
    max_expected_pinball_wizard: 5.0,
    max_expected_untouchable: 6.0,
    max_expected_comeback: 4.0,
    max_expected_perfect_node: 25.0,
)";
    let defaults: HighlightDefaults =
        ron::de::from_str(ron_str).expect("HighlightDefaults should deserialize");
    // Original 13 fields
    assert!((defaults.clutch_clear_secs - 4.0).abs() < f32::EPSILON);
    assert!((defaults.fast_clear_fraction - 0.4).abs() < f32::EPSILON);
    assert_eq!(defaults.perfect_streak_count, 6);
    assert_eq!(defaults.mass_destruction_count, 12);
    assert!((defaults.mass_destruction_window_secs - 1.5).abs() < f32::EPSILON);
    assert_eq!(defaults.combo_king_cells, 10);
    assert_eq!(defaults.pinball_wizard_bounces, 15);
    assert!((defaults.speed_demon_secs - 6.0).abs() < f32::EPSILON);
    assert!((defaults.close_save_pixels - 25.0).abs() < f32::EPSILON);
    assert_eq!(defaults.comeback_bolts_lost, 4);
    assert!((defaults.nail_biter_pixels - 35.0).abs() < f32::EPSILON);
    assert_eq!(defaults.untouchable_nodes, 3);
    assert_eq!(defaults.highlight_cap, 7);
    // Popup fields (9)
    assert_eq!(defaults.popup_max_visible, 5);
    assert!((defaults.popup_fade_duration_secs - 1.0).abs() < f32::EPSILON);
    assert!((defaults.popup_overshoot_duration_secs - 0.2).abs() < f32::EPSILON);
    assert!((defaults.popup_overshoot_scale - 1.25).abs() < f32::EPSILON);
    assert!((defaults.popup_base_y - 120.0).abs() < f32::EPSILON);
    assert!((defaults.popup_vertical_spacing - 60.0).abs() < f32::EPSILON);
    assert!((defaults.popup_jitter_min_x - -15.0).abs() < f32::EPSILON);
    assert!((defaults.popup_jitter_max_x - 15.0).abs() < f32::EPSILON);
    assert!((defaults.popup_cascade_stagger_secs - 0.15).abs() < f32::EPSILON);
    // Scoring fields (12)
    assert!((defaults.diversity_penalty - 0.6).abs() < f32::EPSILON);
    assert!((defaults.max_expected_clutch_clear - 12.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_speed_demon - 12.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_close_save - 12.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_nail_biter - 12.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_mass_destruction - 6.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_perfect_streak - 5.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_combo_king - 5.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_pinball_wizard - 5.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_untouchable - 6.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_comeback - 4.0).abs() < f32::EPSILON);
    assert!((defaults.max_expected_perfect_node - 25.0).abs() < f32::EPSILON);
}

#[test]
fn highlights_ron_file_parses() {
    let ron_str = include_str!("../../../../assets/config/defaults.highlights.ron");
    let defaults: HighlightDefaults =
        ron::de::from_str(ron_str).expect("defaults.highlights.ron should parse");
    assert!(
        defaults.clutch_clear_secs > 0.0,
        "clutch_clear_secs should be positive"
    );
    assert!(
        defaults.highlight_cap > 0,
        "highlight_cap should be positive"
    );
    // Popup fields are present and positive
    assert!(
        defaults.popup_max_visible > 0,
        "popup_max_visible should be positive"
    );
    assert!(
        defaults.popup_fade_duration_secs > 0.0,
        "popup_fade_duration_secs should be positive"
    );
    assert!(
        defaults.popup_vertical_spacing > 0.0,
        "popup_vertical_spacing should be positive"
    );
    // Scoring fields are present and positive
    assert!(
        defaults.diversity_penalty > 0.0,
        "diversity_penalty should be positive"
    );
    assert!(
        defaults.max_expected_clutch_clear > 0.0,
        "max_expected_clutch_clear should be positive"
    );
}

#[test]
fn highlight_config_from_defaults_copies_all_34_fields() {
    let defaults = HighlightDefaults {
        clutch_clear_secs: 2.5,
        fast_clear_fraction: 0.35,
        perfect_streak_count: 4,
        mass_destruction_count: 8,
        mass_destruction_window_secs: 1.0,
        combo_king_cells: 6,
        pinball_wizard_bounces: 10,
        speed_demon_secs: 4.0,
        close_save_pixels: 15.0,
        comeback_bolts_lost: 2,
        nail_biter_pixels: 25.0,
        untouchable_nodes: 3,
        highlight_cap: 4,
        // Popup fields
        popup_max_visible: 5,
        popup_fade_duration_secs: 1.2,
        popup_overshoot_duration_secs: 0.15,
        popup_overshoot_scale: 1.2,
        popup_base_y: 110.0,
        popup_vertical_spacing: 55.0,
        popup_jitter_min_x: -12.0,
        popup_jitter_max_x: 12.0,
        popup_cascade_stagger_secs: 0.12,
        // Scoring fields
        diversity_penalty: 0.4,
        max_expected_clutch_clear: 8.0,
        max_expected_speed_demon: 8.0,
        max_expected_close_save: 8.0,
        max_expected_nail_biter: 8.0,
        max_expected_mass_destruction: 4.0,
        max_expected_perfect_streak: 3.0,
        max_expected_combo_king: 3.0,
        max_expected_pinball_wizard: 3.0,
        max_expected_untouchable: 4.0,
        max_expected_comeback: 2.0,
        max_expected_perfect_node: 15.0,
    };

    let config = HighlightConfig::from(defaults);

    // Original 13 fields
    assert!((config.clutch_clear_secs - 2.5).abs() < f32::EPSILON);
    assert!((config.fast_clear_fraction - 0.35).abs() < f32::EPSILON);
    assert_eq!(config.perfect_streak_count, 4);
    assert_eq!(config.mass_destruction_count, 8);
    assert!((config.mass_destruction_window_secs - 1.0).abs() < f32::EPSILON);
    assert_eq!(config.combo_king_cells, 6);
    assert_eq!(config.pinball_wizard_bounces, 10);
    assert!((config.speed_demon_secs - 4.0).abs() < f32::EPSILON);
    assert!((config.close_save_pixels - 15.0).abs() < f32::EPSILON);
    assert_eq!(config.comeback_bolts_lost, 2);
    assert!((config.nail_biter_pixels - 25.0).abs() < f32::EPSILON);
    assert_eq!(config.untouchable_nodes, 3);
    assert_eq!(config.highlight_cap, 4);
    // Popup fields (9)
    assert_eq!(config.popup_max_visible, 5);
    assert!((config.popup_fade_duration_secs - 1.2).abs() < f32::EPSILON);
    assert!((config.popup_overshoot_duration_secs - 0.15).abs() < f32::EPSILON);
    assert!((config.popup_overshoot_scale - 1.2).abs() < f32::EPSILON);
    assert!((config.popup_base_y - 110.0).abs() < f32::EPSILON);
    assert!((config.popup_vertical_spacing - 55.0).abs() < f32::EPSILON);
    assert!((config.popup_jitter_min_x - -12.0).abs() < f32::EPSILON);
    assert!((config.popup_jitter_max_x - 12.0).abs() < f32::EPSILON);
    assert!((config.popup_cascade_stagger_secs - 0.12).abs() < f32::EPSILON);
    // Scoring fields (12)
    assert!((config.diversity_penalty - 0.4).abs() < f32::EPSILON);
    assert!((config.max_expected_clutch_clear - 8.0).abs() < f32::EPSILON);
    assert!((config.max_expected_speed_demon - 8.0).abs() < f32::EPSILON);
    assert!((config.max_expected_close_save - 8.0).abs() < f32::EPSILON);
    assert!((config.max_expected_nail_biter - 8.0).abs() < f32::EPSILON);
    assert!((config.max_expected_mass_destruction - 4.0).abs() < f32::EPSILON);
    assert!((config.max_expected_perfect_streak - 3.0).abs() < f32::EPSILON);
    assert!((config.max_expected_combo_king - 3.0).abs() < f32::EPSILON);
    assert!((config.max_expected_pinball_wizard - 3.0).abs() < f32::EPSILON);
    assert!((config.max_expected_untouchable - 4.0).abs() < f32::EPSILON);
    assert!((config.max_expected_comeback - 2.0).abs() < f32::EPSILON);
    assert!((config.max_expected_perfect_node - 15.0).abs() < f32::EPSILON);
}
