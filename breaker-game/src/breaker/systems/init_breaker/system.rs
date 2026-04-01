//! Breaker initialization systems — config overrides and component stamping.

use bevy::prelude::*;
use tracing::warn;

#[cfg(test)]
use crate::breaker::resources::BreakerDefaults;
#[cfg(any(test, feature = "dev"))]
use crate::breaker::{definition::BreakerStatOverrides, resources::BreakerConfig};
use crate::{
    breaker::{
        SelectedBreaker,
        components::{Breaker, BreakerInitialized},
        registry::BreakerRegistry,
    },
    effect::effects::life_lost::LivesCount,
};

/// Applies optional stat overrides to a `BreakerConfig`.
///
/// Each `Some` field in `overrides` replaces the corresponding field in `config`.
/// Used by both `apply_breaker_config_overrides` (at init) and hot-reload propagation (at runtime).
#[cfg(any(test, feature = "dev"))]
pub(crate) const fn apply_stat_overrides(
    config: &mut BreakerConfig,
    overrides: &BreakerStatOverrides,
) {
    if let Some(width) = overrides.width {
        config.width = width;
    }
    if let Some(height) = overrides.height {
        config.height = height;
    }
    if let Some(max_speed) = overrides.max_speed {
        config.max_speed = max_speed;
    }
    if let Some(acceleration) = overrides.acceleration {
        config.acceleration = acceleration;
    }
    if let Some(deceleration) = overrides.deceleration {
        config.deceleration = deceleration;
    }
}

/// Resets `BreakerConfig` from defaults and applies breaker stat overrides.
///
/// Runs `OnEnter(GameState::Playing)` BEFORE `init_breaker_params` so that
/// stamped components reflect the overridden config values.
#[cfg(test)]
pub(crate) fn apply_breaker_config_overrides(
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    defaults: Res<Assets<BreakerDefaults>>,
    mut config: ResMut<BreakerConfig>,
) {
    // Reset config from loaded RON defaults (not code defaults)
    if let Some(loaded) = defaults.iter().next().map(|(_, d)| d) {
        *config = BreakerConfig::from(loaded.clone());
    }

    // Apply breaker overrides
    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };

    apply_stat_overrides(&mut config, &def.stat_overrides);
}

/// Stamps init-time behavior components (`LivesCount`, `BreakerInitialized`).
///
/// Runs `OnEnter(GameState::Playing)` AFTER `init_breaker_params`.
pub(crate) fn init_breaker(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    breaker_query: Query<Entity, (With<Breaker>, Without<BreakerInitialized>)>,
) {
    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };

    for entity in &breaker_query {
        commands.entity(entity).insert(BreakerInitialized);
        commands.entity(entity).insert(LivesCount(def.life_pool));
    }
}
