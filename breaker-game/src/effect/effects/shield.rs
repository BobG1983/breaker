//! Shield effect handler — temporary protection for the breaker.
//!
//! Observes [`EffectFired`], pattern-matches on [`TriggerChain::Shield`],
//! and inserts or extends [`ShieldActive`] on the breaker entity.
//! [`tick_shield`] decrements the remaining time each fixed tick and
//! removes the component when it expires.

use bevy::prelude::*;

use crate::{
    effect::events::EffectFired, breaker::components::Breaker, chips::definition::TriggerChain,
};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks a breaker as shielded. The `remaining` field counts down each tick.
///
/// When `remaining` reaches zero (or below), `tick_shield` removes the component.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ShieldActive {
    /// Seconds of shield time remaining.
    pub remaining: f32,
}

// ---------------------------------------------------------------------------
// Observer — inserts / extends ShieldActive
// ---------------------------------------------------------------------------

/// Observer: handles shield activation — inserts or extends [`ShieldActive`]
/// on the breaker entity.
///
/// Self-selects via pattern matching on [`TriggerChain::Shield`].
/// Duration formula: `base_duration + (stacks.saturating_sub(1)) * duration_per_level`.
/// If the breaker already has `ShieldActive`, the computed duration is added
/// to the existing `remaining` time (additive extension).
pub(crate) fn handle_shield(
    trigger: On<EffectFired>,
    mut breaker_query: Query<(Entity, Option<&mut ShieldActive>), With<Breaker>>,
    mut commands: Commands,
) {
    let TriggerChain::Shield {
        base_duration,
        duration_per_level,
        stacks,
    } = &trigger.event().effect
    else {
        return;
    };

    let duration = base_duration
        + f32::from(u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX))
            * duration_per_level;

    let Ok((breaker_entity, existing_shield)) = breaker_query.single_mut() else {
        return;
    };

    if let Some(mut shield) = existing_shield {
        shield.remaining += duration;
    } else {
        commands.entity(breaker_entity).insert(ShieldActive {
            remaining: duration,
        });
    }
}

// ---------------------------------------------------------------------------
// Tick system — decrements and removes
// ---------------------------------------------------------------------------

