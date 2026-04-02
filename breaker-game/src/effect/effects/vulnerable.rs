use bevy::prelude::*;

/// Tracks active vulnerability multipliers on an entity.
///
/// The effective multiplier is the product of all entries (default 1.0).
/// Applied to cells — the damage system multiplies incoming damage by
/// this value when present.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveVulnerability(pub Vec<f32>);

impl ActiveVulnerability {
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

pub(crate) fn fire(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if world.get::<ActiveVulnerability>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(ActiveVulnerability::default());
    }

    if let Some(mut active) = world.get_mut::<ActiveVulnerability>(entity) {
        active.0.push(multiplier);
    }
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveVulnerability>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - multiplier).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_pushes_multiplier_onto_active_vulnerability() {
        let mut world = World::new();
        let entity = world.spawn(ActiveVulnerability(vec![])).id();
        fire(entity, 2.0, "", &mut world);
        let active = world.get::<ActiveVulnerability>(entity).unwrap();
        assert_eq!(active.0, vec![2.0]);
    }

    #[test]
    fn fire_on_bare_entity_inserts_and_populates() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 2.0, "", &mut world);
        let active = world.get::<ActiveVulnerability>(entity).unwrap();
        assert_eq!(active.0, vec![2.0]);
    }

    #[test]
    fn reverse_removes_matching_multiplier() {
        let mut world = World::new();
        let entity = world.spawn(ActiveVulnerability(vec![2.0, 1.5])).id();
        reverse(entity, 2.0, "", &mut world);
        let active = world.get::<ActiveVulnerability>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&1.5));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 2.0, "", &mut world);
        assert!(world.get::<ActiveVulnerability>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveVulnerability(vec![])).id();
        fire(entity, 2.0, "", &mut world);
        fire(entity, 1.5, "", &mut world);
        let active = world.get::<ActiveVulnerability>(entity).unwrap();
        assert_eq!(active.0, vec![2.0, 1.5]);
    }

    #[test]
    fn multiplier_returns_product_of_all_entries() {
        let vuln = ActiveVulnerability(vec![2.0, 1.5]);
        assert!((vuln.multiplier() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_returns_one_for_empty() {
        let vuln = ActiveVulnerability(vec![]);
        assert!((vuln.multiplier() - 1.0).abs() < f32::EPSILON);
    }
}
