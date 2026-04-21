use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, quadtree::circle_overlaps_aabb};

use super::{components::PhantomBreaker, config::AfterimageConfig};
use crate::{
    bolt::{components::BoltRadius, filters::ActiveFilter},
    breaker::{
        components::{BaseHeight, BaseWidth, BumpLateWindow, BumpPerfectWindow, BumpState},
        messages::BumpGrade,
        systems::{forward_grade, retroactive_grade},
    },
    effect_v3::effects::phantom_bolt::components::PhantomBolt,
    prelude::*,
};

// ── System 3 — afterimage_check_phantom_bounce ──────────────────────────────

type PhantomBounceBreakerQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static BumpState,
        &'static BumpPerfectWindow,
        &'static BumpLateWindow,
        Option<&'static AnchorPlanted>,
        Option<&'static AnchorActive>,
    ),
    With<Breaker>,
>;

type PhantomBounceBoltQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Position2D,
        &'static mut Velocity2D,
        &'static BoltRadius,
    ),
    (ActiveFilter, Without<PhantomBreaker>, Without<PhantomBolt>),
>;

/// Detects bolt-vs-`PhantomBreaker` AABB overlap, reflects the bolt's
/// vertical velocity, snaps the bolt above the phantom's top face, and
/// emits a synthetic `BumpPerformed` whose grade is computed from the real
/// breaker's `BumpState`.
///
/// The emitted `BumpPerformed` is broadcast to every consumer (audio, UI,
/// Overclock, other protocols) by design — phantom bumps are real bumps
/// from downstream consumers' perspective.
///
/// Entry guard: a bolt with `velocity.y >= 0.0` (already moving away from
/// the bump surface, or moving purely horizontally along it) is NOT
/// reflected and does NOT emit `BumpPerformed`.
///
/// Dedup: `bounced_this_frame` ensures a bolt overlapping multiple
/// phantoms reflects at most once per tick.
///
/// Does NOT stamp `LastImpact`, does NOT reset `PiercingRemaining`, does
/// NOT mutate the real breaker's `BumpState`.
///
/// Harness-safe: early-returns when `AfterimageConfig` is absent or no
/// `PhantomBreaker` entities exist. Aborts processing when the real
/// breaker query cannot resolve to exactly one entity.
pub(crate) fn afterimage_check_phantom_bounce(
    config: Option<Res<AfterimageConfig>>,
    phantoms: Query<(Entity, &Position2D, &BaseWidth, &BaseHeight), With<PhantomBreaker>>,
    mut bolts: PhantomBounceBoltQuery,
    real_breaker: PhantomBounceBreakerQuery,
    mut bump_writer: MessageWriter<BumpPerformed>,
    mut bounced_this_frame: Local<Vec<Entity>>,
) {
    bounced_this_frame.clear();
    let Some(_config) = config else {
        return;
    };
    if phantoms.is_empty() {
        return;
    }
    let Ok((_real_entity, bump_state, perfect_window, late_window, anchor_planted, anchor_active)) =
        real_breaker.single()
    else {
        return;
    };
    let effective_pw = match (anchor_planted, anchor_active) {
        (Some(_), Some(a)) => perfect_window.0 * a.perfect_window_multiplier,
        _ => perfect_window.0,
    };

    for (bolt_entity, mut bolt_pos, mut bolt_velocity, bolt_radius) in &mut bolts {
        if bounced_this_frame.contains(&bolt_entity) {
            continue;
        }
        for (phantom_entity, phantom_pos, phantom_width, phantom_height) in &phantoms {
            let phantom_half = Vec2::new(phantom_width.half_width(), phantom_height.half_height());
            let phantom_aabb = Aabb2D::new(phantom_pos.0, phantom_half);
            if !circle_overlaps_aabb(bolt_pos.0, bolt_radius.0, &phantom_aabb) {
                continue;
            }
            if bolt_velocity.0.y >= 0.0 {
                // Bolt already moving upward — past the bump surface. Skip.
                continue;
            }
            // Vertical mirror reflection — phantom has no tilt, no spread.
            bolt_velocity.0.y = -bolt_velocity.0.y;
            bolt_pos.0.y = phantom_pos.0.y + phantom_half.y + bolt_radius.0;

            let grade = if bump_state.active && bump_state.timer > 0.0 {
                forward_grade(bump_state.timer, effective_pw)
            } else if bump_state.post_hit_timer > 0.0 {
                // Mirrors `update_bump`'s retroactive formula:
                //   time_since_hit = (effective_pw + late_window) - post_hit_timer
                let time_since_hit = (effective_pw + late_window.0) - bump_state.post_hit_timer;
                retroactive_grade(time_since_hit, effective_pw)
            } else {
                // Neither window open — player is not engaged. Does NOT promote.
                BumpGrade::Late
            };

            bump_writer.write(BumpPerformed {
                grade,
                bolt: Some(bolt_entity),
                breaker: phantom_entity,
            });
            bounced_this_frame.push(bolt_entity);
            break;
        }
    }
}
