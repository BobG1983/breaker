//! Normalizes bolt speed after physics constraint redistribution.
//!
//! Runs in `FixedUpdate` after `enforce_distance_constraints` to ensure
//! bolt speed satisfies `(base_speed * boost_mult).clamp(min_speed, max_speed)`.

use bevy::prelude::*;

use crate::bolt::{
    filters::ActiveFilter,
    queries::{BoltSpeedData, apply_velocity_formula},
};

/// Re-applies the canonical velocity formula to all active bolts.
///
/// After the physics constraint system redistributes velocity, bolt speed
/// may no longer satisfy `(base_speed * boost_mult).clamp(min, max)`.
/// This system corrects it by calling `apply_velocity_formula` on every
/// active bolt.
pub(crate) fn normalize_bolt_speed_after_constraints(
    mut bolt_query: Query<BoltSpeedData, ActiveFilter>,
) {
    for mut bolt in &mut bolt_query {
        apply_velocity_formula(
            &mut bolt.spatial,
            bolt.active_speed_boosts
                .map_or(1.0, crate::effect_v3::stacking::EffectStack::aggregate),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::world::CommandQueue, prelude::*};

    use super::*;
    use crate::{
        bolt::{definition::BoltDefinition, test_utils::speed_stack},
        prelude::*,
    };

    const TOLERANCE: f32 = 0.5;

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, normalize_bolt_speed_after_constraints)
            .build()
    }

    /// Creates a `BoltDefinition` with the specified speed parameters.
    fn bolt_definition(base_speed: f32, min_speed: f32, max_speed: f32) -> BoltDefinition {
        BoltDefinition {
            name: "TestBolt".to_string(),
            base_speed,
            min_speed,
            max_speed,
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

    /// Spawns a bolt with a given velocity and definition-derived speed params.
    fn spawn_bolt(
        app: &mut App,
        velocity: Vec2,
        base_speed: f32,
        min_speed: f32,
        max_speed: f32,
    ) -> Entity {
        let def = bolt_definition(base_speed, min_speed, max_speed);
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::new(0.0, 0.0))
                .definition(&def)
                .with_velocity(Velocity2D(velocity))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    }

    /// Spawns a bolt with speed boosts attached.
    fn spawn_bolt_with_boosts(
        app: &mut App,
        velocity: Vec2,
        base_speed: f32,
        min_speed: f32,
        max_speed: f32,
        boost_multipliers: Vec<f32>,
    ) -> Entity {
        let entity = spawn_bolt(app, velocity, base_speed, min_speed, max_speed);
        app.world_mut()
            .entity_mut(entity)
            .insert(speed_stack(&boost_multipliers));
        entity
    }

    #[test]
    fn bolt_speed_normalized_to_base_speed_after_constraint_modifies_velocity() {
        // Given: bolt with velocity magnitude 350 but base_speed 400
        // The constraint system might have changed the magnitude away from
        // the canonical formula value.
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 350.0); // magnitude 350, should become 400
        let entity = spawn_bolt(&mut app, velocity, 400.0, 200.0, 600.0);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 400.0).abs() < TOLERANCE,
            "bolt speed should be normalized to base_speed 400.0, got {}",
            vel.speed()
        );
    }

    #[test]
    fn bolt_with_speed_boost_normalized_to_boosted_speed() {
        // Given: bolt with velocity magnitude 350, base_speed 400, boost 1.5x
        // Expected: 400 * 1.5 = 600, clamped to max 600 -> 600
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 350.0);
        let entity = spawn_bolt_with_boosts(&mut app, velocity, 400.0, 200.0, 600.0, vec![1.5]);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 600.0).abs() < TOLERANCE,
            "bolt speed should be 400 * 1.5 = 600 (clamped to max), got {}",
            vel.speed()
        );
    }

    #[test]
    fn bolt_already_at_correct_speed_remains_unchanged() {
        // Given: bolt already at base_speed 400, no boosts
        // The velocity formula should produce 400 -> unchanged
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 400.0); // already at base_speed
        let entity = spawn_bolt(&mut app, velocity, 400.0, 200.0, 600.0);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 400.0).abs() < TOLERANCE,
            "bolt speed should remain at 400.0, got {}",
            vel.speed()
        );
    }

    #[test]
    fn direction_preserved_during_normalization() {
        // Given: bolt moving at 45 degrees with wrong magnitude (350 instead of 400)
        // After normalization: magnitude should be 400, direction preserved at ~45 degrees
        let mut app = test_app();
        let direction = Vec2::new(1.0, 1.0).normalize(); // 45 degrees
        let velocity = direction * 350.0; // wrong magnitude
        let entity = spawn_bolt(&mut app, velocity, 400.0, 200.0, 600.0);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();

        // Check magnitude is corrected to base_speed
        assert!(
            (vel.speed() - 400.0).abs() < TOLERANCE,
            "bolt speed should be normalized to 400.0, got {}",
            vel.speed()
        );

        // Check direction is preserved (both components should be roughly equal
        // for a 45-degree angle, since the angle clamping has small thresholds)
        let result_dir = vel.0.normalize();
        let angle_radians = result_dir.y.atan2(result_dir.x);
        let expected_angle = std::f32::consts::FRAC_PI_4; // 45 degrees
        // Allow some tolerance since angle clamping may adjust slightly
        assert!(
            (angle_radians - expected_angle).abs() < 0.2,
            "direction should be preserved near 45 degrees, got {angle_radians} radians"
        );
    }
}
