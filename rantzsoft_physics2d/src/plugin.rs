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