/// Decrements `ShieldActive::remaining` each fixed tick and removes the
/// component when it expires (<= 0.0).
pub(crate) fn tick_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShieldActive)>,
) {
    for (entity, mut shield) in &mut query {
        shield.remaining -= time.delta_secs();
        if shield.remaining <= 0.0 {
            commands.entity(entity).remove::<ShieldActive>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::TriggerChain;

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_shield)
            .add_systems(FixedUpdate, tick_shield);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_breaker(app: &mut App) -> Entity {
        app.world_mut().spawn(Breaker).id()
    }

    fn spawn_breaker_with_shield(app: &mut App, remaining: f32) -> Entity {
        app.world_mut()
            .spawn((Breaker, ShieldActive { remaining }))
            .id()
    }

    fn trigger_shield(app: &mut App, base_duration: f32, duration_per_level: f32, stacks: u32) {
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::Shield {
                base_duration,
                duration_per_level,
                stacks,
            },
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
    }

    // --- Observer tests ---

    #[test]
    fn shield_inserts_shield_active_on_breaker() {
        let mut app = test_app();
        let breaker = spawn_breaker(&mut app);

        trigger_shield(&mut app, 5.0, 2.0, 1);
        // Need a tick for commands to apply
        tick(&mut app);

        let shield = app
            .world()
            .get::<ShieldActive>(breaker)
            .expect("breaker should have ShieldActive after Shield effect");
        assert!(
            (shield.remaining - 5.0).abs() < 0.1,
            "ShieldActive remaining should be 5.0 (base only, stacks=1), got {:.2}",
            shield.remaining
        );
    }

    #[test]
    fn shield_stacking_formula_base_plus_extra_stacks_times_per_level() {
        let mut app = test_app();
        let breaker = spawn_breaker(&mut app);

        // Formula: 5.0 + (3-1)*2.0 = 5.0 + 4.0 = 9.0
        trigger_shield(&mut app, 5.0, 2.0, 3);
        tick(&mut app);

        let shield = app
            .world()
            .get::<ShieldActive>(breaker)
            .expect("breaker should have ShieldActive");
        assert!(
            (shield.remaining - 9.0).abs() < 0.1,
            "ShieldActive remaining should be ~9.0 (5.0 + 2*2.0), got {:.2}",
            shield.remaining
        );
    }

    #[test]
    fn shield_extends_existing_shield_additive() {
        let mut app = test_app();
        let breaker = spawn_breaker_with_shield(&mut app, 3.0);

        trigger_shield(&mut app, 5.0, 0.0, 1);
        tick(&mut app);

        let shield = app
            .world()
            .get::<ShieldActive>(breaker)
            .expect("breaker should still have ShieldActive");
        // Existing 3.0 + new 5.0 = 8.0 (minus a small tick decrement is acceptable)
        // But the observer should add to existing before the tick system decrements,
        // so we check for approximately 8.0 minus one tick's worth.
        // The key assertion: it must be significantly more than 5.0 (i.e., additive).
        assert!(
            shield.remaining > 7.0,
            "ShieldActive should be additive: existing 3.0 + new 5.0 = ~8.0, got {:.2}",
            shield.remaining
        );
    }

    #[test]
    fn shield_returns_early_when_no_breaker() {
        let mut app = test_app();
        // No breaker entity spawned

        trigger_shield(&mut app, 5.0, 0.0, 1);
        tick(&mut app);

        // Should not panic — the test passing without panic is the assertion.
        // Also verify no ShieldActive exists anywhere.
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<ShieldActive>>();
        let count = query.iter(app.world()).count();
        assert_eq!(
            count, 0,
            "no ShieldActive should exist when there is no breaker"
        );
    }

    #[test]
    fn shield_self_selects_ignores_non_shield() {
        let mut app = test_app();
        let breaker = spawn_breaker(&mut app);

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::SpawnBolt,
            bolt: None,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        assert!(
            app.world().get::<ShieldActive>(breaker).is_none(),
            "SpawnBolt effect should not insert ShieldActive on breaker (self-selection)"
        );
    }

    // --- tick_shield tests ---

    #[test]
    fn tick_shield_decrements_remaining() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, tick_shield);

        let entity = app.world_mut().spawn(ShieldActive { remaining: 5.0 }).id();

        tick(&mut app);

        let shield = app
            .world()
            .get::<ShieldActive>(entity)
            .expect("ShieldActive should still exist after one tick with remaining=5.0");
        // dt = 1/64 = 0.015625
        let expected = 5.0 - (1.0 / 64.0);
        assert!(
            (shield.remaining - expected).abs() < 0.01,
            "remaining should decrease by delta_secs (~1/64), expected ~{expected:.4}, got {:.4}",
            shield.remaining
        );
    }

    #[test]
    fn tick_shield_removes_at_zero_or_below() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, tick_shield);

        // remaining=0.01, dt ~0.0167 => goes below zero => removed
        let entity = app.world_mut().spawn(ShieldActive { remaining: 0.01 }).id();

        tick(&mut app);

        assert!(
            app.world().get::<ShieldActive>(entity).is_none(),
            "ShieldActive should be removed when remaining drops to <= 0.0"
        );
    }

    #[test]
    fn tick_shield_does_nothing_without_shield_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, tick_shield);

        let entity = app.world_mut().spawn_empty().id();

        tick(&mut app);

        // Should not panic — entity without ShieldActive is simply skipped.
        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity without ShieldActive should survive tick_shield without panic"
        );
    }

    #[test]
    fn tick_shield_removes_at_exactly_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, tick_shield);

        let entity = app.world_mut().spawn(ShieldActive { remaining: 0.0 }).id();

        tick(&mut app);

        assert!(
            app.world().get::<ShieldActive>(entity).is_none(),
            "ShieldActive with remaining=0.0 should be removed on tick"
        );
    }
}
