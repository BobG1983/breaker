use super::super::data::*;

/// Builds a valid [`CellTypeDefinition`] with sensible defaults.
/// Individual tests override fields to test specific validation rules.
pub(super) fn valid_definition() -> CellTypeDefinition {
    CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::Standard,
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         None,
        effects:           None,
    }
}

pub(super) fn valid_guarded_behavior() -> GuardedBehavior {
    GuardedBehavior {
        guardian_hp_fraction: 0.5,
        guardian_color_rgb:   [0.5, 0.8, 1.0],
        slide_speed:          30.0,
    }
}
