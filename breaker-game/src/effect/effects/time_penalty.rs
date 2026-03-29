use bevy::prelude::*;

use crate::run::node::resources::NodeTimer;

/// Subtracts seconds from the node timer.
///
/// Directly mutates `NodeTimer::remaining`, clamping to 0.0 minimum.
/// The `tick_node_timer` system handles expiry detection on the next tick.
pub(crate) fn fire(_entity: Entity, seconds: f32, world: &mut World) {
    if let Some(mut timer) = world.get_resource_mut::<NodeTimer>() {
        timer.remaining = (timer.remaining - seconds).max(0.0);
    }
}

/// Adds seconds back to the node timer — reverses the penalty.
///
/// Clamps `remaining` to `total` so the timer never exceeds its configured duration.
pub(crate) fn reverse(_entity: Entity, seconds: f32, world: &mut World) {
    if let Some(mut timer) = world.get_resource_mut::<NodeTimer>() {
        let total = timer.total;
        timer.remaining = (timer.remaining + seconds).min(total);
    }
}

/// Registers systems for `TimePenalty` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::node::resources::NodeTimer;

    // ── fire() tests ──────────────────────────────────────────────

    #[test]
    fn fire_subtracts_seconds_from_node_timer_remaining() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 30.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        fire(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 25.0).abs() < f32::EPSILON,
            "remaining should be 25.0, got {}",
            timer.remaining
        );
    }

    #[test]
    fn fire_clamps_remaining_to_zero_not_negative() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 3.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        fire(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 0.0).abs() < f32::EPSILON,
            "remaining should clamp to 0.0, got {}",
            timer.remaining
        );
    }

    #[test]
    fn fire_does_not_modify_total() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 30.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        fire(entity, 10.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.total - 60.0).abs() < f32::EPSILON,
            "total should remain 60.0, got {}",
            timer.total
        );
    }

    #[test]
    fn fire_with_timer_already_at_zero_is_idempotent() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 0.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        fire(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 0.0).abs() < f32::EPSILON,
            "remaining should stay 0.0, got {}",
            timer.remaining
        );
    }

    #[test]
    fn fire_with_no_node_timer_does_not_panic() {
        let mut world = World::new();
        // No NodeTimer resource inserted
        let entity = world.spawn_empty().id();

        // Should not panic
        fire(entity, 5.0, &mut world);
    }

    // ── reverse() tests ───────────────────────────────────────────

    #[test]
    fn reverse_adds_seconds_back_to_remaining() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 25.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        reverse(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 30.0).abs() < f32::EPSILON,
            "remaining should be 30.0, got {}",
            timer.remaining
        );
    }

    #[test]
    fn reverse_restores_time_from_zero() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 0.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        reverse(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 5.0).abs() < f32::EPSILON,
            "remaining should be 5.0, got {}",
            timer.remaining
        );
    }

    #[test]
    fn reverse_does_not_modify_total() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 25.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        reverse(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.total - 60.0).abs() < f32::EPSILON,
            "total should remain 60.0, got {}",
            timer.total
        );
    }

    #[test]
    fn reverse_clamps_remaining_to_total_not_overflow() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 58.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        reverse(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 60.0).abs() < f32::EPSILON,
            "remaining should clamp to total (60.0), got {}",
            timer.remaining
        );
    }

    #[test]
    fn reverse_at_total_is_idempotent() {
        let mut world = World::new();
        world.insert_resource(NodeTimer {
            remaining: 60.0,
            total: 60.0,
        });
        let entity = world.spawn_empty().id();

        reverse(entity, 5.0, &mut world);

        let timer = world.resource::<NodeTimer>();
        assert!(
            (timer.remaining - 60.0).abs() < f32::EPSILON,
            "remaining should stay at total (60.0), got {}",
            timer.remaining
        );
    }

    #[test]
    fn reverse_with_no_node_timer_does_not_panic() {
        let mut world = World::new();
        // No NodeTimer resource inserted
        let entity = world.spawn_empty().id();

        // Should not panic
        reverse(entity, 5.0, &mut world);
    }
}
