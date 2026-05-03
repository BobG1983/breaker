use bevy::prelude::*;

use super::{components::PhantomBreakerLifetime, config::AfterimageConfig};
use crate::{
    breaker::{
        builder::core::types::BreakerPhantomParams,
        components::{BaseHeight, BaseWidth, DashState, PhantomBreaker},
        definition::BreakerDefinition,
    },
    prelude::*,
};

// ── System 1 — afterimage_spawn_phantom_breaker ─────────────────────────────

/// Default phantom AABB dimensions used when the real breaker does not
/// carry `BaseWidth` / `BaseHeight`. Matches the canonical breaker
/// footprint used by the phantom-breaker test fixtures so the AABB overlap
/// check in `afterimage_check_phantom_bounce` remains geometrically
/// consistent when minimal breakers (e.g. in harness tests) don't supply
/// the size components.
const DEFAULT_PHANTOM_BASE_WIDTH: f32 = 100.0;
/// Companion to [`DEFAULT_PHANTOM_BASE_WIDTH`].
const DEFAULT_PHANTOM_BASE_HEIGHT: f32 = 20.0;

const PHANTOM_COLOR_RGB: [f32; 3] = [0.4, 0.8, 1.0];
const PHANTOM_FLICKER_FREQUENCY: f32 = 4.0;
const PHANTOM_FLICKER_MIN_ALPHA: f32 = 0.3;

type SpawnPhantomBreakerBreakerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static DashState,
        &'static Position2D,
        Option<&'static BaseWidth>,
        Option<&'static BaseHeight>,
    ),
    (With<Breaker>, Without<PhantomBreaker>),
>;

/// Detects the `DashState` rising edge into `Dashing` and spawns a
/// `PhantomBreaker` entity at the breaker's current position via
/// `Breaker::builder()...phantom(...).rendered(...).extra().spawn(...)`.
///
/// Despawns any existing `PhantomBreaker` entity before spawning a new one
/// so at most one phantom breaker exists globally at a time. Copies the
/// breaker's RAW `BaseWidth` / `BaseHeight` into the phantom — no size-boost
/// or node-scale multiplication. If the breaker does not carry those
/// components the phantom falls back to [`DEFAULT_PHANTOM_BASE_WIDTH`] /
/// [`DEFAULT_PHANTOM_BASE_HEIGHT`].
///
/// Harness-safe: early-returns without touching `prev_state` when
/// `AfterimageConfig` is absent or when the single-breaker query fails.
pub(crate) fn afterimage_spawn_phantom_breaker(
    config: Option<Res<AfterimageConfig>>,
    breakers: SpawnPhantomBreakerBreakerQuery,
    existing_phantoms: Query<Entity, With<PhantomBreaker>>,
    mut prev_state: Local<Option<DashState>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(config) = config else {
        return;
    };
    let Ok((state, position, base_width, base_height)) = breakers.single() else {
        // Zero or multi breakers — silently skip, do not update `prev_state`.
        return;
    };
    let current = *state;
    let was_dashing = matches!(*prev_state, Some(DashState::Dashing));
    let is_rising = !was_dashing && current == DashState::Dashing;

    if is_rising {
        for existing in &existing_phantoms {
            commands.entity(existing).despawn();
        }
        let width = base_width.map_or(DEFAULT_PHANTOM_BASE_WIDTH, |w| w.0);
        let height = base_height.map_or(DEFAULT_PHANTOM_BASE_HEIGHT, |h| h.0);
        let phantom_def = BreakerDefinition::default();
        let entity = Breaker::builder()
            .definition(&phantom_def)
            .with_width(width)
            .with_height(height)
            .at_position(position.0)
            .phantom(BreakerPhantomParams {
                lifespan:          config.phantom_duration,
                phantom_color_rgb: PHANTOM_COLOR_RGB,
                flicker_frequency: PHANTOM_FLICKER_FREQUENCY,
                flicker_min_alpha: PHANTOM_FLICKER_MIN_ALPHA,
            })
            .rendered(&mut meshes, &mut materials)
            .extra()
            .spawn(&mut commands);
        commands
            .entity(entity)
            .insert(PhantomBreakerLifetime(config.phantom_duration));
    }

    *prev_state = Some(current);
}
