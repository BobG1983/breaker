//! Dash, brake, and settle state machine systems.

use bevy::{
    math::curve::{Curve, easing::EaseFunction},
    prelude::*,
};
use rantzsoft_spatial2d::components::{MaxSpeed, Position2D, Velocity2D};

use crate::{
    breaker::{
        components::{
            BaseWidth, BrakeDecel, BrakeTilt, Breaker, BreakerDeceleration, BreakerTilt,
            DashDuration, DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase,
            DecelEasing, SettleDuration, SettleTiltEase,
        },
        queries::BreakerDashData,
    },
    effect::effects::{
        flash_step::FlashStepActive, size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts,
    },
    input::resources::{GameAction, InputActions},
    shared::PlayfieldConfig,
};

/// Read-only dash configuration components, bundled to reduce argument count.
struct DashParams<'a> {
    max_speed: &'a MaxSpeed,
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

/// Idle/Settling context — input, timing, and optional `FlashStep` teleport data.
/// Bundled to keep `handle_idle_or_settling` under the argument-count lint threshold.
struct SettleContext<'a> {
    actions: &'a InputActions,
    dt: f32,
    flash_step: Option<&'a FlashStepActive>,
    position: Option<&'a mut Position2D>,
    breaker_width: Option<&'a BaseWidth>,
    playfield: &'a PlayfieldConfig,
    speed_mult: Option<&'a ActiveSpeedBoosts>,
    size_mult: Option<&'a ActiveSizeBoosts>,
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
    playfield: Res<PlayfieldConfig>,
    mut query: Query<BreakerDashData, With<Breaker>>,
) {
    let dt = time.delta_secs();

    for mut data in &mut query {
        let params = DashParams {
            max_speed: data.max_speed,
            decel: data.deceleration,
            easing: data.decel_easing,
            dash_speed: data.dash_speed,
            dash_duration: data.dash_duration,
            dash_tilt: data.dash_tilt,
            dash_tilt_ease: data.dash_tilt_ease,
            brake_tilt: data.brake_tilt,
            brake_decel: data.brake_decel,
            settle_duration: data.settle_duration,
            settle_tilt_ease: data.settle_tilt_ease,
        };

        let ctx = SettleContext {
            actions: &actions,
            dt,
            flash_step: data.flash_step,
            position: data.position.as_deref_mut(),
            breaker_width: data.base_width,
            playfield: &playfield,
            speed_mult: data.speed_boosts,
            size_mult: data.size_boosts,
        };

        match *data.state {
            DashState::Idle | DashState::Settling => {
                handle_idle_or_settling(
                    &mut data.state,
                    &mut data.velocity,
                    &mut data.tilt,
                    &mut data.timer,
                    &params,
                    ctx,
                );
            }
            DashState::Dashing => {
                handle_dashing(
                    dt,
                    &mut data.state,
                    *data.velocity,
                    &mut data.tilt,
                    &mut data.timer,
                    &params,
                );
            }
            DashState::Braking => {
                handle_braking(
                    dt,
                    &mut data.state,
                    &mut data.velocity,
                    &mut data.tilt,
                    &mut data.timer,
                    &params,
                );
            }
        }
    }
}

