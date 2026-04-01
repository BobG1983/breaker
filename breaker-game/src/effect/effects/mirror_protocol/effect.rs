//! `MirrorProtocol` effect -- spawns mirrored bolts reflected across the last impact surface.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::{
        components::{Bolt, ImpactSide, LastImpact},
        resources::BoltConfig,
    },
    effect::BoundEffects,
};

/// Spawns a mirrored bolt reflected across the bolt's last impact surface.
///
/// Reads `LastImpact` from the bolt entity to determine the mirror axis.
/// Top/Bottom impacts mirror X position and negate X velocity.
/// Left/Right impacts mirror Y position and negate Y velocity.
///
/// The spawned bolt gets full physics components via the `Bolt` builder,
/// then its velocity is overwritten with the deterministic mirror velocity.
/// If `inherit` is true, `BoundEffects` from the source bolt are cloned onto
/// the spawned bolt.
pub(crate) fn fire(entity: Entity, inherit: bool, _source_chip: &str, world: &mut World) {
    // Guard: despawned entity
    if world.get_entity(entity).is_err() {
        return;
    }

    // Guard: must be a bolt
    if world.get::<Bolt>(entity).is_none() {
        return;
    }

    // Guard: must have LastImpact
    let Some(last_impact) = world.get::<LastImpact>(entity).cloned() else {
        return;
    };

    // Read current position and velocity
    let bolt_pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
    let bolt_vel = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);

    // Compute side-dependent mirror position and velocity
    let (mirror_pos, mirror_vel) = match last_impact.side {
        ImpactSide::Top | ImpactSide::Bottom => (
            Vec2::new(
                2.0f32.mul_add(last_impact.position.x, -bolt_pos.x),
                bolt_pos.y,
            ),
            Vec2::new(-bolt_vel.x, bolt_vel.y),
        ),
        ImpactSide::Left | ImpactSide::Right => (
            Vec2::new(
                bolt_pos.x,
                2.0f32.mul_add(last_impact.position.y, -bolt_pos.y),
            ),
            Vec2::new(bolt_vel.x, -bolt_vel.y),
        ),
    };

    // Clone BoundEffects before spawning (if inherit)
    let bound_effects = if inherit {
        world.get::<BoundEffects>(entity).cloned()
    } else {
        None
    };

    // Spawn the mirrored bolt with the deterministic mirror velocity
    let config = world.resource::<BoltConfig>().clone();
    let bolt_id = Bolt::builder()
        .at_position(mirror_pos)
        .config(&config)
        .with_velocity(Velocity2D(mirror_vel))
        .extra()
        .spawn(world);

    // Inherit BoundEffects if requested
    if let Some(effects) = bound_effects {
        world.entity_mut(bolt_id).insert(effects);
    }
}

/// No-op -- mirrored bolts persist independently once spawned.
pub(crate) const fn reverse(
    _entity: Entity,
    _inherit: bool,
    _source_chip: &str,
    _world: &mut World,
) {
}

/// Registers systems for `MirrorProtocol` effect.
pub(crate) const fn register(_app: &mut App) {}
