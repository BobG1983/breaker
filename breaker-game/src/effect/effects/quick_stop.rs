//! Breaker deceleration multiplier — enables precise stops at high speed.

use bevy::prelude::*;

/// Tracks active quick stop multipliers on an entity.
///
/// Recalculation: `base_decel * product(all_boosts)`.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveQuickStops(pub Vec<f32>);

impl ActiveQuickStops {
    /// Returns the combined multiplier (product of all entries, default 1.0).
    #[must_use]
    pub fn multiplier(&self) -> f32 {
        if self.0.is_empty() {
            1.0
        } else {
            self.0.iter().product()
        }
    }
}

/// Pushes the deceleration multiplier onto `ActiveQuickStops`.
pub(crate) fn fire(entity: Entity, multiplier: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveQuickStops>(entity) {
        active.0.push(multiplier);
    }
}

/// Removes the matching multiplier entry from `ActiveQuickStops`.
pub(crate) fn reverse(entity: Entity, multiplier: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveQuickStops>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - multiplier).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

/// Registers systems for `QuickStop` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_multiplier_onto_active_quick_stops() {
        let mut world = World::new();
        let entity = world.spawn(ActiveQuickStops(vec![])).id();
        fire(entity, 2.0, &mut world);
        let active = world.get::<ActiveQuickStops>(entity).unwrap();
        assert_eq!(active.0, vec![2.0]);
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 2.0, &mut world);
        assert!(world.get::<ActiveQuickStops>(entity).is_none());
    }

    #[test]
    fn reverse_removes_matching_multiplier() {
        let mut world = World::new();
        let entity = world.spawn(ActiveQuickStops(vec![2.0, 1.5])).id();
        reverse(entity, 2.0, &mut world);
        let active = world.get::<ActiveQuickStops>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&1.5));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 2.0, &mut world);
        assert!(world.get::<ActiveQuickStops>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveQuickStops(vec![])).id();
        fire(entity, 2.0, &mut world);
        fire(entity, 1.5, &mut world);
        fire(entity, 3.0, &mut world);
        let active = world.get::<ActiveQuickStops>(entity).unwrap();
        assert_eq!(active.0, vec![2.0, 1.5, 3.0]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActiveQuickStops(vec![2.0, 2.0, 1.5])).id();
        reverse(entity, 2.0, &mut world);
        let active = world.get::<ActiveQuickStops>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert!(active.0.contains(&2.0));
        assert!(active.0.contains(&1.5));
    }

    #[test]
    fn multiplier_returns_product_of_all_entries() {
        let boosts = ActiveQuickStops(vec![2.0, 1.5]);
        assert!((boosts.multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_returns_one_for_empty() {
        let boosts = ActiveQuickStops(vec![]);
        assert!((boosts.multiplier() - 1.0).abs() < f32::EPSILON);
    }
}
