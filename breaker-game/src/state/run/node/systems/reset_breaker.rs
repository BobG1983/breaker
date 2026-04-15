//! System to reset breaker state at the start of each node.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::PreviousPosition;

use crate::{
    breaker::{components::DashState, messages::BreakerSpawned, queries::BreakerResetData},
    prelude::*,
};

/// Resets breaker state at the start of each node.
///
/// Runs when entering [`GameState::Playing`]. Returns breaker to center,
/// clears velocity/tilt/state. On the first node, `spawn_breaker` handles
/// initialization — this system is a no-op if no breaker exists yet.
pub(crate) fn reset_breaker(
    playfield: Res<PlayfieldConfig>,
    mut query: Query<BreakerResetData, With<Breaker>>,
    mut spawned: MessageWriter<BreakerSpawned>,
) {
    // Robust if PlayfieldConfig is ever offset from world origin
    let center_x = f32::midpoint(playfield.left(), playfield.right());
    let mut any_reset = false;
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
        any_reset = true;
    }
    if any_reset {
        spawned.write(BreakerSpawned);
    }
}
