//! System to spawn the breaker entity.

use bevy::prelude::*;

use crate::breaker::{
    BreakerRegistry, SelectedBreaker, components::Breaker, messages::BreakerSpawned,
};

/// Spawns or reuses the breaker entity using the builder.
///
/// Runs when entering [`GameState::Playing`]. If a breaker already exists
/// (persisted from a previous node), this sends `BreakerSpawned` without
/// spawning a new one. Otherwise, looks up the selected breaker in the
/// registry and spawns via `Breaker::builder().definition(def)`.
pub(crate) fn spawn_or_reuse_breaker(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing: Query<Entity, With<Breaker>>,
    mut breaker_spawned: MessageWriter<BreakerSpawned>,
) {
    if !existing.is_empty() {
        breaker_spawned.write(BreakerSpawned);
        return;
    }
    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };
    Breaker::builder()
        .definition(def)
        .rendered(&mut meshes, &mut materials)
        .primary()
        .spawn(&mut commands);
    breaker_spawned.write(BreakerSpawned);
}
