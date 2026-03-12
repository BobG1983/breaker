//! Bump visual feedback — eased upward pop animation on the breaker.

use bevy::{math::curve::Curve, prelude::*};

use crate::{
    breaker::{
        components::{Breaker, BumpVisual},
        resources::BreakerConfig,
    },
    input::resources::{GameAction, InputActions},
};

/// Query filter for breakers eligible for a new bump visual.
type BumpTriggerFilter = (With<Breaker>, Without<BumpVisual>);

/// Triggers a bump pop animation when the player presses bump.
///
/// Reads [`InputActions`] directly so the visual fires on every press,
/// regardless of cooldown or active-window state. The [`Without<BumpVisual>`]
/// filter in [`BumpTriggerFilter`] prevents duplicate triggers.
pub fn trigger_bump_visual(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    actions: Res<InputActions>,
    query: Query<Entity, BumpTriggerFilter>,
) {
    if !actions.active(GameAction::Bump) {
        return;
    }
    for entity in &query {
        commands.entity(entity).insert(BumpVisual {
            timer: config.bump_visual_duration,
            duration: config.bump_visual_duration,
            peak_offset: config.bump_visual_peak,
        });
    }
}

/// Animates the bump pop — fast rise, slower settle.
///
/// Uses a piecewise two-phase curve with configurable easing per phase.
/// Removes [`BumpVisual`] when the animation completes.
pub fn animate_bump_visual(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<BreakerConfig>,
    mut query: Query<(Entity, &mut Transform, &mut BumpVisual), With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut visual) in &mut query {
        // Remove previous frame's offset
        let prev_offset = bump_offset(&visual, &config);
        transform.translation.y -= prev_offset;

        visual.timer -= dt;

        if visual.timer <= 0.0 {
            // Animation complete — snap to base position
            transform.translation.y = config.y_position;
            commands.entity(entity).remove::<BumpVisual>();
        } else {
            // Apply new eased offset
            transform.translation.y += bump_offset(&visual, &config);
        }
    }
}

