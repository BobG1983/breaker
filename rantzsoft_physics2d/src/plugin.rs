//! Plugin that registers physics resources and systems.

use bevy::prelude::*;

use crate::{resources::CollisionQuadtree, systems::maintain_quadtree};

/// Game-agnostic 2D physics plugin. Registers the `CollisionQuadtree`
/// resource and the `maintain_quadtree` system in `FixedUpdate`.
pub struct RantzPhysics2dPlugin;

impl Plugin for RantzPhysics2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionQuadtree>();
        app.add_systems(FixedUpdate, maintain_quadtree);
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
        use rantzsoft_spatial2d::components::Position2D;

        use crate::{aabb::Aabb2D, collision_layers::CollisionLayers};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.update();

        // Spawn entity with required components
        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
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
}
