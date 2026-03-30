//! Shared helpers for effect fire/reverse functions.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer, WALL_LAYER,
        rng::GameRng,
    },
};

/// Returns the entity's [`Position2D`] value, or [`Vec2::ZERO`] if absent.
pub(crate) fn entity_position(world: &World, entity: Entity) -> Vec2 {
    world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0)
}

/// Computes the effective range for an area-of-effect based on stacks.
///
/// Formula: `base_range + u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX) as f32 * range_per_level`
///
/// For `stacks >= 1`, the extra range is `(stacks - 1) * range_per_level` (capped at `u16::MAX` levels).
/// For `stacks == 0`, `saturating_sub(1)` wraps to `u32::MAX` which saturates to `u16::MAX`.
pub(crate) fn effective_range(base_range: f32, range_per_level: f32, stacks: u32) -> f32 {
    let extra = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    base_range + f32::from(extra) * range_per_level
}

/// Spawn an extra bolt entity with full physics components at the given position.
///
/// Reads [`BoltConfig`] for radius/speed values and [`GameRng`] for a random
/// velocity direction. Returns the spawned entity ID. Callers insert
/// effect-specific markers (e.g., `ChainBoltMarker`, `PhantomBoltMarker`) after.
pub(crate) fn spawn_extra_bolt(world: &mut World, spawn_pos: Vec2) -> Entity {
    let config = world.resource::<BoltConfig>();
    let radius = config.radius;
    let base_speed = config.base_speed;
    let min_speed = config.min_speed;
    let max_speed = config.max_speed;

    let angle = {
        let mut rng = world.resource_mut::<GameRng>();
        rng.0.random_range(0.0..std::f32::consts::TAU)
    };
    let direction = Vec2::new(angle.cos(), angle.sin());
    let velocity = direction * base_speed;

    world
        .spawn((
            (
                Bolt,
                ExtraBolt,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: radius,
                    y: radius,
                },
                PreviousScale {
                    x: radius,
                    y: radius,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
            ),
            (
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                Velocity2D(velocity),
                BoltBaseSpeed(base_speed),
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
                BoltRadius(radius),
                CleanupOnNodeExit,
                GameDrawLayer::Bolt,
            ),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;

    // -- A4: effective_range with stacks=0 returns base_range ──────────────

    #[test]
    fn effective_range_stacks_zero_returns_base_range() {
        let result = effective_range(50.0, 10.0, 0);
        assert!(
            (result - 50.0).abs() < f32::EPSILON,
            "stacks=0: expected 50.0, got {result}"
        );
    }

    // -- A5: effective_range with stacks=1 returns base ────────────────────

    #[test]
    fn effective_range_stacks_one_returns_base() {
        let result = effective_range(100.0, 20.0, 1);
        assert!(
            (result - 100.0).abs() < f32::EPSILON,
            "stacks=1: expected 100.0, got {result}"
        );
    }

    // -- A6: effective_range with stacks=3 linear scaling ──────────────────

    #[test]
    fn effective_range_stacks_three_linear_scaling() {
        let result = effective_range(100.0, 20.0, 3);
        assert!(
            (result - 140.0).abs() < f32::EPSILON,
            "stacks=3: expected 140.0, got {result}"
        );
    }

    #[test]
    fn effective_range_stacks_two_linear_scaling() {
        let result = effective_range(100.0, 20.0, 2);
        assert!(
            (result - 120.0).abs() < f32::EPSILON,
            "stacks=2: expected 120.0, got {result}"
        );
    }

    // -- A7: effective_range with stacks=u32::MAX caps extra at u16::MAX ──

    #[test]
    fn effective_range_stacks_u32_max_caps_at_u16_max() {
        let result = effective_range(100.0, 1.0, u32::MAX);
        assert!(
            (result - 65635.0).abs() < f32::EPSILON,
            "stacks=u32::MAX: expected 65635.0, got {result}"
        );
    }

    #[test]
    fn effective_range_stacks_u16_max_plus_two_caps_at_u16_max() {
        let stacks = u32::from(u16::MAX) + 2; // 65537
        let result = effective_range(100.0, 1.0, stacks);
        assert!(
            (result - 65635.0).abs() < f32::EPSILON,
            "stacks=65537: expected 65635.0 (cap at u16::MAX), got {result}"
        );
    }

    // -- A8: spawn_extra_bolt velocity magnitude equals config.base_speed ─

    #[test]
    fn spawn_extra_bolt_velocity_magnitude_equals_base_speed() {
        let mut world = World::new();
        world.insert_resource(BoltConfig::default());
        world.insert_resource(GameRng::from_seed(42));

        let entity = spawn_extra_bolt(&mut world, Vec2::new(100.0, 200.0));

        assert!(
            world.get_entity(entity).is_ok(),
            "spawned entity should exist in the world"
        );

        let vel = world
            .get::<Velocity2D>(entity)
            .expect("entity should have Velocity2D");
        assert!(
            (vel.0.length() - 400.0).abs() < 0.01,
            "velocity magnitude should be ~400.0, got {}",
            vel.0.length()
        );

        let pos = world
            .get::<Position2D>(entity)
            .expect("entity should have Position2D");
        assert_eq!(
            pos.0,
            Vec2::new(100.0, 200.0),
            "position should match spawn position"
        );

        assert!(
            world.get::<Bolt>(entity).is_some(),
            "entity should have Bolt component"
        );
        assert!(
            world.get::<ExtraBolt>(entity).is_some(),
            "entity should have ExtraBolt component"
        );

        let base_speed = world.get::<BoltBaseSpeed>(entity).unwrap();
        assert!(
            (base_speed.0 - 400.0).abs() < f32::EPSILON,
            "BoltBaseSpeed should be 400.0"
        );
        let min_speed = world.get::<BoltMinSpeed>(entity).unwrap();
        assert!(
            (min_speed.0 - 200.0).abs() < f32::EPSILON,
            "BoltMinSpeed should be 200.0"
        );
        let max_speed = world.get::<BoltMaxSpeed>(entity).unwrap();
        assert!(
            (max_speed.0 - 800.0).abs() < f32::EPSILON,
            "BoltMaxSpeed should be 800.0"
        );
        let bolt_radius = world.get::<BoltRadius>(entity).unwrap();
        assert!(
            (bolt_radius.0 - 8.0).abs() < f32::EPSILON,
            "BoltRadius should be 8.0"
        );
    }

    #[test]
    fn spawn_extra_bolt_at_zero_position_still_has_correct_speed() {
        let mut world = World::new();
        world.insert_resource(BoltConfig::default());
        world.insert_resource(GameRng::from_seed(42));

        let entity = spawn_extra_bolt(&mut world, Vec2::ZERO);

        let vel = world
            .get::<Velocity2D>(entity)
            .expect("entity should have Velocity2D");
        assert!(
            (vel.0.length() - 400.0).abs() < 0.01,
            "velocity magnitude should be ~400.0 regardless of spawn position, got {}",
            vel.0.length()
        );

        let pos = world
            .get::<Position2D>(entity)
            .expect("entity should have Position2D");
        assert_eq!(pos.0, Vec2::ZERO, "position should be Vec2::ZERO");
    }

    #[test]
    fn spawn_extra_bolt_twice_different_direction_same_magnitude() {
        let mut world = World::new();
        world.insert_resource(BoltConfig::default());
        world.insert_resource(GameRng::from_seed(42));

        let entity1 = spawn_extra_bolt(&mut world, Vec2::new(50.0, 50.0));
        let entity2 = spawn_extra_bolt(&mut world, Vec2::new(50.0, 50.0));

        let vel1 = world.get::<Velocity2D>(entity1).unwrap().0;
        let vel2 = world.get::<Velocity2D>(entity2).unwrap().0;

        assert!(
            (vel1.length() - 400.0).abs() < 0.01,
            "first bolt velocity magnitude should be ~400.0, got {}",
            vel1.length()
        );
        assert!(
            (vel2.length() - 400.0).abs() < 0.01,
            "second bolt velocity magnitude should be ~400.0, got {}",
            vel2.length()
        );

        // Directions should differ because the RNG advances
        let dot = vel1.normalize().dot(vel2.normalize());
        assert!(
            (dot - 1.0).abs() > 0.01,
            "two bolts spawned with advancing RNG should have different directions (dot product = {dot})"
        );
    }
}
