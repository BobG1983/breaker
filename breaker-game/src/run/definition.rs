//! Tier-based difficulty curve and highlight thresholds — RON-deserialized content data types.

use bevy::prelude::*;
use rantzsoft_defaults::GameConfig;
use serde::Deserialize;

/// The type of a node in the run sequence.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// A passive node — no active timer pressure.
    Passive,
    /// An active node — timer ticks down during play.
    Active,
    /// A boss node — harder encounter at the end of a tier.
    Boss,
}

/// How many nodes a tier contains — fixed count or a range.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierNodeCount {
    /// Exactly this many nodes.
    Fixed(u32),
    /// Between min and max nodes (inclusive).
    Range(u32, u32),
}

impl TierNodeCount {
    /// Validates that the count specification is well-formed.
    ///
    /// # Errors
    ///
    /// Returns an error string if the range has min greater than max.
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Range(min, max) = self
            && min > max
        {
            return Err(format!(
                "TierNodeCount::Range min ({min}) must not be greater than max ({max})"
            ));
        }
        Ok(())
    }
}

/// Definition of a single difficulty tier loaded from RON.
#[derive(Deserialize, Clone, Debug)]
pub struct TierDefinition {
    /// How many nodes this tier contains.
    pub nodes: TierNodeCount,
    /// Fraction of nodes in this tier that are active (0.0 to 1.0).
    pub active_ratio: f32,
    /// Hit-point multiplier applied to cells in this tier.
    pub hp_mult: f32,
    /// Timer multiplier applied to node timers in this tier.
    pub timer_mult: f32,
    /// Cell-type aliases introduced in this tier.
    pub introduced_cells: Vec<char>,
}

/// Difficulty curve defaults loaded from `difficulty.ron`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct DifficultyCurveDefaults {
    /// Ordered list of tier definitions.
    pub tiers: Vec<TierDefinition>,
    /// HP multiplier applied to boss nodes.
    pub boss_hp_mult: f32,
    /// Timer reduction applied after each boss encounter.
    pub timer_reduction_per_boss: f32,
}

/// Highlight detection thresholds loaded from `defaults.highlights.ron`.
///
/// The `GameConfig` derive generates a `HighlightConfig` resource with `From<HighlightDefaults>`.
#[derive(Asset, TypePath, Deserialize, Clone, Debug, GameConfig)]
#[game_config(name = "HighlightConfig")]
pub struct HighlightDefaults {
    /// Seconds remaining for `ClutchClear` detection.
    pub clutch_clear_secs: f32,
    /// Fraction of total time for `FastClear` detection.
    pub fast_clear_fraction: f32,
    /// Consecutive perfect bumps for `PerfectStreak`.
    pub perfect_streak_count: u32,
    /// Cells destroyed in window for `MassDestruction`.
    pub mass_destruction_count: u32,
    /// Window duration (seconds) for `MassDestruction`.
    pub mass_destruction_window_secs: f32,
    /// Cells destroyed between breaker impacts for `ComboKing`.
    pub combo_king_cells: u32,
    /// Cell bounces without breaker for `PinballWizard`.
    pub pinball_wizard_bounces: u32,
    /// Seconds for fastest node clear (`SpeedDemon`).
    pub speed_demon_secs: f32,
    /// Pixels from bottom boundary for `CloseSave`.
    pub close_save_pixels: f32,
    /// Bolts lost in node for `Comeback`.
    pub comeback_bolts_lost: u32,
    /// Pixels from bottom boundary for `NailBiter`.
    pub nail_biter_pixels: f32,
    /// Consecutive no-damage nodes for `Untouchable`.
    pub untouchable_nodes: u32,
    /// Maximum highlights recorded per run.
    pub highlight_cap: u32,

    // -- Popup animation --
    /// Maximum visible highlight popups at once.
    pub popup_max_visible: u32,
    /// Popup fade-out duration (seconds).
    pub popup_fade_duration_secs: f32,
    /// Popup scale overshoot animation duration (seconds).
    pub popup_overshoot_duration_secs: f32,
    /// Popup initial scale overshoot multiplier (e.g. 1.15).
    pub popup_overshoot_scale: f32,
    /// Popup base y-position (above playfield center).
    pub popup_base_y: f32,
    /// Vertical spacing between stacked popups.
    pub popup_vertical_spacing: f32,
    /// Minimum horizontal jitter (pixels, can be negative).
    pub popup_jitter_min_x: f32,
    /// Maximum horizontal jitter (pixels, can be positive).
    pub popup_jitter_max_x: f32,
    /// Cascade stagger delay between simultaneous popups (seconds).
    pub popup_cascade_stagger_secs: f32,

    // -- Scoring --
    /// Diversity penalty base for same-category highlights (0.0–1.0).
    pub diversity_penalty: f32,
    /// Max expected raw score for `ClutchClear` normalization.
    pub max_expected_clutch_clear: f32,
    /// Max expected raw score for `SpeedDemon` normalization.
    pub max_expected_speed_demon: f32,
    /// Max expected raw score for `CloseSave` normalization.
    pub max_expected_close_save: f32,
    /// Max expected raw score for `NailBiter` normalization.
    pub max_expected_nail_biter: f32,
    /// Max expected raw score for `MassDestruction` normalization.
    pub max_expected_mass_destruction: f32,
    /// Max expected raw score for `PerfectStreak` normalization.
    pub max_expected_perfect_streak: f32,
    /// Max expected raw score for `ComboKing` normalization.
    pub max_expected_combo_king: f32,
    /// Max expected raw score for `PinballWizard` normalization.
    pub max_expected_pinball_wizard: f32,
    /// Max expected raw score for `Untouchable` normalization.
    pub max_expected_untouchable: f32,
    /// Max expected raw score for `Comeback` normalization.
    pub max_expected_comeback: f32,
    /// Max expected raw score for `PerfectNode` normalization.
    pub max_expected_perfect_node: f32,
}

