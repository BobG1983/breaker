//! Bolt-lost detection — bolt falls below the playfield.

use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::Rng;
use rantzsoft_spatial2d::components::PreviousPosition;

use crate::{
    bolt::{
        filters::ActiveFilter,
        messages::BoltLost,
        queries::{LostBoltData, apply_velocity_formula},
    },
    breaker::filters::CollisionFilterBreaker,
    prelude::*,
    shared::death_pipeline::kill_yourself::KillYourself,
};

/// Bundled message writers for `bolt_lost` to satisfy clippy's
/// `too_many_arguments` lint.
#[derive(SystemParam)]
pub(crate) struct BoltLostWriters<'w> {
    writer:        MessageWriter<'w, BoltLost>,
    kill_yourself: MessageWriter<'w, KillYourself<Bolt>>,
}

/// Collected data for a single lost bolt, used as scratch storage between
/// the filter pass and the command-application pass.
#[derive(Clone, Copy)]
pub(crate) struct LostBoltEntry {
    entity:       Entity,
    spawn_offset: f32,
    angle_spread: f32,
    is_extra:     bool,
    radius:       f32,
    node_scale:   f32,
    layers:       CollisionLayers,
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
    let Ok((breaker_entity, breaker_position)) = breaker_query.single_mut() else {
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
                entity:       bolt.entity,
                spawn_offset: bolt.spawn_offset.0,
                angle_spread: bolt
                    .angle_spread
                    .map_or(crate::bolt::resources::DEFAULT_BOLT_ANGLE_SPREAD, |a| a.0),
                is_extra:     bolt.is_extra,
                radius:       bolt.radius.0,
                node_scale:   bolt.node_scale.map_or(1.0, |s| s.0),
                layers:       *bolt.layers,
            }),
    );

    for entry in &*lost_bolts {
        writers.writer.write(BoltLost {
            bolt:    entry.entity,
            breaker: breaker_entity,
        });
        if entry.is_extra {
            // Unified death pipeline: write KillYourself<Bolt>. handle_kill<Bolt>
            // (registered by DeathPipelinePlugin) will mark Dead, emit
            // Destroyed<Bolt> (read by the effect_v3 death bridge), and enqueue
            // DespawnEntity for FixedPostUpdate.
            writers.kill_yourself.write(KillYourself::<Bolt> {
                victim:  entry.entity,
                killer:  None,
                _marker: PhantomData,
            });
        } else {
            // Respawn above breaker
            let angle = rng.0.random_range(-entry.angle_spread..=entry.angle_spread);
            // Angle from vertical: sin->X, cos->Y; positive Y is upward.
            // Set direction only; speed is applied by the velocity formula.
            if let Ok(mut bolt) = bolt_query.get_mut(entry.entity) {
                bolt.spatial.velocity.0 = Vec2::new(angle.sin(), angle.cos());
                apply_velocity_formula(
                    &mut bolt.spatial,
                    bolt.active_speed_boosts
                        .map_or(1.0, crate::effect_v3::stacking::EffectStack::aggregate),
                );
                let new_pos = Vec2::new(breaker_pos.x, breaker_pos.y + entry.spawn_offset);
                bolt.spatial.position.0 = new_pos;

                let effective_radius = entry.radius * entry.node_scale;
                let target_scale = Scale2D {
                    x: effective_radius,
                    y: effective_radius,
                };
                let stashed_layers = entry.layers;

                commands.entity(entry.entity).insert((
                    PreviousPosition(new_pos),
                    Scale2D { x: 0.0, y: 0.0 },
                    PreviousScale { x: 0.0, y: 0.0 },
                    CollisionLayers::default(),
                    Birthing::new(target_scale, stashed_layers),
                ));
            }
        }
    }
}
