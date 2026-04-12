//! Phantom bolt systems — tick lifetime countdown.

use bevy::prelude::*;

use super::components::{PhantomBolt, PhantomLifetime};

/// Decrements phantom bolt lifetime each frame and despawns expired phantoms.
pub fn tick_phantom_lifetime(
    mut query: Query<(Entity, &mut PhantomLifetime), With<PhantomBolt>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
