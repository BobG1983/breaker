//! System to spawn the breaker entity.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::PreviousPosition;

use crate::{
    breaker::{
        BreakerRegistry, SelectedBreaker,
        components::{Breaker, DashState},
        messages::BreakerSpawned,
        queries::BreakerResetData,
    },
    shared::PlayfieldConfig,
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

/// Resets breaker state at the start of each node.
///
/// Runs when entering [`GameState::Playing`]. Returns breaker to center,
/// clears velocity/tilt/state. On the first node, `spawn_breaker` handles
/// initialization — this system is a no-op if no breaker exists yet.
pub(crate) fn reset_breaker(
    playfield: Res<PlayfieldConfig>,
    mut query: Query<BreakerResetData, With<Breaker>>,
) {
    // Robust if PlayfieldConfig is ever offset from world origin
    let center_x = f32::midpoint(playfield.left(), playfield.right());
    for mut data in &mut query {
        data.position.0.x = center_x;
        data.position.0.y = data.base_y.0;
        *data.state = DashState::Idle;
        data.velocity.0.x = 0.0;
        data.tilt.angle = 0.0;
        data.tilt.ease_start = 0.0;
        data.tilt.ease_target = 0.0;
        data.timer.remaining = 0.0;
        data.bump.active = false;
        data.bump.timer = 0.0;
        data.bump.post_hit_timer = 0.0;
        data.bump.cooldown = 0.0;
        // Snap interpolation to avoid lerping through teleport
        if let Some(mut prev) = data.prev_position {
            *prev = PreviousPosition(data.position.0);
        }
    }
}
