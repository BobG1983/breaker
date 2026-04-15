//! Generic fade-out animation system for text entities with [`FadeOut`].

use bevy::prelude::*;

use crate::fx::components::FadeOut;

/// Ticks [`FadeOut`] timers and adjusts text alpha. Despawns when finished.
///
/// Operates on all entities with `FadeOut + TextColor` — used by bolt-lost
/// text, bump grade text, and whiff text.
pub(crate) fn animate_fade_out(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FadeOut, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fade, mut color) in &mut query {
        fade.timer -= dt;
        if fade.timer <= 0.0 {
            color.0 = color.0.with_alpha(0.0);
            commands.entity(entity).despawn();
            continue;
        }
        let t = fade.timer / fade.duration;
        // Quadratic ease-out: alpha = t * t
        let alpha = t * t;
        color.0 = color.0.with_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::prelude::*;

    /// Build a test app that advances time by `dt` each update.
    fn test_app(dt: Duration) -> App {
        TestAppBuilder::new()
            .insert_resource(TimeUpdateStrategy::ManualDuration(dt))
            .with_system(Update, animate_fade_out)
            .build()
    }

    /// Default 16ms timestep for tests that just need time to advance.
    fn default_app() -> App {
        test_app(Duration::from_millis(16))
    }

    #[test]
    fn fade_out_reduces_alpha() {
        let mut app = default_app();

        app.world_mut().spawn((
            FadeOut {
                timer:    1.0,
                duration: 1.0,
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            Text2d::new("test"),
        ));

        // First update initializes time, second advances it
        app.update();
        app.update();

        let color = app
            .world_mut()
            .query::<&TextColor>()
            .iter(app.world())
            .next()
            .expect("entity should exist");
        assert!(color.0.alpha() < 1.0, "alpha should decrease after a tick");
    }

    #[test]
    fn fade_out_despawns_when_expired() {
        let mut app = default_app();

        app.world_mut().spawn((
            FadeOut {
                timer:    0.0,
                duration: 1.0,
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            Text2d::new("test"),
        ));

        // First update: delta is 0 but timer is already <= 0 → despawn
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "entity should be despawned when timer expires");
    }

    #[test]
    fn timer_crosses_zero_during_tick_despawns_with_zero_alpha() {
        let mut app = default_app();

        // Timer slightly above zero — will cross zero during a normal tick
        let entity = app
            .world_mut()
            .spawn((
                FadeOut {
                    timer:    0.001,
                    duration: 1.0,
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                Text2d::new("test"),
            ))
            .id();

        // Two updates: first initializes time, second advances dt past the timer
        app.update();
        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "entity should be despawned when timer crosses zero mid-tick"
        );
    }

    #[test]
    fn fade_out_uses_quadratic_easing() {
        let mut app = default_app();

        let entity = app
            .world_mut()
            .spawn((
                FadeOut {
                    timer:    1.0,
                    duration: 1.0,
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                Text2d::new("test"),
            ))
            .id();

        // Manually set timer to 50% remaining to test easing directly
        app.world_mut().get_mut::<FadeOut>(entity).unwrap().timer = 0.5;

        // Run one tick to apply the easing calculation
        app.update();

        let color = app.world().get::<TextColor>(entity).unwrap();

        // At t = timer/duration. After the tick, timer < 0.5 due to dt,
        // but the alpha is set based on (timer/duration)^2.
        // With timer ≈ 0.5 (pre-dt), t ≈ 0.5, alpha ≈ 0.25.
        // Allow generous tolerance since dt subtracts a small amount first.
        let alpha = color.0.alpha();
        assert!(
            alpha < 0.5,
            "quadratic easing at ~50% time should produce alpha well below linear 0.5, got {alpha:.3}"
        );
        assert!(
            alpha > 0.0,
            "alpha should still be positive, got {alpha:.3}"
        );
    }

    #[test]
    fn fade_out_timer_ticks_down() {
        let mut app = default_app();

        let entity = app
            .world_mut()
            .spawn((
                FadeOut {
                    timer:    1.0,
                    duration: 1.0,
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                Text2d::new("test"),
            ))
            .id();

        app.update();
        app.update();

        let fade = app.world().get::<FadeOut>(entity).unwrap();
        assert!(
            fade.timer < 1.0,
            "timer should tick down, got {}",
            fade.timer
        );
    }

    #[test]
    fn multiple_fade_outs_each_tick_independently() {
        let mut app = test_app(Duration::from_millis(100));

        let fast = app
            .world_mut()
            .spawn((
                FadeOut {
                    timer:    0.5,
                    duration: 0.5,
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                Text2d::new("fast"),
            ))
            .id();

        let slow = app
            .world_mut()
            .spawn((
                FadeOut {
                    timer:    2.0,
                    duration: 2.0,
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                Text2d::new("slow"),
            ))
            .id();

        app.update();
        app.update();

        let fast_alpha = app
            .world()
            .get::<TextColor>(fast)
            .map_or(0.0, |c| c.0.alpha());
        let slow_alpha = app.world().get::<TextColor>(slow).unwrap().0.alpha();

        assert!(
            slow_alpha > fast_alpha,
            "slow fade should have higher alpha ({slow_alpha:.2}) than fast ({fast_alpha:.2})"
        );
    }

    #[test]
    fn entities_without_text_color_unaffected() {
        let mut app = default_app();

        // Entity with FadeOut but no TextColor — should not be queried
        let entity = app
            .world_mut()
            .spawn(FadeOut {
                timer:    1.0,
                duration: 1.0,
            })
            .id();

        app.update();

        // Entity should still exist (not matched by the query)
        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity without TextColor should not be affected"
        );
    }
}
