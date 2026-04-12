use crate::cells::builder::core::types::GuardianSpawnConfig;
pub(super) use crate::cells::test_utils::{spawn_cell_in_world, test_cell_definition};

pub(super) fn test_guardian_config() -> GuardianSpawnConfig {
    GuardianSpawnConfig {
        hp:          10.0,
        color_rgb:   [0.5, 0.8, 1.0],
        slide_speed: 30.0,
        cell_height: 24.0,
        step_x:      72.0,
        step_y:      26.0,
    }
}
