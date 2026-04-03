//! Bolt-lost detection — bolt falls below the playfield.

use bevy::{
    ecs::system::{SystemParam, SystemParamValidationError},
    prelude::*,
};
use rand::Rng;
use rantzsoft_spatial2d::components::PreviousPosition;

use crate::{
    bolt::{
        filters::ActiveFilter,
        messages::{BoltLost, RequestBoltDestroyed},
        queries::{LostBoltData, apply_velocity_formula},
    },
    breaker::filters::CollisionFilterBreaker,
    shared::{GameRng, PlayfieldConfig},
};

/// Bundled message writers for `bolt_lost` to satisfy clippy's
/// `too_many_arguments` lint.
#[derive(SystemParam)]
pub(crate) struct BoltLostWriters<'w> {
    writer: MessageWriter<'w, BoltLost>,
    request_destroyed_writer:
        Result<MessageWriter<'w, RequestBoltDestroyed>, SystemParamValidationError>,
}

/// Collected data for a single lost bolt, used as scratch storage between
/// the filter pass and the command-application pass.
#[derive(Clone, Copy)]
pub(crate) struct LostBoltEntry {
    entity: Entity,
    spawn_offset: f32,
    angle_spread: f32,
    is_extra: bool,
}

/// Detects when the bolt falls below the playfield.
///
/// Baseline bolts (without [`ExtraBolt`]) are respawned above the breaker.
/// Extra bolts (with [`ExtraBolt`]) are despawned permanently.
/// Sends a [`BoltLost`] message in both cases.
pub(crate) fn bolt_lost(
    mut commands: Commands,
    playfield: Res<PlayfieldConfig>,
    mut rng: ResMut<GameRng>,
    mut bolt_query: Query<LostBoltData, ActiveFilter>,
    mut breaker_query: Query<
        (Entity, &rantzsoft_spatial2d::components::Position2D),
        CollisionFilterBreaker,
    >,
    mut writers: BoltLostWriters,
    mut lost_bolts: Local<Vec<LostBoltEntry>>,
) {
    let Ok((_breaker_entity, breaker_position)) = breaker_query.single_mut() else {
        return;
    };
    let breaker_pos = breaker_position.0;

    // Collect lost bolts to avoid mutable borrow conflicts with despawn.
    // `Local<Vec>` reuses its heap allocation across frames — zero allocs after warmup.
    lost_bolts.clear();
    lost_bolts.extend(
        bolt_query
            .iter()
            .filter(|bolt| {
                let r = bolt.radius.0 * bolt.node_scale.map_or(1.0, |s| s.0);
                bolt.spatial.position.0.y < playfield.bottom() - r
            })
            .map(|bolt| LostBoltEntry {
                entity: bolt.entity,
                spawn_offset: bolt.spawn_offset.0,
                angle_spread: bolt
                    .angle_spread
                    .map_or(crate::bolt::resources::DEFAULT_BOLT_ANGLE_SPREAD, |a| a.0),
                is_extra: bolt.is_extra,
            }),
    );

    for entry in &*lost_bolts {
        writers.writer.write(BoltLost);
        if entry.is_extra {
            if let Ok(ref mut destroyed_writer) = writers.request_destroyed_writer {
                // Two-phase destruction: write request (entity stays alive for bridge evaluation)
                destroyed_writer.write(RequestBoltDestroyed { bolt: entry.entity });
            } else {
                // Legacy path: despawn directly when RequestBoltDestroyed is not registered
                commands.entity(entry.entity).despawn();
            }
        } else {
            // Respawn above breaker
            let angle = rng.0.random_range(-entry.angle_spread..=entry.angle_spread);
            // Angle from vertical: sin->X, cos->Y; positive Y is upward.
            // Set direction only; speed is applied by the velocity formula.
            if let Ok(mut bolt) = bolt_query.get_mut(entry.entity) {
                bolt.spatial.velocity.0 = Vec2::new(angle.sin(), angle.cos());
                apply_velocity_formula(&mut bolt.spatial, bolt.active_speed_boosts);
                let new_pos = Vec2::new(breaker_pos.x, breaker_pos.y + entry.spawn_offset);
                bolt.spatial.position.0 = new_pos;
                commands
                    .entity(entry.entity)
                    .insert(PreviousPosition(new_pos));
            }
        }
    }
}
