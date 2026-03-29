use bevy::prelude::*;

/// Tracks active speed boost multipliers on an entity.
///
/// Recalculation: `base_speed * product(all_boosts)`, clamped to [min, max].
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveSpeedBoosts(pub Vec<f32>);

pub(crate) fn fire(entity: Entity, multiplier: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveSpeedBoosts>(entity) {
        active.0.push(multiplier);
    }
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveSpeedBoosts>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - multiplier).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

/// Effective speed multiplier computed by `recalculate_speed`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EffectiveSpeedMultiplier(pub f32);

impl Default for EffectiveSpeedMultiplier {
    fn default() -> Self {
        Self(1.0)
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        recalculate_speed.in_set(crate::effect::sets::EffectSystems::Recalculate),
    );
}

impl ActiveSpeedBoosts {
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

fn recalculate_speed(mut query: Query<(&ActiveSpeedBoosts, &mut EffectiveSpeedMultiplier)>) {
    for (active, mut effective) in &mut query {
        effective.0 = active.multiplier();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_multiplier_onto_active_speed_boosts() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![])).id();
        fire(entity, 1.5, &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![1.5]);
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 1.5, &mut world);
        assert!(world.get::<ActiveSpeedBoosts>(entity).is_none());
    }

    #[test]
    fn reverse_removes_matching_multiplier() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![1.5, 2.0])).id();
        reverse(entity, 1.5, &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&2.0));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 1.5, &mut world);
        assert!(world.get::<ActiveSpeedBoosts>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![])).id();
        fire(entity, 1.5, &mut world);
        fire(entity, 2.0, &mut world);
        fire(entity, 1.25, &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![1.5, 2.0, 1.25]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![1.5, 1.5, 2.0])).id();
        reverse(entity, 1.5, &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert!(active.0.contains(&1.5));
        assert!(active.0.contains(&2.0));
    }

    #[test]
    fn multiplier_returns_product_of_all_entries() {
        let boosts = ActiveSpeedBoosts(vec![1.5, 2.0]);
        assert!((boosts.multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_returns_one_for_empty() {
        let boosts = ActiveSpeedBoosts(vec![]);
        assert!((boosts.multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_speed_single_boost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_speed);
        let entity = app
            .world_mut()
            .spawn((ActiveSpeedBoosts(vec![1.5]), EffectiveSpeedMultiplier(1.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveSpeedMultiplier>(entity).unwrap();
        assert!((effective.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_speed_multiple_boosts_multiplicative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_speed);
        let entity = app
            .world_mut()
            .spawn((
                ActiveSpeedBoosts(vec![1.5, 2.0]),
                EffectiveSpeedMultiplier(1.0),
            ))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveSpeedMultiplier>(entity).unwrap();
        assert!((effective.0 - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_speed_empty_boosts_resets_to_default() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_speed);
        let entity = app
            .world_mut()
            .spawn((ActiveSpeedBoosts(vec![]), EffectiveSpeedMultiplier(3.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveSpeedMultiplier>(entity).unwrap();
        assert!((effective.0 - 1.0).abs() < f32::EPSILON);
    }
}
