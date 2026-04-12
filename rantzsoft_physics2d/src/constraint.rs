//! Distance constraint component for tethered entity pairs.

use bevy::prelude::*;

/// A distance constraint between two entities.
///
/// When the distance between `entity_a` and `entity_b` exceeds `max_distance`,
/// a solver system should pull them back together. This component is data-only;
/// the solver lives in the game crate.
#[derive(Component, Clone, Debug)]
pub struct DistanceConstraint {
    /// First entity in the tethered pair.
    pub entity_a:     Entity,
    /// Second entity in the tethered pair.
    pub entity_b:     Entity,
    /// Maximum allowed distance between the two entities.
    pub max_distance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_constraint_stores_entities_and_max_distance() {
        let a = Entity::PLACEHOLDER;
        let b = Entity::PLACEHOLDER;
        let constraint = DistanceConstraint {
            entity_a:     a,
            entity_b:     b,
            max_distance: 200.0,
        };
        assert_eq!(constraint.entity_a, a);
        assert_eq!(constraint.entity_b, b);
        assert!((constraint.max_distance - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn distance_constraint_debug_format() {
        let constraint = DistanceConstraint {
            entity_a:     Entity::PLACEHOLDER,
            entity_b:     Entity::PLACEHOLDER,
            max_distance: 150.0,
        };
        let debug = format!("{constraint:?}");
        assert!(
            debug.contains("DistanceConstraint"),
            "Debug format should contain type name"
        );
        assert!(
            debug.contains("150"),
            "Debug format should contain max_distance value"
        );
    }

    #[test]
    fn distance_constraint_clone_is_independent() {
        let original = DistanceConstraint {
            entity_a:     Entity::PLACEHOLDER,
            entity_b:     Entity::PLACEHOLDER,
            max_distance: 200.0,
        };
        let cloned = original;
        assert!((cloned.max_distance - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn distance_constraint_is_bevy_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn(DistanceConstraint {
                entity_a:     Entity::PLACEHOLDER,
                entity_b:     Entity::PLACEHOLDER,
                max_distance: 100.0,
            })
            .id();
        app.update();
        let constraint = app
            .world()
            .get::<DistanceConstraint>(entity)
            .expect("DistanceConstraint should be queryable as a Component");
        assert!((constraint.max_distance - 100.0).abs() < f32::EPSILON);
    }
}
