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

pub(crate) fn fire(entity: Entity, count: u32, _source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if world.get::<ActivePiercings>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert((ActivePiercings::default(), EffectivePiercing::default()));
    }

    if world.get::<EffectivePiercing>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(EffectivePiercing::default());
    }

    if let Some(mut active) = world.get_mut::<ActivePiercings>(entity) {
        active.0.push(count);
    }
}

pub(crate) fn reverse(entity: Entity, count: u32, _source_chip: &str, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActivePiercings>(entity)
        && let Some(pos) = active.0.iter().position(|&v| v == count)
    {
        active.0.swap_remove(pos);
    }
}

/// Effective piercing count computed by `recalculate_piercing`.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EffectivePiercing(pub u32);

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        recalculate_piercing.in_set(crate::effect::sets::EffectSystems::Recalculate),
    );
}

fn recalculate_piercing(mut query: Query<(&ActivePiercings, &mut EffectivePiercing)>) {
    for (active, mut effective) in &mut query {
        effective.0 = active.total();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_count_onto_active_piercings() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![])).id();
        fire(entity, 3, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3]);
    }

    #[test]
    fn fire_on_bare_entity_inserts_and_populates() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 3, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3]);
        assert!(world.get::<EffectivePiercing>(entity).is_some());
    }

    #[test]
    fn fire_on_bare_entity_second_fire_appends() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 3, "", &mut world);
        fire(entity, 2, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3, 2]);
        // Effective retains default from first fire — not recalculated until system runs
        let effective = world.get::<EffectivePiercing>(entity).unwrap();
        assert_eq!(effective.0, 0);
    }

    #[test]
    fn fire_with_existing_components_preserves_effective() {
        let mut world = World::new();
        let entity = world
            .spawn((ActivePiercings(vec![]), EffectivePiercing(5)))
            .id();
        fire(entity, 3, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3]);
        // fire() must not overwrite the pre-existing effective value
        let effective = world.get::<EffectivePiercing>(entity).unwrap();
        assert_eq!(effective.0, 5);
    }

    #[test]
    fn reverse_on_bare_entity_double_call_no_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 3, "", &mut world);
        reverse(entity, 3, "", &mut world);
        assert!(world.get::<ActivePiercings>(entity).is_none());
        assert!(world.get::<EffectivePiercing>(entity).is_none());
    }

    #[test]
    fn reverse_with_non_matching_value_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![3, 2])).id();
        reverse(entity, 999, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3, 2]);
    }

    #[test]
    fn fire_on_half_initialized_entity_inserts_effective() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![])).id();
        fire(entity, 3, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3]);
        let effective = world.get::<EffectivePiercing>(entity).unwrap();
        assert_eq!(effective.0, 0);
    }

    #[test]
    fn reverse_removes_matching_count() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![3, 2])).id();
        reverse(entity, 3, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&2));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 3, "", &mut world);
        assert!(world.get::<ActivePiercings>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![])).id();
        fire(entity, 3, "", &mut world);
        fire(entity, 2, "", &mut world);
        fire(entity, 1, "", &mut world);
        let active = world.get::<ActivePiercings>(entity).unwrap();
        assert_eq!(active.0, vec![3, 2, 1]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActivePiercings(vec![3, 3, 2])).id();
        reverse(entity, 3, "", &mut world);
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

    #[test]
    fn recalculate_piercing_single_entry() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_piercing);
        let entity = app
            .world_mut()
            .spawn((ActivePiercings(vec![3]), EffectivePiercing(0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectivePiercing>(entity).unwrap();
        assert_eq!(effective.0, 3);
    }

    #[test]
    fn recalculate_piercing_multiple_entries_additive() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_piercing);
        let entity = app
            .world_mut()
            .spawn((ActivePiercings(vec![3, 2, 1]), EffectivePiercing(0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectivePiercing>(entity).unwrap();
        assert_eq!(effective.0, 6);
    }

    #[test]
    fn recalculate_piercing_empty_entries_resets_to_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_piercing);
        let entity = app
            .world_mut()
            .spawn((ActivePiercings(vec![]), EffectivePiercing(5)))
            .id();
        app.update();
        let effective = app.world().get::<EffectivePiercing>(entity).unwrap();
        assert_eq!(effective.0, 0);
    }
}
