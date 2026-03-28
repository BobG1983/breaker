//! Breaker initialization systems — config overrides and component stamping.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    breaker::{
        SelectedBreaker,
        components::{Breaker, BreakerInitialized},
        definition::BreakerStatOverrides,
        queries::InitBreakerQuery,
        registry::BreakerRegistry,
        resources::{BreakerConfig, BreakerDefaults},
    },
    effect::{RootEffect, Target, effects::life_lost::LivesCount},
};

/// Applies optional stat overrides to a `BreakerConfig`.
///
/// Each `Some` field in `overrides` replaces the corresponding field in `config`.
/// Used by both `apply_breaker_config_overrides` (at init) and hot-reload propagation (at runtime).
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

/// Stamps init-time behavior components (`LivesCount`, `BoundEffects`, `StagedEffects`).
///
/// Runs `OnEnter(GameState::Playing)` AFTER `init_breaker_params`.
pub(crate) fn init_breaker(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    mut breaker_query: InitBreakerQuery,
) {
    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };

    for (entity, mut bound) in &mut breaker_query {
        commands.entity(entity).insert(BreakerInitialized);
        if let Some(life_pool) = def.life_pool {
            commands.entity(entity).insert(LivesCount(life_pool));
        }
        for root_effect in &def.effects {
            let RootEffect::On { target, then } = root_effect;
            if *target == Target::Breaker {
                for child in then {
                    bound.0.push((String::new(), child.clone()));
                }
            }
        }
    }
}

/// Dispatches breaker-defined effects to target entities.
///
/// Resolves `RootEffect::On { target, then }` from the breaker definition
/// and pushes children to target entity's `BoundEffects`.
/// Stub — real implementation in Wave 6.
pub(crate) fn dispatch_breaker_effects(
    mut _commands: Commands,
    _selected: Res<SelectedBreaker>,
    _registry: Res<BreakerRegistry>,
    _breaker_query: Query<Entity, With<Breaker>>,
) {
    // TODO: Wave 6 — resolve RootEffect targets, push to BoundEffects,
    // fire bare Do children via commands.fire_effect()
}
