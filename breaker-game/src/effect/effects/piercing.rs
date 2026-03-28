use bevy::prelude::*;

/// Tracks active piercing counts on an entity.
///
/// The effective total is the sum of all entries.
#[derive(Component, Debug, Default, Clone)]
pub struct ActivePiercings(pub Vec<u32>);

impl ActivePiercings {
    /// Returns the total piercing count (sum of all entries).
    #[must_use]
    pub fn total(&self) -> u32 {
        self.0.iter().sum()
    }
}

pub(crate) fn fire(entity: Entity, count: u32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActivePiercings>(entity) {
        active.0.push(count);
    }
}

pub(crate) fn reverse(entity: Entity, count: u32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActivePiercings>(entity)
        && let Some(pos) = active.0.iter().position(|&v| v == count)
    {
        active.0.swap_remove(pos);
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, recalculate_piercing);
}

fn recalculate_piercing(query: Query<&ActivePiercings>) {
    // Placeholder -- exact recalculation wired in Wave 6
    for _active in &query {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_count_onto_active_piercings() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![])).id();
        fire(entity, 3, &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3]);
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 3, &mut world);
        assert!(world.get::<ActivePiercings>(entity).is_none());
    }

    #[test]
    fn reverse_removes_matching_count() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![3, 2])).id();
        reverse(entity, 3, &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&2));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 3, &mut world);
        assert!(world.get::<ActivePiercings>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![])).id();
        fire(entity, 3, &mut world);
        fire(entity, 2, &mut world);
        fire(entity, 1, &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3, 2, 1]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![3, 3, 2])).id();
        reverse(entity, 3, &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert!(active.0.contains(&3));
        assert!(active.0.contains(&2));
    }

    #[test]
    fn total_returns_sum_of_all_entries() {
        let piercings = ActivePiercings(vec![3, 2, 1]);
        assert_eq!(piercings.total(), 6);
    }

    #[test]
    fn total_returns_zero_for_empty() {
        let piercings = ActivePiercings(vec![]);
        assert_eq!(piercings.total(), 0);
    }
}
