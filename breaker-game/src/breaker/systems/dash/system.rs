//! Dash, brake, and settle state machine systems.

use bevy::{
    math::curve::{Curve, easing::EaseFunction},
    prelude::*,
};

use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerMaxSpeed, BreakerState,
            BreakerStateTimer, BreakerTilt, BreakerVelocity, DashDuration, DashSpeedMultiplier,
            DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
        },
        queries::DashQuery,
    },
    input::resources::{GameAction, InputActions},
};

/// Read-only dash configuration components, bundled to reduce argument count.
struct DashParams<'a> {
    max_speed: &'a BreakerMaxSpeed,
    decel: &'a BreakerDeceleration,
    easing: &'a DecelEasing,
    dash_speed: &'a DashSpeedMultiplier,
    dash_duration: &'a DashDuration,
    dash_tilt: &'a DashTilt,
    dash_tilt_ease: &'a DashTiltEase,
    brake_tilt: &'a BrakeTilt,
    brake_decel: &'a BrakeDecel,
    settle_duration: &'a SettleDuration,
    settle_tilt_ease: &'a SettleTiltEase,
}

/// Handles dash input and the Dashing → Braking → Settling → Idle state machine.
///
/// - Dash input (`DashLeft`/`DashRight` from input domain): triggers dash from Idle or Settling
/// - Dashing: high-speed burst with tilt, transitions to Braking when timer expires
/// - Braking: rapid deceleration with opposite tilt, transitions to Settling when speed ~0
/// - Settling: tilt returns to neutral, transitions to Idle when timer expires
pub fn update_breaker_state(
    actions: Res<InputActions>,
    time: Res<Time<Fixed>>,
    mut query: Query<DashQuery, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (
        mut state,
        mut velocity,
        mut tilt,
        mut timer,
        max_speed,
        decel,
        easing,
        dash_speed,
        dash_duration,
        dash_tilt,
        dash_tilt_ease,
        brake_tilt,
        brake_decel,
        settle_duration,
        settle_tilt_ease,
    ) in &mut query
    {
        let params = DashParams {
            max_speed,
            decel,
            easing,
            dash_speed,
            dash_duration,
            dash_tilt,
            dash_tilt_ease,
            brake_tilt,
            brake_decel,
            settle_duration,
            settle_tilt_ease,
        };

        match *state {
            BreakerState::Idle | BreakerState::Settling => {
                handle_idle_or_settling(
                    &actions,
                    dt,
                    &mut state,
                    &mut velocity,
                    &mut tilt,
                    &mut timer,
                    &params,
                );
            }
            BreakerState::Dashing => {
                handle_dashing(dt, &mut state, &velocity, &mut tilt, &mut timer, &params);
            }
            BreakerState::Braking => {
                handle_braking(
                    dt,
                    &mut state,
                    &mut velocity,
                    &mut tilt,
                    &mut timer,
                    &params,
                );
            }
        }
    }
}

/// In Idle or Settling: check for dash actions from the input domain.
fn handle_idle_or_settling(
    actions: &InputActions,
    dt: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    p: &DashParams,
) {
    if *state == BreakerState::Settling {
        // Tick settle timer, return tilt toward zero with easing
        timer.remaining -= dt;
        let settle_progress = if p.settle_duration.0 > f32::EPSILON {
            1.0 - (timer.remaining / p.settle_duration.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let eased = p.settle_tilt_ease.0.sample_clamped(settle_progress);
        tilt.angle = (tilt.ease_target - tilt.ease_start).mul_add(eased, tilt.ease_start);

        if timer.remaining <= 0.0 {
            *state = BreakerState::Idle;
            tilt.angle = 0.0;
        }
    }

    // Dash left
    if actions.active(GameAction::DashLeft) {
        start_dash(-1.0, state, velocity, tilt, timer, p);
        return;
    }

    // Dash right
    if actions.active(GameAction::DashRight) {
        start_dash(1.0, state, velocity, tilt, timer, p);
    }
}

/// Enters the Dashing state in the given direction.
fn start_dash(
    direction: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    p: &DashParams,
) {
    *state = BreakerState::Dashing;
    velocity.x = direction * p.max_speed.0 * p.dash_speed.0;
    // Tilt starts at zero — handle_dashing eases it to full value
    tilt.angle = 0.0;
    timer.remaining = p.dash_duration.0;
}

/// Dashing: count down timer, ease tilt to full angle, then transition to Braking.
fn handle_dashing(
    dt: f32,
    state: &mut BreakerState,
    velocity: &BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    p: &DashParams,
) {
    timer.remaining -= dt;

    let dash_dir = velocity.x.signum();

    if timer.remaining <= 0.0 {
        *state = BreakerState::Braking;
        tilt.ease_start = tilt.angle;
        tilt.ease_target = -dash_dir * p.brake_tilt.angle;
        timer.remaining = p.brake_tilt.duration;
    } else {
        // Ease tilt from 0 to full angle over dash duration
        let progress = if p.dash_duration.0 > f32::EPSILON {
            1.0 - (timer.remaining / p.dash_duration.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let eased = p.dash_tilt_ease.0.sample_clamped(progress);
        tilt.angle = dash_dir * p.dash_tilt.0 * eased;
    }
}

/// Braking: ease tilt toward brake angle, decelerate, then transition to Settling.
fn handle_braking(
    dt: f32,
    state: &mut BreakerState,
    velocity: &mut BreakerVelocity,
    tilt: &mut BreakerTilt,
    timer: &mut BreakerStateTimer,
    p: &DashParams,
) {
    // Ease tilt toward brake angle
    if timer.remaining > 0.0 {
        timer.remaining -= dt;
        let progress = if p.brake_tilt.duration > f32::EPSILON {
            1.0 - (timer.remaining / p.brake_tilt.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let eased = p.brake_tilt.ease.sample_clamped(progress);
        tilt.angle = (tilt.ease_target - tilt.ease_start).mul_add(eased, tilt.ease_start);
    }

    // Decelerate
    let base_decel = p.decel.0 * p.brake_decel.0;
    let reference_speed = p.max_speed.0 * p.dash_speed.0;
    let effective_decel = eased_decel(
        base_decel,
        velocity.x.abs(),
        reference_speed,
        p.easing.ease,
        p.easing.strength,
    );

    if velocity.x > f32::EPSILON {
        velocity.x = effective_decel.mul_add(-dt, velocity.x).max(0.0);
    } else if velocity.x < -f32::EPSILON {
        velocity.x = effective_decel.mul_add(dt, velocity.x).min(0.0);
    }

    // Speed near zero → transition to Settling
    if velocity.x.abs() <= f32::EPSILON {
        velocity.x = 0.0;
        tilt.ease_start = tilt.angle;
        tilt.ease_target = 0.0;
        timer.remaining = p.settle_duration.0;
        *state = BreakerState::Settling;
    }
}

/// Computes effective deceleration scaled by an easing curve over speed ratio.
///
/// `effective_decel = base_decel * (1.0 + strength * ease(speed / reference_speed))`
pub fn eased_decel(
    base_decel: f32,
    speed: f32,
    reference_speed: f32,
    ease: EaseFunction,
    strength: f32,
) -> f32 {
    let speed_ratio = if reference_speed > f32::EPSILON {
        (speed / reference_speed).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let factor = ease.sample_clamped(speed_ratio);
    base_decel * strength.mul_add(factor, 1.0)
}