/// Calculates the current Y offset for the bump animation.
///
/// Piecewise two-phase curve: rise phase (0→peak) uses `bump_visual_rise_ease`,
/// fall phase (peak→0) uses `bump_visual_fall_ease`. `bump_visual_peak_fraction`
/// controls what fraction of the total duration is spent rising.
fn bump_offset(visual: &BumpVisual, config: &BreakerConfig) -> f32 {
    let peak_fraction = config.bump_visual_peak_fraction;
    // progress: 0.0 at start → 1.0 at end
    let progress = 1.0 - (visual.timer / visual.duration).clamp(0.0, 1.0);

    if progress <= peak_fraction {
        // Rise phase: 0→peak_offset
        let rise_t = if peak_fraction > f32::EPSILON {
            progress / peak_fraction
        } else {
            1.0
        };
        config.bump_visual_rise_ease.sample_clamped(rise_t) * visual.peak_offset
    } else {
        // Fall phase: peak_offset→0
        let fall_t = if (1.0 - peak_fraction) > f32::EPSILON {
            (progress - peak_fraction) / (1.0 - peak_fraction)
        } else {
            1.0
        };
        (1.0 - config.bump_visual_fall_ease.sample_clamped(fall_t)) * visual.peak_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bump_offset(timer_fraction: f32) -> f32 {
        let config = BreakerConfig::default();
        let visual = BumpVisual {
            timer: config.bump_visual_duration * timer_fraction,
            duration: config.bump_visual_duration,
            peak_offset: config.bump_visual_peak,
        };
        bump_offset(&visual, &config)
    }

    #[test]
    fn bump_offset_starts_at_zero() {
        assert!(test_bump_offset(1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn bump_offset_ends_at_zero() {
        assert!(test_bump_offset(0.0).abs() < 1e-5);
    }

    #[test]
    fn bump_offset_positive_mid_animation() {
        assert!(
            test_bump_offset(0.5) > 0.0,
            "offset should be positive during animation"
        );
    }

    #[test]
    fn bump_offset_at_peak_fraction_equals_peak() {
        let config = BreakerConfig::default();
        let timer = config.bump_visual_duration * (1.0 - config.bump_visual_peak_fraction);
        let visual = BumpVisual {
            timer,
            duration: config.bump_visual_duration,
            peak_offset: config.bump_visual_peak,
        };
        let offset = bump_offset(&visual, &config);
        assert!(
            (offset - config.bump_visual_peak).abs() < 0.01,
            "offset at peak_fraction should equal peak_offset, got {offset}"
        );
    }

    #[test]
    fn bump_offset_asymmetric_shape() {
        let config = BreakerConfig::default();
        let rise_mid = bump_offset(
            &BumpVisual {
                timer: config.bump_visual_duration * (1.0 - 0.15),
                duration: config.bump_visual_duration,
                peak_offset: config.bump_visual_peak,
            },
            &config,
        );

        let fall_mid = bump_offset(
            &BumpVisual {
                timer: config.bump_visual_duration * (1.0 - 0.65),
                duration: config.bump_visual_duration,
                peak_offset: config.bump_visual_peak,
            },
            &config,
        );

        // With CubicOut rise (fast start) and QuadraticIn fall (lingers near peak),
        // both should be well above 50% of peak at their respective midpoints.
        assert!(
            rise_mid > config.bump_visual_peak * 0.5,
            "CubicOut rise at 50% should be above 50% of peak, got {rise_mid}"
        );
        assert!(
            fall_mid > config.bump_visual_peak * 0.5,
            "QuadraticIn fall at 50% should still be above 50% of peak (lingering), got {fall_mid}"
        );
    }

    use crate::breaker::components::BumpState;

    fn trigger_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.init_resource::<InputActions>();
        app.add_systems(Update, trigger_bump_visual);
        app
    }

    fn set_bump_action(app: &mut App) {
        app.world_mut()
            .resource_mut::<InputActions>()
            .0
            .push(GameAction::Bump);
    }

    #[test]
    fn trigger_inserts_bump_visual_on_bump_action() {
        let mut app = trigger_test_app();

        let entity = app.world_mut().spawn((Breaker, BumpState::default())).id();

        set_bump_action(&mut app);
        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_some(),
            "BumpVisual should be inserted when Bump action is active"
        );
    }

    #[test]
    fn trigger_skips_without_bump_action() {
        let mut app = trigger_test_app();

        let entity = app.world_mut().spawn((Breaker, BumpState::default())).id();

        // No Bump action set
        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_none(),
            "BumpVisual should not be inserted without Bump action"
        );
    }

    #[test]
    fn trigger_fires_during_cooldown() {
        let mut app = trigger_test_app();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    cooldown: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        set_bump_action(&mut app);
        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_some(),
            "BumpVisual should fire even during cooldown"
        );
    }

    #[test]
    fn trigger_does_not_retrigger_while_animating() {
        let mut app = trigger_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState::default(),
                BumpVisual {
                    timer: 0.1,
                    duration: config.bump_visual_duration,
                    peak_offset: config.bump_visual_peak,
                },
            ))
            .id();

        set_bump_action(&mut app);
        app.update();

        let visual = app
            .world()
            .get::<BumpVisual>(entity)
            .expect("should still have BumpVisual");
        assert!(
            (visual.timer - 0.1).abs() < f32::EPSILON,
            "timer should be unchanged — trigger should not overwrite existing animation"
        );
    }

    fn animate_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        app.add_systems(Update, animate_bump_visual);
        // Prime wall-clock time so the next update has a non-zero delta
        app.update();
        app
    }

    #[test]
    fn animate_applies_y_offset_during_animation() {
        let mut app = animate_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        app.world_mut().spawn((
            Breaker,
            Transform::from_xyz(0.0, config.y_position, 0.0),
            BumpVisual {
                timer: config.bump_visual_duration,
                duration: config.bump_visual_duration,
                peak_offset: config.bump_visual_peak,
            },
        ));

        app.update();

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        assert!(
            tf.translation.y > config.y_position,
            "breaker should pop upward during animation, y={} base={}",
            tf.translation.y,
            config.y_position
        );
    }

    #[test]
    fn animate_removes_bump_visual_when_done() {
        let mut app = animate_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                Transform::from_xyz(0.0, config.y_position, 0.0),
                BumpVisual {
                    // Zero timer — will expire on next tick
                    timer: 0.0,
                    duration: config.bump_visual_duration,
                    peak_offset: config.bump_visual_peak,
                },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_none(),
            "BumpVisual should be removed after animation completes"
        );

        let tf = app.world().get::<Transform>(entity).expect("should exist");
        assert!(
            (tf.translation.y - config.y_position).abs() < f32::EPSILON,
            "breaker should return to base y after animation, got {}",
            tf.translation.y
        );
    }

    #[test]
    fn animate_snaps_to_base_after_expiry() {
        let mut app = animate_test_app();
        let config = app.world().resource::<BreakerConfig>().clone();

        // Start with an offset Y to verify the snap overrides it
        app.world_mut().spawn((
            Breaker,
            Transform::from_xyz(0.0, config.y_position + 5.0, 0.0),
            BumpVisual {
                // Near-expired timer — will complete within a few test updates
                timer: 0.0001,
                duration: config.bump_visual_duration,
                peak_offset: config.bump_visual_peak,
            },
        ));

        // A few ticks to let the timer expire and commands flush
        for _ in 0..5 {
            app.update();
        }

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        assert!(
            (tf.translation.y - config.y_position).abs() < f32::EPSILON,
            "breaker should snap to base y after animation, got {} base={}",
            tf.translation.y,
            config.y_position
        );
    }
}