/// In Idle or Settling: check for dash actions from the input domain.
///
/// During Settling with `FlashStepActive`, a reversal dash (new direction sign
/// equals `tilt.ease_start` sign) triggers a teleport instead of a normal dash.
fn handle_idle_or_settling(
    state: &mut DashState,
    velocity: &mut Velocity2D,
    tilt: &mut BreakerTilt,
    timer: &mut DashStateTimer,
    p: &DashParams,
    mut ctx: SettleContext,
) {
    let is_settling = *state == DashState::Settling;
    let ease_start_before_tick = tilt.ease_start;

    if is_settling {
        // Tick settle timer, return tilt toward zero with easing
        timer.remaining -= ctx.dt;
        let settle_progress = if p.settle_duration.0 > f32::EPSILON {
            1.0 - (timer.remaining / p.settle_duration.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let eased = p.settle_tilt_ease.0.sample_clamped(settle_progress);
        tilt.angle = (tilt.ease_target - tilt.ease_start).mul_add(eased, tilt.ease_start);

        if timer.remaining <= 0.0 {
            *state = DashState::Idle;
            tilt.angle = 0.0;
        }
    }

    // Determine dash direction from input
    let direction = if ctx.actions.active(GameAction::DashLeft) {
        Some(-1.0_f32)
    } else if ctx.actions.active(GameAction::DashRight) {
        Some(1.0_f32)
    } else {
        None
    };

    let Some(direction) = direction else {
        return;
    };

    // Check for FlashStep teleport: only during Settling with FlashStepActive,
    // and only when the dash direction is a reversal (direction sign == ease_start sign,
    // with ease_start != 0.0). Using product > 0 avoids exact float comparison.
    if is_settling
        && ctx.flash_step.is_some()
        && direction * ease_start_before_tick > 0.0
        && let Some(pos) = ctx.position.as_deref_mut()
    {
        let effective_max_speed =
            p.max_speed.0 * ctx.speed_mult.map_or(1.0, ActiveSpeedBoosts::multiplier);
        let teleport_distance =
            direction * effective_max_speed * p.dash_speed.0 * p.dash_duration.0;
        pos.0.x += teleport_distance;

        // Clamp to playfield bounds accounting for effective half-width
        let effective_half_width = ctx.breaker_width.map_or(0.0, BaseWidth::half_width)
            * ctx.size_mult.map_or(1.0, ActiveSizeBoosts::multiplier);
        let min_x = ctx.playfield.left() + effective_half_width;
        let max_x = ctx.playfield.right() - effective_half_width;
        pos.0.x = pos.0.x.clamp(min_x, max_x);

        // Reset to clean Idle state
        velocity.0.x = 0.0;
        tilt.angle = 0.0;
        tilt.ease_start = 0.0;
        tilt.ease_target = 0.0;
        timer.remaining = 0.0;
        *state = DashState::Idle;
        return;
    }

    // Normal dash
    start_dash(direction, state, velocity, tilt, timer, p);
}

/// Enters the Dashing state in the given direction.
fn start_dash(
    direction: f32,
    state: &mut DashState,
    velocity: &mut Velocity2D,
    tilt: &mut BreakerTilt,
    timer: &mut DashStateTimer,
    p: &DashParams,
) {
    *state = DashState::Dashing;
    velocity.0.x = direction * p.max_speed.0 * p.dash_speed.0;
    // Tilt starts at zero — handle_dashing eases it to full value
    tilt.angle = 0.0;
    timer.remaining = p.dash_duration.0;
}

/// Dashing: count down timer, ease tilt to full angle, then transition to Braking.
fn handle_dashing(
    dt: f32,
    state: &mut DashState,
    velocity: Velocity2D,
    tilt: &mut BreakerTilt,
    timer: &mut DashStateTimer,
    p: &DashParams,
) {
    timer.remaining -= dt;

    let dash_dir = velocity.0.x.signum();

    if timer.remaining <= 0.0 {
        *state = DashState::Braking;
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
    state: &mut DashState,
    velocity: &mut Velocity2D,
    tilt: &mut BreakerTilt,
    timer: &mut DashStateTimer,
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
        velocity.0.x.abs(),
        reference_speed,
        p.easing.ease,
        p.easing.strength,
    );

    if velocity.0.x > f32::EPSILON {
        velocity.0.x = effective_decel.mul_add(-dt, velocity.0.x).max(0.0);
    } else if velocity.0.x < -f32::EPSILON {
        velocity.0.x = effective_decel.mul_add(dt, velocity.0.x).min(0.0);
    }

    // Speed near zero → transition to Settling
    if velocity.0.x.abs() <= f32::EPSILON {
        velocity.0.x = 0.0;
        tilt.ease_start = tilt.angle;
        tilt.ease_target = 0.0;
        timer.remaining = p.settle_duration.0;
        *state = DashState::Settling;
    }
}

/// Computes effective deceleration scaled by an easing curve over speed ratio.
///
/// `effective_decel = base_decel * (1.0 + strength * ease(speed / reference_speed))`
pub(crate) fn eased_decel(
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
