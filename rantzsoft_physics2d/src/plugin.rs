//! Plugin that registers physics resources and systems.

use bevy::prelude::*;

use crate::{
    resources::CollisionQuadtree,
    systems::{enforce_distance_constraints, maintain_quadtree},
};

/// System sets for ordering game systems relative to physics maintenance.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSystems {
    /// Incremental quadtree maintenance. Game collision systems should run
    /// `.after(PhysicsSystems::MaintainQuadtree)`.
    MaintainQuadtree,
    /// Distance constraint solver. Runs after quadtree maintenance to ensure
    /// tethered entity pairs stay within their maximum allowed distance.
    EnforceDistanceConstraints,
}

/// Game-agnostic 2D physics plugin. Registers the `CollisionQuadtree`
/// resource, the `maintain_quadtree` system, and
/// `enforce_distance_constraints` in `FixedUpdate`.
pub struct RantzPhysics2dPlugin;

impl Plugin for RantzPhysics2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionQuadtree>();
        app.add_systems(
            FixedUpdate,
            (
                maintain_quadtree.in_set(PhysicsSystems::MaintainQuadtree),
                enforce_distance_constraints.in_set(PhysicsSystems::EnforceDistanceConstraints),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::CollisionQuadtree;

    // ── Behavior 20: Plugin registers CollisionQuadtree and maintain_quadtree ──

    #[test]
    fn plugin_registers_collision_quadtree_resource() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.update();

        assert!(
            app.world().contains_resource::<CollisionQuadtree>(),
            "CollisionQuadtree resource should exist after plugin registration"
        );
    }

    #[test]
    fn plugin_maintain_quadtree_system_runs_in_fixed_update() {
        use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D};

        use crate::{aabb::Aabb2D, collision_layers::CollisionLayers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.update();

        // Spawn entity with required components (including GlobalPosition2D)
        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
                GlobalPosition2D(Vec2::new(50.0, 50.0)),
                Position2D(Vec2::new(50.0, 50.0)),
                CollisionLayers::new(0x01, 0x01),
            ))
            .id();

        // Tick fixed update
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let cq = app.world().resource::<CollisionQuadtree>();
        assert_eq!(
            cq.quadtree.len(),
            1,
            "maintain_quadtree should have inserted the entity via FixedUpdate"
        );
        let region = Aabb2D::new(Vec2::new(50.0, 50.0), Vec2::new(10.0, 10.0));
        let results = cq.quadtree.query_aabb(&region);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entity);
    }

    // ── Behavior: Prelude re-exports all public API types ──

    #[test]
    fn prelude_provides_plugin_and_system_sets() {
        // Verify RantzPhysics2dPlugin and PhysicsSystems are accessible via prelude.
        use crate::prelude::{PhysicsSystems, RantzPhysics2dPlugin};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.update();

        assert!(
            app.world()
                .contains_resource::<crate::resources::CollisionQuadtree>(),
            "Plugin from prelude should register CollisionQuadtree"
        );

        // PhysicsSystems variants should be usable for scheduling.
        let _ = PhysicsSystems::MaintainQuadtree;
        let _ = PhysicsSystems::EnforceDistanceConstraints;
    }

    #[test]
    fn prelude_provides_aabb2d() {
        use crate::prelude::Aabb2D;

        let aabb = Aabb2D::new(Vec2::new(5.0, 10.0), Vec2::new(2.0, 3.0));
        assert_eq!(aabb.center, Vec2::new(5.0, 10.0));
        assert_eq!(aabb.half_extents, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn prelude_provides_collision_layers() {
        use crate::prelude::CollisionLayers;

        let layers = CollisionLayers::new(0x01, 0x02);
        assert_eq!(layers.membership, 0x01);
        assert_eq!(layers.mask, 0x02);
    }

    #[test]
    fn prelude_provides_distance_constraint() {
        use crate::prelude::DistanceConstraint;

        let constraint = DistanceConstraint {
            entity_a: Entity::PLACEHOLDER,
            entity_b: Entity::PLACEHOLDER,
            max_distance: 200.0,
        };
        assert!((constraint.max_distance - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn prelude_provides_collision_quadtree_and_quadtree() {
        use crate::prelude::{CollisionQuadtree, Quadtree};

        let cq = CollisionQuadtree::default();
        assert!(cq.quadtree.is_empty());

        let bounds = crate::aabb::Aabb2D::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let qt = Quadtree::new(bounds, 4, 4);
        assert!(qt.is_empty());
    }

    #[test]
    fn prelude_provides_ccd_function_and_types() {
        use crate::prelude::{Aabb2D, CCD_EPSILON, MAX_BOUNCES, RayHit, ray_vs_aabb};

        // Verify constants have expected values.
        assert_eq!(MAX_BOUNCES, 4);
        assert!((CCD_EPSILON - 0.01).abs() < f32::EPSILON);

        // Verify ray_vs_aabb is callable through the prelude.
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(43.0, 20.0));
        let hit: Option<RayHit> = ray_vs_aabb(Vec2::new(0.0, -30.0), Vec2::Y, 100.0, &aabb);
        let hit = hit.expect("ray should hit AABB from below");
        assert!(
            (hit.distance - 10.0).abs() < 0.01,
            "distance should be ~10.0, got {}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_Y);
    }

    #[test]
    fn prelude_wildcard_import_provides_all_public_types() {
        // A single wildcard import should bring in every public API type.
        use crate::prelude::*;

        // Plugin + sets
        let _ = RantzPhysics2dPlugin;
        let _ = PhysicsSystems::MaintainQuadtree;

        // AABB
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(aabb.contains_point(Vec2::ZERO));

        // Collision layers
        let layers = CollisionLayers::new(0x01, 0x01);
        assert!(layers.interacts_with(&layers));

        // Distance constraint
        let _ = DistanceConstraint {
            entity_a: Entity::PLACEHOLDER,
            entity_b: Entity::PLACEHOLDER,
            max_distance: 100.0,
        };

        // Resources
        let cq = CollisionQuadtree::default();
        assert!(cq.quadtree.is_empty());

        // CCD
        assert_eq!(MAX_BOUNCES, 4);
        assert!((CCD_EPSILON - 0.01).abs() < f32::EPSILON);
        let hit: Option<RayHit> = ray_vs_aabb(Vec2::new(0.0, -50.0), Vec2::Y, 100.0, &aabb);
        assert!(hit.is_some(), "ray_vs_aabb should be callable via prelude");

        // SweepHit
        let _ = SweepHit {
            entity: Entity::PLACEHOLDER,
            position: Vec2::ZERO,
            normal: Vec2::Y,
            remaining: 100.0,
        };
    }

    // ── Behavior 10: RayHit is publicly accessible from prelude ──

    #[test]
    fn prelude_provides_ray_hit_with_accessible_fields() {
        use crate::prelude::RayHit;

        let ray_hit = RayHit {
            distance: 42.0,
            normal: Vec2::NEG_Y,
        };
        assert!((ray_hit.distance - 42.0).abs() < f32::EPSILON);
        assert_eq!(ray_hit.normal, Vec2::NEG_Y);
    }

    // ── Behavior 11: SweepHit is publicly accessible from prelude ──

    #[test]
    fn prelude_provides_sweep_hit_with_accessible_fields() {
        use crate::prelude::SweepHit;

        let sweep_hit = SweepHit {
            entity: Entity::PLACEHOLDER,
            position: Vec2::new(10.0, 20.0),
            normal: Vec2::NEG_X,
            remaining: 55.5,
        };
        assert_eq!(sweep_hit.entity, Entity::PLACEHOLDER);
        assert_eq!(sweep_hit.position, Vec2::new(10.0, 20.0));
        assert_eq!(sweep_hit.normal, Vec2::NEG_X);
        assert!((sweep_hit.remaining - 55.5).abs() < f32::EPSILON);
    }

    // ── Behavior 10: EnforceDistanceConstraints set registered ──

    #[test]
    fn plugin_enforce_distance_constraints_corrects_taut_pair() {
        use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

        use crate::constraint::DistanceConstraint;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.update();

        // Spawn taut pair: A at (0,0), B at (300,0), max_distance=200
        let a = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 0.0)),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(300.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 0.0)),
            ))
            .id();
        app.world_mut().spawn(DistanceConstraint {
            entity_a: a,
            entity_b: b,
            max_distance: 200.0,
        });

        // Tick fixed update — enforce_distance_constraints should run via plugin
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let pos_a = app.world().get::<Position2D>(a).unwrap();
        let pos_b = app.world().get::<Position2D>(b).unwrap();

        // After correction: A.x ~50, B.x ~250
        assert!(
            (pos_a.0.x - 50.0).abs() < 0.01,
            "A.x should be ~50 after constraint correction via plugin, got {:.2}",
            pos_a.0.x,
        );
        assert!(
            (pos_b.0.x - 250.0).abs() < 0.01,
            "B.x should be ~250 after constraint correction via plugin, got {:.2}",
            pos_b.0.x,
        );
    }
}
