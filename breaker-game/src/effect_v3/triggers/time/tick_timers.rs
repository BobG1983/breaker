//! Effect timer tick system.
//!
//! Decrements all active effect timers each frame and sends
//! [`EffectTimerExpired`] when a timer reaches zero.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{components::EffectTimers, messages::EffectTimerExpired};

/// Ticks all [`EffectTimers`] components, decrementing remaining time.
///
/// When an entry reaches zero, sends [`EffectTimerExpired`] with the entity
/// and original duration, then removes the entry. If all entries are removed,
/// removes the [`EffectTimers`] component from the entity.
pub fn tick_effect_timers(
    mut query: Query<(Entity, &mut EffectTimers)>,
    time: Res<Time>,
    mut writer: MessageWriter<EffectTimerExpired>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut timers) in &mut query {
        let mut expired = Vec::new();

        for (i, (remaining, original)) in timers.timers.iter_mut().enumerate() {
            *remaining = OrderedFloat(remaining.0 - dt);
            if remaining.0 <= 0.0 {
                expired.push((i, *original));
            }
        }

        // Remove expired entries in reverse order to preserve indices
        for &(i, original) in expired.iter().rev() {
            timers.timers.swap_remove(i);
            writer.write(EffectTimerExpired {
                entity,
                original_duration: original,
            });
        }

        // Clean up component if no timers remain
        if timers.timers.is_empty() {
            commands.entity(entity).remove::<EffectTimers>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::tick_effect_timers;
    use crate::{
        effect_v3::triggers::time::{components::EffectTimers, messages::EffectTimerExpired},
        shared::test_utils::{MessageCollector, TestAppBuilder},
    };

    // -- Helpers ----------------------------------------------------------

    fn timer_test_app() -> App {
        TestAppBuilder::new()
            .with_message_capture::<EffectTimerExpired>()
            .with_system(FixedUpdate, tick_effect_timers)
            .build()
    }

    fn tick(app: &mut App) {
        crate::shared::test_utils::tick(app);
    }

    // -- Behavior 1: single timer decrements by delta time each tick ------

    #[test]
    fn single_timer_decrements_by_delta_time() {
        let mut app = timer_test_app();

        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(1.0), OrderedFloat(1.0))],
            })
            .id();

        tick(&mut app);

        let timers = app
            .world()
            .get::<EffectTimers>(entity)
            .expect("EffectTimers should still be present");
        assert_eq!(timers.timers.len(), 1);
        // TestAppBuilder uses a 64 Hz fixed timestep (dt = 1/64 s).
        // 1.0 - 1/64 = 0.984_375 — exact in f32 since 1/64 = 2^-6.
        assert_eq!(
            timers.timers[0].0,
            OrderedFloat(0.984_375),
            "remaining should be 1.0 - 1/64 = 0.984_375"
        );
    }

    #[test]
    fn timer_at_zero_immediately_expires_and_sends_message() {
        let mut app = timer_test_app();

        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(0.0), OrderedFloat(1.0))],
            })
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<EffectTimers>(entity).is_none(),
            "EffectTimers component should be removed after timer expires"
        );

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(collector.0.len(), 1, "should send one EffectTimerExpired");
        assert_eq!(collector.0[0].entity, entity);
        assert_eq!(collector.0[0].original_duration, OrderedFloat(1.0));
    }

    // -- Behavior 2: timer reaching zero sends EffectTimerExpired ---------

    #[test]
    fn timer_reaching_zero_sends_expired_message_with_correct_original_duration() {
        let mut app = timer_test_app();

        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(0.01), OrderedFloat(5.0))],
            })
            .id();

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(collector.0.len(), 1, "should send one EffectTimerExpired");
        assert_eq!(collector.0[0].entity, entity);
        assert_eq!(
            collector.0[0].original_duration,
            OrderedFloat(5.0),
            "original_duration should be 5.0"
        );
    }

    #[test]
    fn timer_going_negative_still_triggers_expired() {
        let mut app = timer_test_app();

        // remaining 0.005 < dt 0.015625 — will go negative
        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(0.005), OrderedFloat(3.0))],
            })
            .id();

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(collector.0.len(), 1);
        assert_eq!(collector.0[0].entity, entity);
        assert_eq!(collector.0[0].original_duration, OrderedFloat(3.0));
    }

    // -- Behavior 3: EffectTimers component removed when all entries expire

    #[test]
    fn component_removed_when_all_entries_expire() {
        let mut app = timer_test_app();

        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(0.001), OrderedFloat(3.0))],
            })
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<EffectTimers>(entity).is_none(),
            "EffectTimers component should be removed when all timers expire"
        );
    }

    #[test]
    fn empty_timers_vec_removes_component() {
        let mut app = timer_test_app();

        let entity = app.world_mut().spawn(EffectTimers { timers: vec![] }).id();

        tick(&mut app);

        assert!(
            app.world().get::<EffectTimers>(entity).is_none(),
            "empty EffectTimers should be removed immediately"
        );
    }

    // -- Behavior 4: multiple timers, only expired ones are removed -------

    #[test]
    fn multiple_timers_only_expired_ones_removed() {
        let mut app = timer_test_app();

        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![
                    (OrderedFloat(0.001), OrderedFloat(2.0)),
                    (OrderedFloat(10.0), OrderedFloat(10.0)),
                ],
            })
            .id();

        tick(&mut app);

        let timers = app
            .world()
            .get::<EffectTimers>(entity)
            .expect("EffectTimers should still be present (second timer alive)");
        assert_eq!(
            timers.timers.len(),
            1,
            "only the second timer should remain"
        );

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(collector.0.len(), 1, "only one timer should have expired");
        assert_eq!(collector.0[0].original_duration, OrderedFloat(2.0));
    }

    #[test]
    fn both_timers_expire_in_same_frame_sends_two_messages_and_removes_component() {
        let mut app = timer_test_app();

        let entity = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![
                    (OrderedFloat(0.001), OrderedFloat(2.0)),
                    (OrderedFloat(0.001), OrderedFloat(4.0)),
                ],
            })
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<EffectTimers>(entity).is_none(),
            "component should be removed when both timers expire"
        );

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(
            collector.0.len(),
            2,
            "two EffectTimerExpired messages should be sent"
        );

        let durations: Vec<OrderedFloat<f32>> =
            collector.0.iter().map(|m| m.original_duration).collect();
        assert!(durations.contains(&OrderedFloat(2.0)));
        assert!(durations.contains(&OrderedFloat(4.0)));
    }

    // -- Behavior 5: multiple entities with independent timers ------------

    #[test]
    fn multiple_entities_independent_timers() {
        let mut app = timer_test_app();

        let entity_a = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(0.001), OrderedFloat(1.0))],
            })
            .id();

        let entity_b = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(100.0), OrderedFloat(100.0))],
            })
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<EffectTimers>(entity_a).is_none(),
            "entity_a timer should have expired and component removed"
        );

        let timers_b = app
            .world()
            .get::<EffectTimers>(entity_b)
            .expect("entity_b should still have EffectTimers");
        assert_eq!(timers_b.timers.len(), 1);

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(collector.0.len(), 1);
        assert_eq!(collector.0[0].entity, entity_a);
    }

    #[test]
    fn no_entities_with_timers_is_noop() {
        let mut app = timer_test_app();

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert!(collector.0.is_empty(), "no messages should be sent");
    }

    // -- Behavior 6: entity field matches the entity that owned the timer -

    #[test]
    fn expired_message_entity_matches_owner() {
        let mut app = timer_test_app();

        let entity_a = app
            .world_mut()
            .spawn(EffectTimers {
                timers: vec![(OrderedFloat(0.001), OrderedFloat(7.5))],
            })
            .id();

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<EffectTimerExpired>>();
        assert_eq!(collector.0.len(), 1);
        assert_eq!(
            collector.0[0].entity, entity_a,
            "EffectTimerExpired entity should match the entity that owned the timer"
        );
        assert_eq!(
            collector.0[0].original_duration,
            OrderedFloat(7.5),
            "original_duration should match"
        );
    }
}
