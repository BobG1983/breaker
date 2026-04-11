use bevy::prelude::*;
use rantzsoft_spatial2d::queries::SpatialData;

use crate::bolt::queries::apply_velocity_formula;

/// Tracks active speed boost multipliers on an entity.
///
/// Formula: `base_speed * product(all_boosts)`, clamped to [min, max]. Computed on demand via `multiplier()`.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveSpeedBoosts(pub Vec<f32>);

pub(crate) fn fire(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    if world.get_entity(entity).is_err() {
        return;
    }

    if world.get::<ActiveSpeedBoosts>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(ActiveSpeedBoosts::default());
    }

    if let Some(mut active) = world.get_mut::<ActiveSpeedBoosts>(entity) {
        active.0.push(multiplier);
    }

    recalculate_velocity(entity, world);
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) {
    if let Some(mut active) = world.get_mut::<ActiveSpeedBoosts>(entity)
        && let Some(pos) = active
            .0
            .iter()
            .position(|&v| (v - multiplier).abs() < f32::EPSILON)
    {
        active.0.swap_remove(pos);
    }

    recalculate_velocity(entity, world);
}

/// Recalculates bolt velocity after a speed boost change.
///
/// Queries `SpatialData` and calls `apply_velocity_formula` — the same
/// function used by collision systems.
fn recalculate_velocity(entity: Entity, world: &mut World) {
    let boosts = world.get::<ActiveSpeedBoosts>(entity).cloned();
    let mut query = world.query::<SpatialData>();
    let Ok(mut spatial) = query.get_mut(world, entity) else {
        return;
    };
    apply_velocity_formula(&mut spatial, boosts.as_ref());
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

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;

    use super::*;

    #[test]
    fn fire_pushes_multiplier_onto_active_speed_boosts() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![])).id();
        fire(entity, 1.5, "", &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![1.5]);
    }

    #[test]
    fn fire_on_bare_entity_inserts_and_populates() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 1.5, "", &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![1.5]);
    }

    #[test]
    fn reverse_on_bare_entity_double_call_no_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 1.5, "", &mut world);
        reverse(entity, 1.5, "", &mut world);
        assert!(world.get::<ActiveSpeedBoosts>(entity).is_none());
    }

    #[test]
    fn reverse_with_non_matching_value_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![1.5, 2.0])).id();
        reverse(entity, 999.0, "", &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![1.5, 2.0]);
    }

    #[test]
    fn reverse_removes_matching_multiplier() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![1.5, 2.0])).id();
        reverse(entity, 1.5, "", &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 1);
        assert!(active.0.contains(&2.0));
    }

    #[test]
    fn reverse_without_component_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 1.5, "", &mut world);
        assert!(world.get::<ActiveSpeedBoosts>(entity).is_none());
    }

    #[test]
    fn multiple_fires_stack() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![])).id();
        fire(entity, 1.5, "", &mut world);
        fire(entity, 2.0, "", &mut world);
        fire(entity, 1.25, "", &mut world);
        let active = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(active.0, vec![1.5, 2.0, 1.25]);
    }

    #[test]
    fn reverse_removes_only_one_matching_entry() {
        let mut world = World::new();
        let entity = world.spawn(ActiveSpeedBoosts(vec![1.5, 1.5, 2.0])).id();
        reverse(entity, 1.5, "", &mut world);
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
    fn fire_recalculates_velocity_to_reflect_new_boost() {
        use rantzsoft_spatial2d::components::Velocity2D;

        use crate::bolt::{components::Bolt, definition::BoltDefinition};

        fn test_bolt_definition() -> BoltDefinition {
            BoltDefinition {
                name: "Bolt".to_string(),
                base_speed: 400.0,
                min_speed: 200.0,
                max_speed: 800.0,
                radius: 8.0,
                base_damage: 10.0,
                effects: vec![],
                color_rgb: [6.0, 5.0, 0.5],
                min_angle_horizontal: 5.0,
                min_angle_vertical: 5.0,
                min_radius: None,
                max_radius: None,
            }
        }

        let mut world = World::new();
        let def = test_bolt_definition();
        let base_speed = def.base_speed;
        let entity = {
            let mut queue = CommandQueue::default();
            let entity = {
                let mut commands = Commands::new(&mut queue, &world);
                Bolt::builder()
                    .at_position(Vec2::ZERO)
                    .definition(&def)
                    .with_velocity(Velocity2D(Vec2::new(0.0, base_speed)))
                    .primary()
                    .headless()
                    .spawn(&mut commands)
            };
            queue.apply(&mut world);
            entity
        };

        // Apply speed boost of 1.5x
        fire(entity, 1.5, "test_chip", &mut world);

        // After boost: speed should be base_speed * 1.5
        let speed_after = world.get::<Velocity2D>(entity).unwrap().speed();
        let expected = def.base_speed * 1.5;
        assert!(
            (speed_after - expected).abs() < 1.0,
            "fire() should recalculate velocity: expected {expected}, got {speed_after}"
        );
    }

    #[test]
    fn reverse_recalculates_velocity_to_reflect_removed_boost() {
        use rantzsoft_spatial2d::components::Velocity2D;

        use crate::bolt::{components::Bolt, definition::BoltDefinition};

        fn test_bolt_definition() -> BoltDefinition {
            BoltDefinition {
                name: "Bolt".to_string(),
                base_speed: 400.0,
                min_speed: 200.0,
                max_speed: 800.0,
                radius: 8.0,
                base_damage: 10.0,
                effects: vec![],
                color_rgb: [6.0, 5.0, 0.5],
                min_angle_horizontal: 5.0,
                min_angle_vertical: 5.0,
                min_radius: None,
                max_radius: None,
            }
        }

        let mut world = World::new();
        let def = test_bolt_definition();
        let boosted_speed = def.base_speed * 1.5;
        let entity = {
            let mut queue = CommandQueue::default();
            let entity = {
                let mut commands = Commands::new(&mut queue, &world);
                Bolt::builder()
                    .at_position(Vec2::ZERO)
                    .definition(&def)
                    .with_velocity(Velocity2D(Vec2::new(0.0, boosted_speed)))
                    .primary()
                    .headless()
                    .spawn(&mut commands)
            };
            queue.apply(&mut world);
            entity
        };
        world
            .entity_mut(entity)
            .insert(ActiveSpeedBoosts(vec![1.5]));

        // Remove the boost
        reverse(entity, 1.5, "test_chip", &mut world);

        // After reverse: speed should be base_speed * 1.0
        let speed_after = world.get::<Velocity2D>(entity).unwrap().speed();
        assert!(
            (speed_after - def.base_speed).abs() < 1.0,
            "reverse() should recalculate velocity: expected {}, got {speed_after}",
            def.base_speed
        );
    }
}
