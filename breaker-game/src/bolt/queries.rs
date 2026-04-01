//! Bolt domain query data types.

use bevy::{
    ecs::query::{Has, QueryData},
    prelude::*,
};
use rantzsoft_spatial2d::{
    components::{BaseSpeed, PreviousPosition},
    queries::{SpatialData, SpatialDataItem},
};

use crate::{
    bolt::components::{
        BoltAngleSpread, BoltBaseDamage, BoltRadius, BoltSpawnOffsetY, ExtraBolt, LastImpact,
        PiercingRemaining, SpawnedByEvolution,
    },
    effect::effects::{
        damage_boost::ActiveDamageBoosts, piercing::ActivePiercings, speed_boost::ActiveSpeedBoosts,
    },
    shared::NodeScalingFactor,
};

/// Collision-specific bolt data: radius, piercing, damage, speed boosts,
/// scale, evolution attribution, and impact tracking.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BoltCollisionParams {
    /// Bolt radius in world units.
    pub radius: &'static BoltRadius,
    /// Remaining pierce charges (decremented on cell pierce-through).
    pub piercing_remaining: Option<&'static mut PiercingRemaining>,
    /// Active piercing effects (sum determines max charges).
    pub active_piercings: Option<&'static ActivePiercings>,
    /// Active damage boost multipliers.
    pub active_damage_boosts: Option<&'static ActiveDamageBoosts>,
    /// Active speed boost multipliers.
    pub active_speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Visual/physics scale factor.
    pub entity_scale: Option<&'static NodeScalingFactor>,
    /// Evolution chip that spawned this bolt (for damage attribution).
    pub spawned_by_evolution: Option<&'static SpawnedByEvolution>,
    /// Last collision impact position and side.
    pub last_impact: Option<&'static mut LastImpact>,
    /// Per-bolt base damage (from definition). Falls back to `DEFAULT_BOLT_BASE_DAMAGE` if absent.
    pub base_damage: Option<&'static BoltBaseDamage>,
}

/// Full collision data for bolt entities. Composes [`SpatialData`] from the
/// spatial crate with bolt-specific [`BoltCollisionParams`].
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BoltCollisionData {
    /// The bolt entity.
    pub entity: Entity,
    /// Spatial position, velocity, and constraint data.
    pub spatial: SpatialData,
    /// Bolt-specific collision parameters.
    pub collision: BoltCollisionParams,
}

/// Bolt entity data needed by the reset system at node start.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct ResetBoltData {
    /// The bolt entity.
    pub entity: Entity,
    /// Spatial position, velocity, and constraint data.
    pub spatial: SpatialData,
    /// Speed boost multipliers for the velocity formula.
    pub active_speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Remaining pierce charges to reset.
    pub piercing_remaining: Option<&'static mut PiercingRemaining>,
    /// Active piercing effects (sum determines reset value).
    pub active_piercings: Option<&'static ActivePiercings>,
    /// Previous position snapshot (reset to prevent interpolation teleport).
    pub previous_position: Option<&'static mut PreviousPosition>,
    /// Angle spread from vertical for launch/respawn (from definition).
    pub angle_spread: Option<&'static BoltAngleSpread>,
    /// Vertical offset above breaker for spawn position (from definition).
    pub spawn_offset: &'static BoltSpawnOffsetY,
}

/// Bolt entity data needed by the bolt-lost detection system.
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct LostBoltData {
    /// The bolt entity.
    pub entity: Entity,
    /// Spatial position, velocity, and constraint data.
    pub spatial: SpatialData,
    /// Speed boost multipliers for the velocity formula.
    pub active_speed_boosts: Option<&'static ActiveSpeedBoosts>,
    /// Bolt radius for below-playfield detection.
    pub radius: &'static BoltRadius,
    /// Vertical offset above breaker for respawn.
    pub spawn_offset: &'static BoltSpawnOffsetY,
    /// Maximum respawn angle spread from vertical (from definition, optional with fallback).
    pub angle_spread: Option<&'static BoltAngleSpread>,
    /// Whether this is an extra bolt (despawned on loss, not respawned).
    pub is_extra: Has<ExtraBolt>,
    /// Visual/physics scale factor.
    pub entity_scale: Option<&'static NodeScalingFactor>,
}

/// Applies the canonical velocity formula to spatial data with optional
/// speed boost multiplier.
///
/// Clamps the angle then sets speed to
/// `(base_speed * boost_mult).clamp(min, max)`.
/// This is the single source of truth for bolt speed — every system that
/// modifies bolt velocity calls this after setting direction.
pub(crate) fn apply_velocity_formula(
    spatial: &mut SpatialDataItem<'_, '_>,
    active_speed_boosts: Option<&ActiveSpeedBoosts>,
) {
    let mult = active_speed_boosts.map_or(1.0, ActiveSpeedBoosts::multiplier);
    let effective_base = BaseSpeed(spatial.base_speed.0 * mult);
    *spatial.velocity = spatial.velocity.constrained(
        &effective_base,
        spatial.min_speed,
        spatial.max_speed,
        spatial.min_angle_h,
        spatial.min_angle_v,
    );
}
