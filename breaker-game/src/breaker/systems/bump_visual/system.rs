//! Bump visual feedback — eased upward pop animation on the breaker.

use bevy::{math::curve::Curve, prelude::*};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{components::*, filters::BumpTriggerFilter},
    input::resources::{GameAction, InputActions},
};

/// Triggers a bump pop animation when the player presses bump.
///
/// Reads [`InputActions`] directly so the visual fires on every press,
/// regardless of cooldown or active-window state. The [`Without<BumpFeedbackState>`]
/// filter in [`BumpTriggerFilter`] prevents duplicate triggers.
pub fn trigger_bump_visual(
    mut commands: Commands,
    actions: Res<InputActions>,
    query: Query<(Entity, &BumpFeedback), BumpTriggerFilter>,
) {
    if !actions.active(GameAction::Bump) {
        return;
    }
    for (entity, params) in &query {
        commands.entity(entity).insert(BumpFeedbackState {
            timer: params.duration,
            duration: params.duration,
            peak_offset: params.peak,
        });
    }
}

/// Animates the bump pop — fast rise, slower settle.
///
/// Uses a piecewise two-phase curve with configurable easing per phase.
/// Removes [`BumpFeedbackState`] when the animation completes.
pub fn animate_bump_visual(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Position2D,
            &mut BumpFeedbackState,
            &BreakerBaseY,
            &BumpFeedback,
        ),
        With<Breaker>,
    >,
) {
    let dt = time.delta_secs();

    for (entity, mut position, mut visual, base_y, params) in &mut query {
        // Remove previous frame's offset
        let prev_offset = bump_offset(&visual, params);
        position.0.y -= prev_offset;

        visual.timer -= dt;

        if visual.timer <= 0.0 {
            // Animation complete — snap to base position
            position.0.y = base_y.0;
            commands.entity(entity).remove::<BumpFeedbackState>();
        } else {
            // Apply new eased offset
            position.0.y += bump_offset(&visual, params);
        }
    }
}

/// Calculates the current Y offset for the bump animation.
///
/// Piecewise two-phase curve: rise phase (0->peak) uses `rise_ease`,
/// fall phase (peak->0) uses `fall_ease`. `peak_fraction`
/// controls what fraction of the total duration is spent rising.
pub(crate) fn bump_offset(visual: &BumpFeedbackState, params: &BumpFeedback) -> f32 {
    let peak_fraction = params.peak_fraction;
    // progress: 0.0 at start -> 1.0 at end
    let progress = 1.0 - (visual.timer / visual.duration).clamp(0.0, 1.0);

    if progress <= peak_fraction {
        // Rise phase: 0->peak_offset
        let rise_t = if peak_fraction > f32::EPSILON {
            progress / peak_fraction
        } else {
            1.0
        };
        params.rise_ease.sample_clamped(rise_t) * visual.peak_offset
    } else {
        // Fall phase: peak_offset->0
        let fall_t = if (1.0 - peak_fraction) > f32::EPSILON {
            (progress - peak_fraction) / (1.0 - peak_fraction)
        } else {
            1.0
        };
        (1.0 - params.fall_ease.sample_clamped(fall_t)) * visual.peak_offset
    }
}
