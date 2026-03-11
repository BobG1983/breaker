//! Bump visual feedback — eased upward pop animation on the breaker.

use bevy::math::curve::Curve;
use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;

use crate::breaker::components::{Breaker, BumpState, BumpVisual};
use crate::breaker::resources::BreakerConfig;

/// Easing function for the bump pop animation.
const BUMP_EASE: EaseFunction = EaseFunction::QuadraticOut;

/// Query filter for bump entities needing visual feedback.
type BumpTriggerFilter = (With<Breaker>, Without<BumpVisual>);

/// Triggers a bump pop animation when the bump is first activated.
///
/// Inserts a [`BumpVisual`] component on the breaker entity. Only fires
/// when the bump timer equals the full duration (just started), preventing
/// re-triggers while the bump is still active after the animation ends.
pub fn trigger_bump_visual(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    query: Query<(Entity, &BumpState), BumpTriggerFilter>,
) {
    for (entity, bump) in &query {
        // Only trigger on fresh bumps — timer equals duration on the activation frame
        let just_started = (bump.timer - config.bump_duration).abs() < f32::EPSILON;
        if bump.active && just_started {
            commands.entity(entity).insert(BumpVisual {
                timer: config.bump_visual_duration,
                duration: config.bump_visual_duration,
                peak_offset: config.bump_visual_peak,
            });
        }
    }
}

/// Animates the bump pop — eased upward then back down.
///
/// Uses a 0→1→0 envelope with the configured [`BUMP_EASE`] curve.
/// Removes [`BumpVisual`] when the animation completes.
pub fn animate_bump_visual(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    config: Res<BreakerConfig>,
    mut query: Query<(Entity, &mut Transform, &mut BumpVisual), With<Breaker>>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut visual) in &mut query {
        // Remove previous frame's offset
        let prev_offset = bump_offset(&visual);
        transform.translation.y -= prev_offset;

        visual.timer -= dt;

        if visual.timer <= 0.0 {
            // Animation complete — snap to base position
            transform.translation.y = config.y_position;
            commands.entity(entity).remove::<BumpVisual>();
        } else {
            // Apply new eased offset
            let offset = bump_offset(&visual);
            transform.translation.y += offset;
        }
    }
}

/// Calculates the current Y offset for the bump animation.
///
/// Maps the timer to a 0→1→0 envelope: normalized progress goes 0→1
/// over the animation lifetime, then a sine envelope turns it into
/// 0→peak→0 so the breaker pops up and settles back down.
fn bump_offset(visual: &BumpVisual) -> f32 {
    // progress: 0.0 at start → 1.0 at end
    let progress = 1.0 - (visual.timer / visual.duration).clamp(0.0, 1.0);

    // Ease the progress for a snappy rise
    let eased = BUMP_EASE.sample_clamped(progress);

    // Envelope: sin(π * eased) gives 0→1→0 shape over the full range
    let envelope = (std::f32::consts::PI * eased).sin();

    envelope * visual.peak_offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_offset_starts_at_zero() {
        let config = BreakerConfig::default();
        let visual = BumpVisual {
            timer: config.bump_visual_duration,
            duration: config.bump_visual_duration,
            peak_offset: config.bump_visual_peak,
        };
        assert!(bump_offset(&visual).abs() < f32::EPSILON);
    }

    #[test]
    fn bump_offset_ends_at_zero() {
        let config = BreakerConfig::default();
        let visual = BumpVisual {
            timer: 0.0,
            duration: config.bump_visual_duration,
            peak_offset: config.bump_visual_peak,
        };
        assert!(bump_offset(&visual).abs() < 1e-5);
    }

    #[test]
    fn bump_offset_positive_mid_animation() {
        let config = BreakerConfig::default();
        let visual = BumpVisual {
            timer: config.bump_visual_duration / 2.0,
            duration: config.bump_visual_duration,
            peak_offset: config.bump_visual_peak,
        };
        assert!(
            bump_offset(&visual) > 0.0,
            "offset should be positive during animation"
        );
    }

    #[test]
    fn trigger_inserts_bump_visual_on_fresh_bump() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: config.bump_duration, // just started
                    cooldown: 0.0,
                },
            ))
            .id();

        app.add_systems(Update, trigger_bump_visual);
        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_some(),
            "BumpVisual should be inserted when bump just started"
        );
    }

    #[test]
    fn trigger_skips_mid_bump() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: config.bump_duration * 0.5, // mid-bump, not fresh
                    cooldown: 0.0,
                },
            ))
            .id();

        app.add_systems(Update, trigger_bump_visual);
        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_none(),
            "BumpVisual should NOT be inserted mid-bump"
        );
    }

    #[test]
    fn trigger_skips_inactive_bump() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: false,
                    timer: 0.0,
                    cooldown: 0.0,
                },
            ))
            .id();

        app.add_systems(Update, trigger_bump_visual);
        app.update();

        assert!(
            app.world().get::<BumpVisual>(entity).is_none(),
            "BumpVisual should not be inserted when bump is inactive"
        );
    }

    #[test]
    fn trigger_does_not_retrigger_while_animating() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BreakerConfig>();
        let config = app.world().resource::<BreakerConfig>().clone();

        let entity = app
            .world_mut()
            .spawn((
                Breaker,
                BumpState {
                    active: true,
                    timer: config.bump_duration,
                    cooldown: 0.0,
                },
                BumpVisual {
                    timer: 0.1,
                    duration: config.bump_visual_duration,
                    peak_offset: config.bump_visual_peak,
                },
            ))
            .id();

        app.add_systems(Update, trigger_bump_visual);
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
        app
    }

    /// Advances `Time<Fixed>` by one default timestep, then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(timestep);
        app.update();
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

        tick(&mut app);

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

        tick(&mut app);

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
            tick(&mut app);
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
