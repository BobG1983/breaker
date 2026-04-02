//! Breaker initialization systems — config overrides and component stamping.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    breaker::{
        SelectedBreaker,
        components::{Breaker, BreakerInitialized},
        registry::BreakerRegistry,
    },
    effect::effects::life_lost::LivesCount,
};

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
