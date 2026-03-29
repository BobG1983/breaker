use bevy::prelude::*;

/// Tracks active size boost multipliers on an entity.
///
/// Recalculation: `base_size * product(all_boosts)`.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveSizeBoosts(pub Vec<f32>);

impl ActiveSizeBoosts {
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

pub(crate) fn fire(entity: Entity, value: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveSizeBoosts>(entity) {
        active.0.push(value);
    }
}

pub(crate) fn reverse(entity: Entity, value: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveSizeBoosts>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - value).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

/// Effective size multiplier computed by `recalculate_size`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EffectiveSizeMultiplier(pub f32);

impl Default for EffectiveSizeMultiplier {
    fn default() -> Self {
        Self(1.0)
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        recalculate_size.in_set(crate::effect::sets::EffectSystems::Recalculate),
    );
}

fn recalculate_size(mut query: Query<(&ActiveSizeBoosts, &mut EffectiveSizeMultiplier)>) {
    for (active, mut effective) in &mut query {
        effective.0 = active.multiplier();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_value_onto_active_size_boosts() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSizeBoosts(vec![])).id();
        fire(entity, 5.0, &mut world);
        let active = world.get::<ActiveSizeBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![5.0]);
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 5.0, &mut world);
        assert!(world.get::<ActiveSizeBoosts>(entity).is_none());
    }

    #[test]
    fn reverse_removes_matching_value() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSizeBoosts(vec![5.0, 3.0])).id();
        reverse(entity, 5.0, &mut world);
        let active = world.get::<ActiveSizeBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&3.0));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 5.0, &mut world);
        assert!(world.get::<ActiveSizeBoosts>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSizeBoosts(vec![])).id();
        fire(entity, 5.0, &mut world);
        fire(entity, 3.0, &mut world);
        fire(entity, 2.0, &mut world);
        let active = world.get::<ActiveSizeBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![5.0, 3.0, 2.0]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSizeBoosts(vec![5.0, 5.0, 3.0])).id();
        reverse(entity, 5.0, &mut world);
        let active = world.get::<ActiveSizeBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert!(active.0.contains(&5.0));
        assert!(active.0.contains(&3.0));
    }

    #[test]
    fn multiplier_returns_product_of_all_entries() {
        let boosts = ActiveSizeBoosts(vec![1.5, 2.0]);
        assert!((boosts.multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_returns_one_for_empty() {
        let boosts = ActiveSizeBoosts(vec![]);
        assert!((boosts.multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_size_single_boost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_size);
        let entity = app
            .world_mut()
            .spawn((ActiveSizeBoosts(vec![1.5]), EffectiveSizeMultiplier(1.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveSizeMultiplier>(entity).unwrap();
        assert!((effective.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_size_multiple_boosts_multiplicative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_size);
        let entity = app
            .world_mut()
            .spawn((
                ActiveSizeBoosts(vec![1.5, 2.0]),
                EffectiveSizeMultiplier(1.0),
            ))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveSizeMultiplier>(entity).unwrap();
        assert!((effective.0 - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_size_empty_boosts_resets_to_default() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_size);
        let entity = app
            .world_mut()
            .spawn((ActiveSizeBoosts(vec![]), EffectiveSizeMultiplier(2.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveSizeMultiplier>(entity).unwrap();
        assert!((effective.0 - 1.0).abs() < f32::EPSILON);
    }
}
