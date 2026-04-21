use bevy::prelude::*;

use super::{
    components::{PhantomBreaker, PhantomBreakerLifetime},
    config::AfterimageConfig,
};

// ── System 2 — afterimage_tick_phantom_breaker ──────────────────────────────

/// Decrements `PhantomBreakerLifetime.0` by `delta_secs` each tick and
/// despawns the phantom when the remaining lifetime drops to `<= 0.0`.
///
/// Harness-safe: early-returns without mutating any lifetime when
/// `AfterimageConfig` is absent.
pub(crate) fn afterimage_tick_phantom_breaker(
    time: Res<Time<Fixed>>,
    config: Option<Res<AfterimageConfig>>,
    mut phantoms: Query<(Entity, &mut PhantomBreakerLifetime), With<PhantomBreaker>>,
    mut commands: Commands,
) {
    let Some(_config) = config else {
        return;
    };
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut phantoms {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}
