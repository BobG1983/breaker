//! Bolt-lost detection — bolt falls below the playfield.

use bevy::{
    ecs::system::{SystemParam, SystemParamValidationError},
    prelude::*,
};
use rand::Rng;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, Velocity2D};

use crate::{
    bolt::{
        filters::ActiveFilter,
        messages::{BoltLost, RequestBoltDestroyed},
        queries::LostQuery,
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
    base_speed: f32,
    respawn_offset: f32,
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
    bolt_query: Query<LostQuery, ActiveFilter>,
    breaker_query: Query<&Position2D, CollisionFilterBreaker>,
    mut writers: BoltLostWriters,
    mut lost_bolts: Local<Vec<LostBoltEntry>>,
) {
    let Ok(breaker_position) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_position.0;

    // Collect lost bolts to avoid mutable borrow conflicts with despawn.
    // `Local<Vec>` reuses its heap allocation across frames — zero allocs after warmup.
    lost_bolts.clear();
    lost_bolts.extend(
        bolt_query
            .iter()
            .filter(|(_, pos, _, _, radius, _, _, _, entity_scale)| {
                let r = radius.0 * entity_scale.map_or(1.0, |s| s.0);
                pos.0.y < playfield.bottom() - r
            })
            .map(
                |(
                    entity,
                    _,
                    _,
                    base_speed,
                    _,
                    respawn_offset,
                    angle_spread,
                    is_extra,
                    _entity_scale,
                )| {
                    LostBoltEntry {
                        entity,
                        base_speed: base_speed.0,
                        respawn_offset: respawn_offset.0,
                        angle_spread: angle_spread.0,
                        is_extra,
                    }
                },
            ),
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
            let new_velocity = Vec2::new(
                entry.base_speed * angle.sin(),
                entry.base_speed * angle.cos(),
            );
            let new_pos = Vec2::new(breaker_pos.x, breaker_pos.y + entry.respawn_offset);
            commands.entity(entry.entity).insert((
                Position2D(new_pos),
                PreviousPosition(new_pos),
                Velocity2D(new_velocity),
            ));
        }
    }
}
