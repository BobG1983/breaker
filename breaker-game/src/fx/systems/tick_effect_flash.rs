//! Countdown timer system for one-shot flash visual entities.

use bevy::prelude::*;

use crate::fx::components::EffectFlashTimer;

/// Ticks [`EffectFlashTimer`] timers. Despawns the entity when the timer reaches zero.
pub(crate) fn tick_effect_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut EffectFlashTimer)>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;

    /// Build a test app that advances time by `dt` each update.
    fn test_app(dt: Duration) -> App {
        use crate::shared::test_utils::TestAppBuilder;

        TestAppBuilder::new()
            .insert_resource(TimeUpdateStrategy::ManualDuration(dt))
            .with_system(Update, tick_effect_flash)
            .build()
    }

    /// Default 16ms timestep for tests.
    fn default_app() -> App {
        test_app(Duration::from_millis(16))
    }

    // ── Behavior 1: EffectFlashTimer ticks down by delta time each frame ──

    #[test]
    fn effect_flash_timer_ticks_down_by_delta_time() {
        let mut app = default_app();

        let entity = app.world_mut().spawn(EffectFlashTimer(0.15)).id();

        // First update primes time, second advances by 16ms
        app.update();
        app.update();

        let timer = app
            .world()
            .get::<EffectFlashTimer>(entity)
            .expect("entity should still exist");
        // 0.15 - 0.016 = 0.134
        assert!(
            (timer.0 - 0.134).abs() < 0.002,
            "timer should be approximately 0.134 after 16ms tick, got {}",
            timer.0
        );
    }

    // ── Behavior 2: Entity is despawned when EffectFlashTimer reaches zero ──

    #[test]
    fn entity_despawned_when_timer_crosses_zero() {
        let mut app = default_app();

        let entity = app.world_mut().spawn(EffectFlashTimer(0.001)).id();

        // First update primes time, second advances 16ms past the timer
        app.update();
        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "entity should be despawned when timer crosses zero (0.001 - 0.016 < 0)"
        );
    }

    #[test]
    fn entity_despawned_when_timer_already_at_zero() {
        let mut app = default_app();

        let entity = app.world_mut().spawn(EffectFlashTimer(0.0)).id();

        // First update: delta is 0 but timer is already <= 0 -> despawn
        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "entity with timer at 0.0 should be despawned on first update"
        );
    }

    // ── Behavior 3: Multiple EffectFlashTimer entities tick independently ──

    #[test]
    fn multiple_effect_flash_timers_tick_independently() {
        let mut app = test_app(Duration::from_millis(100));

        let entity_a = app.world_mut().spawn(EffectFlashTimer(0.05)).id();

        let entity_b = app.world_mut().spawn(EffectFlashTimer(0.20)).id();

        // First update primes time, second advances by 100ms
        app.update();
        app.update();

        // Entity A (0.05 - 0.1 <= 0) should be despawned
        assert!(
            app.world().get_entity(entity_a).is_err(),
            "entity A (timer 0.05) should be despawned after 100ms tick"
        );

        // Entity B (0.20 - 0.1 = 0.10) should still exist
        let timer_b = app
            .world()
            .get::<EffectFlashTimer>(entity_b)
            .expect("entity B should still exist");
        assert!(
            (timer_b.0 - 0.10).abs() < 0.02,
            "entity B timer should be approximately 0.10, got {}",
            timer_b.0
        );
    }

    // ── Behavior 4: Entities without EffectFlashTimer are unaffected ──

    #[test]
    fn entities_without_effect_flash_timer_unaffected() {
        let mut app = default_app();

        let _timer_entity = app.world_mut().spawn(EffectFlashTimer(0.15)).id();

        let bare_entity = app.world_mut().spawn(Transform::default()).id();

        app.update();
        app.update();

        assert!(
            app.world().get_entity(bare_entity).is_ok(),
            "entity without EffectFlashTimer should not be affected by tick_effect_flash"
        );
    }

    // ── Behavior 5: tick_effect_flash is registered in FxPlugin ──

    #[test]
    fn fx_plugin_registers_tick_effect_flash_running_in_node_playing() {
        use crate::shared::test_utils::TestAppBuilder;

        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                16,
            )))
            .build();

        app.add_plugins(crate::fx::FxPlugin);

        // Spawn entity with a very short timer
        let entity = app.world_mut().spawn(EffectFlashTimer(0.001)).id();

        // Two updates: time priming + tick (16ms > 0.001s timer)
        app.update();
        app.update();

        assert!(
            app.world().get_entity(entity).is_err(),
            "tick_effect_flash should be wired via FxPlugin and despawn entities in NodeState::Playing"
        );
    }

    #[test]
    fn tick_effect_flash_does_not_run_outside_node_playing() {
        use crate::shared::test_utils::TestAppBuilder;

        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            // Stay in default AppState (Loading) - NOT NodeState::Playing
            .build();

        app.add_plugins(crate::fx::FxPlugin);

        let entity = app.world_mut().spawn(EffectFlashTimer(0.001)).id();

        // Multiple updates
        app.update();
        app.update();
        app.update();

        assert!(
            app.world().get_entity(entity).is_ok(),
            "tick_effect_flash should NOT run when NOT in NodeState::Playing"
        );
    }
}
