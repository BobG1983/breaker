use bevy::prelude::*;

use crate::state::run::messages::RunLost;

/// Tracks remaining lives for an entity.
///
/// - `None` = infinite lives (never decremented, never incremented)
/// - `Some(n)` = finite lives (decremented on fire, incremented on reverse)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LivesCount(pub Option<u32>);

/// Decrements `LivesCount` on the entity if finite and present.
///
/// - `LivesCount(None)` (infinite lives) — no-op
/// - `LivesCount(Some(0))` — stays at 0 (saturating)
/// - `LivesCount(Some(n))` — decrements to n-1
pub(crate) fn fire(entity: Entity, _source_chip: &str, world: &mut World) {
    let reached_zero = if let Some(mut lives) = world.get_mut::<LivesCount>(entity)
        && let Some(ref mut n) = lives.0
    {
        let was_positive = *n > 0;
        *n = n.saturating_sub(1);
        was_positive && *n == 0
    } else {
        false
    };

    if reached_zero {
        world.write_message(RunLost);
    }
}

/// Restores one life — increments `LivesCount` on the entity.
///
/// - `LivesCount(None)` (infinite lives) — no-op
/// - `LivesCount(Some(n))` — increments to n+1 (saturating at `u32::MAX`)
pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    if let Some(mut lives) = world.get_mut::<LivesCount>(entity)
        && let Some(ref mut n) = lives.0
    {
        *n = n.saturating_add(1);
    }
}

/// Registers systems for `LifeLost` effect.
pub(crate) const fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_decrements_lives_count() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(Some(3))).id();

        fire(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            Some(2),
            "LivesCount(Some(3)) should become LivesCount(Some(2))"
        );
    }

    #[test]
    fn fire_does_not_decrement_below_zero() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(Some(0))).id();

        fire(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            Some(0),
            "LivesCount(Some(0)) should remain Some(0)"
        );
    }

    #[test]
    fn fire_with_infinite_lives_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(None)).id();

        fire(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, None,
            "infinite lives should remain None after fire"
        );
    }

    #[test]
    fn reverse_restores_one_life() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(Some(2))).id();

        reverse(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, Some(3), "reverse should increment LivesCount by 1");
    }

    #[test]
    fn reverse_saturates_at_max_lives() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(Some(u32::MAX))).id();

        reverse(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            Some(u32::MAX),
            "reverse at u32::MAX should saturate, not overflow"
        );
    }

    #[test]
    fn reverse_with_infinite_lives_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(LivesCount(None)).id();

        reverse(entity, "", &mut world);

        let lives = world.get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, None,
            "infinite lives should remain None after reverse"
        );
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        reverse(entity, "", &mut world);

        assert!(
            world.get::<LivesCount>(entity).is_none(),
            "reverse on entity without LivesCount should not add the component"
        );
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, "", &mut world);

        assert!(
            world.get::<LivesCount>(entity).is_none(),
            "fire on entity without LivesCount should not add the component"
        );
    }

    // ── RunLost message tests ────────────────────────────────────

    use crate::state::run::messages::RunLost;

    #[test]
    fn fire_writes_run_lost_when_lives_reach_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RunLost>();
        let entity = app.world_mut().spawn(LivesCount(Some(1))).id();

        fire(entity, "", app.world_mut());

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            Some(0),
            "LivesCount(Some(1)) should become Some(0) after fire"
        );
        let messages = app.world().resource::<Messages<RunLost>>();
        let written: Vec<&RunLost> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            1,
            "fire() should write exactly 1 RunLost message when lives reach 0, got {}",
            written.len()
        );
    }

    #[test]
    fn fire_does_not_write_run_lost_when_lives_still_positive() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RunLost>();
        let entity = app.world_mut().spawn(LivesCount(Some(3))).id();

        fire(entity, "", app.world_mut());

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            Some(2),
            "LivesCount(Some(3)) should become Some(2) after fire"
        );
        let messages = app.world().resource::<Messages<RunLost>>();
        let written: Vec<&RunLost> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            0,
            "fire() should NOT write RunLost when lives are still positive, got {}",
            written.len()
        );
    }

    #[test]
    fn fire_does_not_write_run_lost_for_infinite_lives() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RunLost>();
        let entity = app.world_mut().spawn(LivesCount(None)).id();

        fire(entity, "", app.world_mut());

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, None,
            "infinite lives should remain None after fire"
        );
        let messages = app.world().resource::<Messages<RunLost>>();
        let written: Vec<&RunLost> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            0,
            "fire() should NOT write RunLost for infinite lives, got {}",
            written.len()
        );
    }

    #[test]
    fn fire_does_not_write_run_lost_when_already_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RunLost>();
        let entity = app.world_mut().spawn(LivesCount(Some(0))).id();

        fire(entity, "", app.world_mut());

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0,
            Some(0),
            "LivesCount(Some(0)) should remain Some(0) after fire"
        );
        let messages = app.world().resource::<Messages<RunLost>>();
        let written: Vec<&RunLost> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            0,
            "fire() should NOT write RunLost when lives were already 0 (no double-trigger), got {}",
            written.len()
        );
    }
}
