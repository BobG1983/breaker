use bevy::prelude::*;
use breaker::{
    bolt::components::BoltRadius,
    breaker::components::{BaseHeight, BaseWidth},
};
use rantzsoft_physics2d::aabb::Aabb2D;

use crate::{invariants::*, types::InvariantKind};

/// Epsilon for floating-point comparison of AABB half-extents.
pub(super) const AABB_EPSILON: f32 = 0.001;

/// Query type alias for breaker AABB dimension checks.
///
/// `NodeScalingFactor` is intentionally excluded — the stored `Aabb2D` contains
/// unscaled base dimensions. Collision systems apply scale at runtime.
type BreakerAabbQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Aabb2D,
        &'static BaseWidth,
        &'static BaseHeight,
    ),
    With<ScenarioTagBreaker>,
>;

/// Checks that entity [`Aabb2D`] half-extents match expected dimensions.
///
/// **Bolts**: `half_extents` must approximately equal `Vec2::splat(BoltRadius.0)`.
/// **Breakers**: `half_extents` must approximately equal
/// `Vec2::new(BaseWidth.half_width(), BaseHeight.half_height())`.
/// `NodeScalingFactor` is NOT factored in — the stored `Aabb2D` holds unscaled base dimensions.
///
/// Uses strict greater-than (`>`) comparison against [`AABB_EPSILON`] — a delta
/// exactly equal to epsilon does NOT fire a violation.
pub fn check_aabb_matches_entity_dimensions(
    bolts: Query<(Entity, &Aabb2D, &BoltRadius), With<ScenarioTagBolt>>,
    breakers: BreakerAabbQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    for (entity, aabb, radius) in &bolts {
        let expected = Vec2::splat(radius.0);
        let actual = aabb.half_extents;
        if (actual - expected).abs().max_element() > AABB_EPSILON {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::AabbMatchesEntityDimensions,
                entity: Some(entity),
                message: format!(
                    "AabbMatchesEntityDimensions FAIL frame={} entity={entity:?} expected={expected:?} actual={actual:?}",
                    frame.0,
                ),
            });
        }
    }

    for (entity, aabb, width, height) in &breakers {
        let expected = Vec2::new(width.half_width(), height.half_height());
        let actual = aabb.half_extents;
        if (actual - expected).abs().max_element() > AABB_EPSILON {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::AabbMatchesEntityDimensions,
                entity: Some(entity),
                message: format!(
                    "AabbMatchesEntityDimensions FAIL frame={} entity={entity:?} expected={expected:?} actual={actual:?}",
                    frame.0,
                ),
            });
        }
    }
}
