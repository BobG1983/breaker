//! Synchronizes bolt speed to the current `EffectStack<SpeedBoostConfig>`
//! aggregate after death-trigger `FireEffectCommand`s have flushed.
//!
//! Death-trigger bridges (in `EffectV3Systems::Death`) queue
//! `FireEffectCommand`s via `commands.queue(...)`. Those commands are deferred
//! and apply at the next command flush, AFTER `bolt_cell_collision`'s
//! `apply_velocity_formula` has already run for the tick. Without a
//! re-synchronization step, the bolt's velocity stays at its old value while
//! its `EffectStack<SpeedBoostConfig>` reflects the new boost — a one-tick
//! inconsistency that the `BoltSpeedAccurate` invariant catches on tick N+1.
//!
//! This system runs `.after(EffectV3Systems::Death)` so the deferred command
//! flush has already applied the new stack entry. It re-applies
//! `apply_velocity_formula` to every active bolt, restoring the
//! `(base_speed * stack.aggregate()).clamp(min, max)` invariant within the
//! same tick that the death trigger fires.

use bevy::prelude::*;

use crate::{
    bolt::queries::{BoltSpeedData, apply_velocity_formula},
    effect_v3::stacking::EffectStack,
    prelude::Bolt,
};

/// Re-applies the canonical velocity formula to every bolt after the
/// death-trigger command queue has flushed.
///
/// Mirrors `normalize_bolt_speed_after_constraints` but runs in a different
/// part of the tick: after `EffectV3Systems::Death` so newly-pushed
/// `EffectStack<SpeedBoostConfig>` entries from death triggers are observed
/// on the same tick the cell died.
///
/// Filter rationale: `With<Bolt>` is intentionally unrestricted — death
/// triggers may push `SpeedBoost` entries onto bolts that are temporarily
/// `Birthing` or `BoltServing`. Excluding those would leave the velocity
/// out of sync with the stack until the bolt becomes Active again, which
/// the `BoltSpeedAccurate` invariant catches as a violation. The underlying
/// `apply_velocity_formula` is a no-op for zero-velocity bolts (serving
/// state), so applying it unconditionally is safe.
pub(crate) fn sync_bolt_speed_to_stack(mut bolt_query: Query<BoltSpeedData, With<Bolt>>) {
    for mut bolt in &mut bolt_query {
        apply_velocity_formula(
            &mut bolt.spatial,
            bolt.active_speed_boosts.map_or(1.0, EffectStack::aggregate),
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
            .with_system(FixedUpdate, sync_bolt_speed_to_stack)
            .build()
    }

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
    fn bolt_speed_synced_to_new_stack_aggregate() {
        // Given: bolt at 400 px/s, base_speed 400, single 1.05 multiplier in
        // stack. Expected after sync: 400 * 1.05 = 420 (within min/max bounds).
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 400.0);
        let entity = spawn_bolt_with_boosts(&mut app, velocity, 400.0, 200.0, 600.0, vec![1.05]);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 420.0).abs() < TOLERANCE,
            "bolt speed should be 400 * 1.05 = 420.0, got {}",
            vel.speed()
        );
    }

    #[test]
    fn bolt_without_speed_stack_velocity_unchanged() {
        // Given: bolt at base_speed 400, no `EffectStack<SpeedBoostConfig>`.
        // The velocity formula uses multiplier 1.0 -> velocity stays at 400.
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 400.0);
        let entity = spawn_bolt(&mut app, velocity, 400.0, 200.0, 600.0);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 400.0).abs() < TOLERANCE,
            "bolt without speed stack should remain at 400.0, got {}",
            vel.speed()
        );
    }

    #[test]
    fn bolt_with_empty_speed_stack_velocity_unchanged() {
        // Given: bolt at base_speed 400, empty `EffectStack<SpeedBoostConfig>`.
        // Empty stack aggregates to 1.0 -> velocity stays at 400.
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 400.0);
        let entity = spawn_bolt_with_boosts(&mut app, velocity, 400.0, 200.0, 600.0, vec![]);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 400.0).abs() < TOLERANCE,
            "bolt with empty speed stack should remain at 400.0, got {}",
            vel.speed()
        );
    }

    #[test]
    fn bolt_speed_clamped_to_max_when_stack_pushes_above_max() {
        // Given: bolt at base_speed 400, multiplier 2.0 -> 800, but max is 600.
        // Expected: speed clamped to 600.
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 400.0);
        let entity = spawn_bolt_with_boosts(&mut app, velocity, 400.0, 200.0, 600.0, vec![2.0]);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 600.0).abs() < TOLERANCE,
            "bolt speed should be clamped to max 600.0, got {}",
            vel.speed()
        );
    }

    #[test]
    fn bolt_speed_clamped_to_min_when_stack_pushes_below_min() {
        // Given: bolt at base_speed 400, multiplier 0.1 -> 40, but min is 200.
        // Expected: speed clamped to 200.
        let mut app = test_app();
        let velocity = Vec2::new(0.0, 400.0);
        let entity = spawn_bolt_with_boosts(&mut app, velocity, 400.0, 200.0, 600.0, vec![0.1]);

        tick(&mut app);

        let vel = app.world().get::<Velocity2D>(entity).unwrap();
        assert!(
            (vel.speed() - 200.0).abs() < TOLERANCE,
            "bolt speed should be clamped to min 200.0, got {}",
            vel.speed()
        );
    }
}
