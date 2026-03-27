//! Speed boost effect handler — scales bolt velocity on trigger.
//!
//! Observes [`SpeedBoostFired`] and pushes a multiplier onto the bolt's
//! [`ActiveSpeedBoosts`] vec. The [`apply_speed_boosts`] system recalculates
//! velocity from base speed * product(boosts), clamped to max.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
    chips::components::BoltSpeedBoost,
    effect::definition::EffectTarget,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a speed boost effect resolves via a triggered chain.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpeedBoostFired {
    /// Multiplier applied to the current velocity magnitude.
    pub multiplier: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    // FUTURE: may be used for upcoming phases
    // /// The originating chip name, or `None` for breaker chains.
    // pub source_chip: Option<String>,
}

/// Query for bolts needing speed boost handling (velocity, base/max speed, optional boost amp,
/// optional active boost tracking).
type SpeedBoostQuery = (
    &'static mut Velocity2D,
    &'static BoltBaseSpeed,
    &'static BoltMaxSpeed,
    Option<&'static BoltSpeedBoost>,
    Option<&'static mut ActiveSpeedBoosts>,
);

/// Per-bolt tracking of active speed boost multipliers.
///
/// Each entry is a multiplier (e.g. 1.5 for 50% speed increase). The
/// [`apply_speed_boosts`] system recalculates velocity as
/// `base_speed * product(boosts)`, clamped to `[base_speed, max_speed]`.
/// Until reversal removes entries from the vec.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub(crate) struct ActiveSpeedBoosts(pub Vec<f32>);

/// Recalculates bolt velocity from `BoltBaseSpeed` * product(`ActiveSpeedBoosts`),
/// clamped within [`BoltBaseSpeed`, `BoltMaxSpeed`], preserving direction.
///
/// Skips bolts with zero velocity (cannot determine direction).
pub(crate) fn apply_speed_boosts(
    mut query: Query<
        (
            &mut Velocity2D,
            &BoltBaseSpeed,
            &BoltMaxSpeed,
            &ActiveSpeedBoosts,
        ),
        With<Bolt>,
    >,
) {
    for (mut vel, base_speed, max_speed, active_boosts) in &mut query {
        let current_speed = vel.speed();
        if current_speed < f32::EPSILON {
            continue;
        }

        let direction = vel.0.normalize();
        let product: f32 = active_boosts.0.iter().product();
        let target_speed = (base_speed.0 * product).clamp(base_speed.0, max_speed.0);
        vel.0 = direction * target_speed;
    }
}

/// Observer: handles speed boost when a `SpeedBoostFired` event fires.
///
/// If `targets` contains entity references, applies to those specific bolts.
/// If `targets` is empty, applies to all bolts (`AllBolts` behavior).
/// Clamps within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`.
/// Also pushes the multiplier onto each bolt's [`ActiveSpeedBoosts`] vec
/// (if present) so that Until reversal can remove individual entries.
pub(crate) fn handle_speed_boost(
    trigger: On<SpeedBoostFired>,
    mut bolt_query: Query<SpeedBoostQuery, With<Bolt>>,
) {
    let event = trigger.event();

    let bolt_entities: Vec<_> = event
        .targets
        .iter()
        .filter_map(|t| match t {
            crate::effect::definition::EffectTarget::Entity(e) => Some(*e),
            crate::effect::definition::EffectTarget::Location(_) => None,
        })
        .collect();

    if bolt_entities.is_empty() {
        // No specific targets — apply to all bolts
        for (mut vel, base_speed, max_speed, speed_boost, mut active_boosts) in &mut bolt_query {
            let boost = speed_boost.map_or(0.0, |b| b.0);
            apply_speed_scale(&mut vel, event.multiplier, base_speed.0, max_speed.0, boost);
            if let Some(ref mut boosts) = active_boosts {
                boosts.0.push(event.multiplier);
            }
        }
    } else {
        // Apply to specific bolt entities
        for bolt_entity in bolt_entities {
            let Ok((mut vel, base_speed, max_speed, speed_boost, mut active_boosts)) =
                bolt_query.get_mut(bolt_entity)
            else {
                continue;
            };

            let boost = speed_boost.map_or(0.0, |b| b.0);
            apply_speed_scale(&mut vel, event.multiplier, base_speed.0, max_speed.0, boost);
            if let Some(ref mut boosts) = active_boosts {
                boosts.0.push(event.multiplier);
            }
        }
    }
}

/// Scales a bolt's velocity by `multiplier` and clamps the resulting speed
/// within `[base + boost, max + boost]`. Zero velocity remains zero.
fn apply_speed_scale(vel: &mut Velocity2D, multiplier: f32, base: f32, max: f32, boost: f32) {
    let current = vel.speed();
    if current < f32::EPSILON {
        return;
    }

    vel.0 *= multiplier;

    // Floor at effective base speed (base + boost)
    let speed = vel.speed();
    if speed > 0.0 && speed < base + boost {
        vel.0 = vel.0.normalize_or_zero() * (base + boost);
    }

    // Clamp to effective max speed (max + boost)
    let speed = vel.speed();
    if speed > max + boost {
        vel.0 = vel.0.normalize_or_zero() * (max + boost);
    }
}

/// Registers all observers and systems for the speed boost effect.
pub(crate) fn register(app: &mut App) {
    use crate::{
        effect::{effect_nodes::until, sets::EffectSystems},
        shared::PlayingState,
    };

    app.add_observer(handle_speed_boost);

    // Speed boost recalculation — after bridge and Until reversal
    app.add_systems(
        FixedUpdate,
        apply_speed_boosts
            .after(EffectSystems::Bridge)
            .after(until::tick_until_timers)
            .after(until::check_until_triggers)
            .run_if(in_state(PlayingState::Active)),
    );
}
