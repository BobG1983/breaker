//! Bump system — input, timing grades, velocity modifiers.

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltServing, messages::BoltImpactBreaker},
    breaker::{
        components::{Breaker, DashState, DashStateTimer, SettleDuration},
        messages::{BumpGrade, BumpPerformed, BumpWhiffed},
        queries::{BreakerBumpGradingData, BreakerBumpTimingData},
        resources::ForceBumpGrade,
    },
    effect::{AnchorActive, AnchorPlanted},
    input::resources::{GameAction, InputActions},
};

/// Determines the forward-window grade based on remaining timer.
///
/// Called when the bolt hits while a forward bump is active.
/// Timer counts down from `early_window + perfect_window`.
pub(super) fn forward_grade(timer: f32, perfect_window: f32) -> BumpGrade {
    if timer <= perfect_window {
        BumpGrade::Perfect
    } else {
        BumpGrade::Early
    }
}

/// Determines the retroactive grade based on time elapsed since hit.
///
/// Called when the player presses bump after the bolt has already hit.
pub(super) fn retroactive_grade(time_since_hit: f32, perfect_window: f32) -> BumpGrade {
    if time_since_hit <= perfect_window {
        BumpGrade::Perfect
    } else {
        BumpGrade::Late
    }
}

/// Computes the effective perfect window, widened by the anchor multiplier when planted.
fn effective_perfect_window(
    base: f32,
    planted: Option<&AnchorPlanted>,
    active: Option<&AnchorActive>,
) -> f32 {
    match (planted, active) {
        (Some(_), Some(a)) => base * a.perfect_window_multiplier,
        _ => base,
    }
}

/// Returns the grade-dependent cooldown duration.
const fn cooldown_for_grade(grade: BumpGrade, perfect_cooldown: f32, weak_cooldown: f32) -> f32 {
    match grade {
        BumpGrade::Perfect => perfect_cooldown,
        BumpGrade::Early | BumpGrade::Late => weak_cooldown,
    }
}

/// Updates bump state: handles input, ticks timers, resolves retroactive bumps.
///
/// Ticks the forward window timer but does not expire it — [`grade_bump`]
/// handles expiry after processing any same-frame hits.
/// Retroactive bumps grade and write immediately on press.
pub(crate) fn update_bump(
    actions: Res<InputActions>,
    time: Res<Time<Fixed>>,
    mut query: Query<(Entity, BreakerBumpTimingData), With<Breaker>>,
    mut writer: MessageWriter<BumpPerformed>,
    serving_query: Query<(), With<BoltServing>>,
) {
    let bolt_serving = !serving_query.is_empty();
    let dt = time.delta_secs();

    for (breaker_entity, mut data) in &mut query {
        let effective_pw = effective_perfect_window(
            data.perfect_window.0,
            data.anchor_planted,
            data.anchor_active,
        );

        // Tick cooldown
        if data.bump.cooldown > 0.0 {
            data.bump.cooldown = (data.bump.cooldown - dt).max(0.0);
        }

        // Tick post-hit timer
        if data.bump.post_hit_timer > 0.0 {
            data.bump.post_hit_timer = (data.bump.post_hit_timer - dt).max(0.0);
        }

        // Tick active timer — grade_bump handles expiry
        if data.bump.active {
            data.bump.timer -= dt;
        }

        // Bump input — skip when bolt is still serving (launch_bolt handles that press)
        if actions.active(GameAction::Bump) && data.bump.cooldown <= 0.0 && !bolt_serving {
            if data.bump.post_hit_timer > 0.0 {
                // Retroactive path: bolt already hit, player pressing after
                let time_since_hit = (effective_pw + data.late_window.0) - data.bump.post_hit_timer;
                let grade = retroactive_grade(time_since_hit, effective_pw);
                writer.write(BumpPerformed {
                    grade,
                    bolt: data.bump.last_hit_bolt,
                    breaker: breaker_entity,
                });
                data.bump.cooldown =
                    cooldown_for_grade(grade, data.perfect_cooldown.0, data.weak_cooldown.0);
                data.bump.post_hit_timer = 0.0;
                data.bump.last_hit_bolt = None;
                data.bump.active = false;
            } else if !data.bump.active {
                // Forward path: no recent hit, open the window
                data.bump.active = true;
                data.bump.timer = data.early_window.0 + effective_pw;
            }
        }
    }
}

/// Grades bump timing on bolt-breaker contact and sends [`BumpPerformed`].
///
/// Must run after `BoltSystems::BreakerCollision` to ensure messages are available.
/// If a forward bump is active, grades immediately. Otherwise, sets `post_hit_timer`
/// for the retroactive path in `update_bump`.
///
/// Also expires the forward window when the timer runs out without a hit,
/// sending [`BumpWhiffed`] and setting whiff cooldown.
pub(crate) fn grade_bump(
    mut bump_query: Query<(Entity, BreakerBumpGradingData), With<Breaker>>,
    mut hit_reader: MessageReader<BoltImpactBreaker>,
    mut writer: MessageWriter<BumpPerformed>,
    mut whiff_writer: MessageWriter<BumpWhiffed>,
    force_grade: Option<Res<ForceBumpGrade>>,
) {
    let forced = force_grade.as_ref().and_then(|fg| fg.0);
    let Ok((breaker_entity, mut data)) = bump_query.single_mut() else {
        return;
    };

    let effective_pw = effective_perfect_window(
        data.perfect_window.0,
        data.anchor_planted,
        data.anchor_active,
    );

    for hit in hit_reader.read() {
        if data.bump.active {
            // Forward path: grade based on timer position, with optional override
            let natural_grade = forward_grade(data.bump.timer, effective_pw);
            let grade = forced.unwrap_or(natural_grade);
            writer.write(BumpPerformed {
                grade,
                bolt: Some(hit.bolt),
                breaker: breaker_entity,
            });
            data.bump.active = false;
            data.bump.cooldown =
                cooldown_for_grade(grade, data.perfect_cooldown.0, data.weak_cooldown.0);
        } else {
            // No active bump — open retroactive window for update_bump
            data.bump.post_hit_timer = effective_pw + data.late_window.0;
            data.bump.last_hit_bolt = Some(hit.bolt);
        }
    }

    // Forward window expired without a hit — whiff
    if data.bump.active && data.bump.timer <= 0.0 {
        data.bump.active = false;
        data.bump.timer = 0.0;
        whiff_writer.write(BumpWhiffed);
        data.bump.cooldown = data.weak_cooldown.0;
    }
}

/// Cancels an ongoing dash when a perfect bump is performed.
///
/// Consumes [`BumpPerformed`] messages. When the grade is [`BumpGrade::Perfect`]
/// and the breaker is dashing, transitions directly to Settling.
pub fn perfect_bump_dash_cancel(
    mut reader: MessageReader<BumpPerformed>,
    mut query: Query<(&mut DashState, &mut DashStateTimer, &SettleDuration), With<Breaker>>,
) {
    for performed in reader.read() {
        if performed.grade != BumpGrade::Perfect {
            continue;
        }

        for (mut state, mut timer, settle_duration) in &mut query {
            if *state == DashState::Dashing {
                *state = DashState::Settling;
                timer.remaining = settle_duration.0;
            }
        }
    }
}