impl Default for HighlightDefaults {
    fn default() -> Self {
        Self {
            clutch_clear_secs: 3.0,
            fast_clear_fraction: 0.5,
            perfect_streak_count: 5,
            mass_destruction_count: 10,
            mass_destruction_window_secs: 2.0,
            combo_king_cells: 8,
            pinball_wizard_bounces: 12,
            speed_demon_secs: 5.0,
            close_save_pixels: 20.0,
            comeback_bolts_lost: 3,
            nail_biter_pixels: 30.0,
            untouchable_nodes: 2,
            highlight_cap: 5,
            // Popup animation
            popup_max_visible: 3,
            popup_fade_duration_secs: 0.8,
            popup_overshoot_duration_secs: 0.1,
            popup_overshoot_scale: 1.15,
            popup_base_y: 100.0,
            popup_vertical_spacing: 50.0,
            popup_jitter_min_x: -10.0,
            popup_jitter_max_x: 10.0,
            popup_cascade_stagger_secs: 0.1,
            // Scoring
            diversity_penalty: 0.5,
            max_expected_clutch_clear: 10.0,
            max_expected_speed_demon: 10.0,
            max_expected_close_save: 10.0,
            max_expected_nail_biter: 10.0,
            max_expected_mass_destruction: 5.0,
            max_expected_perfect_streak: 4.0,
            max_expected_combo_king: 4.0,
            max_expected_pinball_wizard: 4.0,
            max_expected_untouchable: 5.0,
            max_expected_comeback: 3.0,
            max_expected_perfect_node: 20.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result: TierNodeCount =
            ron::de::from_str("Fixed(5)").expect("Fixed(5) should deserialize");
        assert_eq!(result, TierNodeCount::Fixed(5));
    }

    #[test]
    fn tier_node_count_fixed_zero_deserializes_from_ron() {
        let result: TierNodeCount =
            ron::de::from_str("Fixed(0)").expect("Fixed(0) should deserialize");
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
        let ron_str = "(nodes: Range(4, 6), active_ratio: 0.2, hp_mult: 1.3, timer_mult: 0.9, introduced_cells: ['T'])";
        let tier: TierDefinition =
            ron::de::from_str(ron_str).expect("TierDefinition should deserialize");
        assert_eq!(tier.nodes, TierNodeCount::Range(4, 6));
        assert!((tier.active_ratio - 0.2).abs() < f32::EPSILON);
        assert!((tier.hp_mult - 1.3).abs() < f32::EPSILON);
        assert!((tier.timer_mult - 0.9).abs() < f32::EPSILON);
        assert_eq!(tier.introduced_cells, vec!['T']);
    }

    #[test]
    fn tier_definition_empty_introduced_cells_deserializes() {
        let ron_str = "(nodes: Fixed(3), active_ratio: 0.0, hp_mult: 1.0, timer_mult: 1.0, introduced_cells: [])";
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
        (nodes: Fixed(3), active_ratio: 0.0, hp_mult: 1.0, timer_mult: 1.0, introduced_cells: []),
        (nodes: Range(4, 6), active_ratio: 0.5, hp_mult: 1.5, timer_mult: 0.8, introduced_cells: ['T']),
    ],
    boss_hp_mult: 3.0,
    timer_reduction_per_boss: 0.1,
)";
        let defaults: DifficultyCurveDefaults =
            ron::de::from_str(ron_str).expect("DifficultyCurveDefaults should deserialize");
        assert_eq!(defaults.tiers.len(), 2);
        assert!((defaults.boss_hp_mult - 3.0).abs() < f32::EPSILON);
        assert!((defaults.timer_reduction_per_boss - 0.1).abs() < f32::EPSILON);
        assert_eq!(defaults.tiers[0].nodes, TierNodeCount::Fixed(3));
        assert!((defaults.tiers[0].active_ratio - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn difficulty_curve_defaults_empty_tiers_deserializes() {
        let ron_str = "
(
    tiers: [],
    boss_hp_mult: 2.0,
    timer_reduction_per_boss: 0.05,
)";
        let defaults: DifficultyCurveDefaults =
            ron::de::from_str(ron_str).expect("empty tiers should deserialize");
        assert!(defaults.tiers.is_empty());
    }

    // -- difficulty.ron file parse --

    #[test]
    fn difficulty_ron_file_parses() {
        let ron_str = include_str!("../../assets/config/defaults.difficulty.ron");
        let defaults: DifficultyCurveDefaults = ron::de::from_str(ron_str)
            .expect("difficulty.ron should parse as DifficultyCurveDefaults");
        assert_eq!(defaults.tiers.len(), 5);
        assert!((defaults.boss_hp_mult - 3.0).abs() < f32::EPSILON);
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
        let ron_str = include_str!("../../assets/config/defaults.highlights.ron");
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
}
