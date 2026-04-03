//! Highlight detection systems.

pub(crate) mod detect_close_save;
pub(crate) mod detect_combo_king;
pub(crate) mod detect_mass_destruction;
pub(crate) mod detect_nail_biter;
pub(crate) mod detect_pinball_wizard;

pub(crate) use detect_close_save::detect_close_save;
pub(crate) use detect_combo_king::detect_combo_king;
pub(crate) use detect_mass_destruction::detect_mass_destruction;
pub(crate) use detect_nail_biter::detect_nail_biter;
pub(crate) use detect_pinball_wizard::detect_pinball_wizard;
