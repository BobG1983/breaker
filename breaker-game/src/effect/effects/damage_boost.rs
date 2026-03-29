use bevy::prelude::*;

/// Tracks active damage boost multipliers on an entity.
///
/// The effective multiplier is the product of all entries (default 1.0).
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveDamageBoosts(pub Vec<f32>);

impl ActiveDamageBoosts {
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

pub(crate) fn fire(entity: Entity, multiplier: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveDamageBoosts>(entity) {
        active.0.push(multiplier);
    }
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveDamageBoosts>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - multiplier).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

/// Effective damage multiplier computed by `recalculate_damage`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EffectiveDamageMultiplier(pub f32);

impl Default for EffectiveDamageMultiplier {
    fn default() -> Self {
        Self(1.0)
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        recalculate_damage.in_set(crate::effect::sets::EffectSystems::Recalculate),
    );
}

fn recalculate_damage(mut query: Query<(&ActiveDamageBoosts, &mut EffectiveDamageMultiplier)>) {
    for (active, mut effective) in &mut query {
        effective.0 = active.multiplier();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_multiplier_onto_active_damage_boosts() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();
        fire(entity, 2.0, &mut world);
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![2.0]);
    }

    #[test]
    fn fire_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 2.0, &mut world);
        assert!(world.get::<ActiveDamageBoosts>(entity).is_none());
    }

    #[test]
    fn reverse_removes_matching_multiplier() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![2.0, 1.5])).id();
        reverse(entity, 2.0, &mut world);
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&1.5));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 2.0, &mut world);
        assert!(world.get::<ActiveDamageBoosts>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![])).id();
        fire(entity, 2.0, &mut world);
        fire(entity, 1.5, &mut world);
        fire(entity, 3.0, &mut world);
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![2.0, 1.5, 3.0]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActiveDamageBoosts(vec![2.0, 2.0, 1.5])).id();
        reverse(entity, 2.0, &mut world);
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 2);
        assert!(active.0.contains(&2.0));
        assert!(active.0.contains(&1.5));
    }

    #[test]
    fn multiplier_returns_product_of_all_entries() {
        let boosts = ActiveDamageBoosts(vec![2.0, 1.5, 3.0]);
        assert!((boosts.multiplier() - 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_returns_one_for_empty() {
        let boosts = ActiveDamageBoosts(vec![]);
        assert!((boosts.multiplier() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_damage_single_boost() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_damage);
        let entity = app
            .world_mut()
            .spawn((
                ActiveDamageBoosts(vec![2.0]),
                EffectiveDamageMultiplier(1.0),
            ))
            .id();
        app.update();
        let effective = app
            .world()
            .get::<EffectiveDamageMultiplier>(entity)
            .unwrap();
        assert!((effective.0 - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_damage_multiple_boosts_multiplicative() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_damage);
        let entity = app
            .world_mut()
            .spawn((
                ActiveDamageBoosts(vec![2.0, 1.5, 3.0]),
                EffectiveDamageMultiplier(1.0),
            ))
            .id();
        app.update();
        let effective = app
            .world()
            .get::<EffectiveDamageMultiplier>(entity)
            .unwrap();
        assert!((effective.0 - 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recalculate_damage_empty_boosts_resets_to_default() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, recalculate_damage);
        let entity = app
            .world_mut()
            .spawn((ActiveDamageBoosts(vec![]), EffectiveDamageMultiplier(5.0)))
            .id();
        app.update();
        let effective = app
            .world()
            .get::<EffectiveDamageMultiplier>(entity)
            .unwrap();
        assert!((effective.0 - 1.0).abs() < f32::EPSILON);
    }
}
