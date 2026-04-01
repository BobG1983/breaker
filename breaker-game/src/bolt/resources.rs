//! Bolt domain resources.

/// Default base damage for a bolt hit. Fixed game-design constant.
pub const DEFAULT_BOLT_BASE_DAMAGE: f32 = 10.0;

/// Default vertical offset above breaker for bolt spawn (world units).
pub const DEFAULT_BOLT_SPAWN_OFFSET_Y: f32 = 54.0;

/// Default angle spread from vertical for launch and respawn (radians, ~30 degrees).
pub const DEFAULT_BOLT_ANGLE_SPREAD: f32 = 0.524;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 23: DEFAULT_BOLT_BASE_DAMAGE equals 10.0 ───────

    #[test]
    fn default_bolt_base_damage_equals_10() {
        assert!(
            (DEFAULT_BOLT_BASE_DAMAGE - 10.0_f32).abs() < f32::EPSILON,
            "DEFAULT_BOLT_BASE_DAMAGE should be 10.0, got {DEFAULT_BOLT_BASE_DAMAGE}"
        );
    }

    // ── Behavior 24: DEFAULT_BOLT_SPAWN_OFFSET_Y equals 54.0 ────

    #[test]
    fn default_bolt_spawn_offset_y_equals_54() {
        assert!(
            (DEFAULT_BOLT_SPAWN_OFFSET_Y - 54.0_f32).abs() < f32::EPSILON,
            "DEFAULT_BOLT_SPAWN_OFFSET_Y should be 54.0, got {DEFAULT_BOLT_SPAWN_OFFSET_Y}"
        );
    }

    // ── Behavior 25: DEFAULT_BOLT_ANGLE_SPREAD equals 0.524 ─────

    #[test]
    fn default_bolt_angle_spread_equals_0_524() {
        assert!(
            (DEFAULT_BOLT_ANGLE_SPREAD - 0.524_f32).abs() < f32::EPSILON,
            "DEFAULT_BOLT_ANGLE_SPREAD should be 0.524, got {DEFAULT_BOLT_ANGLE_SPREAD}"
        );
    }
}
