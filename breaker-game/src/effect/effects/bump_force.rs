use bevy::prelude::*;

/// Tracks active bump force multipliers on an entity.
///
/// Recalculation: `base_force * product(all_boosts)`.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveBumpForces(pub Vec<f32>);

impl ActiveBumpForces {
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

pub(crate) fn fire(entity: Entity, force: f32, _source_chip: &str, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveBumpForces>(entity) {
        active.0.push(force);
    }
}

pub(crate) fn reverse(entity: Entity, force: f32, _source_chip: &str, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveBumpForces>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - force).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

/// Effective bump force multiplier computed by `recalculate_bump_force`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EffectiveBumpForce(pub f32);

impl Default for EffectiveBumpForce {
    fn default() -> Self {
        Self(1.0)
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        recalculate_bump_force.in_set(crate::effect::sets::EffectSystems::Recalculate),
    );
}

fn recalculate_bump_force(mut query: Query<(&ActiveBumpForces, &mut EffectiveBumpForce)>) {
    for (active, mut effective) in &mut query {
        effective.0 = active.multiplier();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_force_onto_active_bump_forces() {
        let mut world = World::new();
        let entity = world.spawn(ActiveBumpForces(vec![])).id();
        fire(entity, 50.0, "", &mut world);
        let active = world.get::<ActiveBumpForces>(entity).unwrap();
        assert_eq!(active.0, vec![50.0]);
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 50.0, "", &mut world);
        assert!(world.get::<ActiveBumpForces>(entity).is_none());
    }

    #[test]
    fn reverse_removes_matching_force() {
        let mut world = World::new();
        let entity = world.spawn(ActiveBumpForces(vec![50.0, 25.0])).id();
        reverse(entity, 50.0, "", &mut world);
        let active = world.get::<ActiveBumpForces>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&25.0));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 50.0, "", &mut world);
        assert!(world.get::<ActiveBumpForces>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveBumpForces(vec![])).id();
        fire(entity, 50.0, "", &mut world);
        fire(entity, 25.0, "", &mut world);
        fire(entity, 10.0, "", &mut world);
        let active = world.get::<ActiveBumpForces>(entity).unwrap();
        assert_eq!(active.0, vec![50.0, 25.0, 10.0]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActiveBumpForces(vec![50.0, 50.0, 25.0])).id();
        reverse(entity, 50.0, "", &mut world);
        let active = world.get::<ActiveBumpForces>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert!(active.0.contains(&50.0));
        assert!(active.0.contains(&25.0));
    }

    #[test]
    fn multiplier_returns_product_of_all_entries() {
        let forces = ActiveBumpForces(vec![1.5, 2.0]);
        assert!((forces.multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_returns_one_for_empty() {
        let forces = ActiveBumpForces(vec![]);
        assert!((forces.multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_bump_force_single_boost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_bump_force);
        let entity = app
            .world_mut()
            .spawn((ActiveBumpForces(vec![1.5]), EffectiveBumpForce(1.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveBumpForce>(entity).unwrap();
        assert!((effective.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_bump_force_multiple_boosts_multiplicative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_bump_force);
        let entity = app
            .world_mut()
            .spawn((ActiveBumpForces(vec![1.5, 2.0]), EffectiveBumpForce(1.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveBumpForce>(entity).unwrap();
        assert!((effective.0 - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_bump_force_empty_boosts_resets_to_default() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_bump_force);
        let entity = app
            .world_mut()
            .spawn((ActiveBumpForces(vec![]), EffectiveBumpForce(2.0)))
            .id();
        app.update();
        let effective = app.world().get::<EffectiveBumpForce>(entity).unwrap();
        assert!((effective.0 - 1.0).abs() < f32::EPSILON);
    }
}
