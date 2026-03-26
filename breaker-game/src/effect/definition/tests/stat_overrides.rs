use bevy::prelude::*;

use super::super::*;
use crate::{breaker::resources::BreakerConfig, effect::init::apply_stat_overrides};

// =========================================================================
// Preserved tests
// =========================================================================

#[test]
fn default_stat_overrides_are_all_none() {
    let overrides = BreakerStatOverrides::default();
    assert!(overrides.width.is_none());
    assert!(overrides.height.is_none());
    assert!(overrides.max_speed.is_none());
    assert!(overrides.acceleration.is_none());
    assert!(overrides.deceleration.is_none());
}

#[test]
fn apply_stat_overrides_partial() {
    let mut config = BreakerConfig::default();
    let original_max_speed = config.max_speed;

    let overrides = BreakerStatOverrides {
        width: Some(200.0),
        height: Some(30.0),
        ..default()
    };

    apply_stat_overrides(&mut config, &overrides);

    assert!((config.width - 200.0).abs() < f32::EPSILON);
    assert!((config.height - 30.0).abs() < f32::EPSILON);
    assert!(
        (config.max_speed - original_max_speed).abs() < f32::EPSILON,
        "unset fields should remain unchanged"
    );
}
