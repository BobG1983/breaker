//! Speed boost effect handler — scales bolt velocity on trigger.
//!
//! Observes [`EffectFired`], pattern-matches on
//! [`TriggerChain::SpeedBoost`], and scales the specific bolt's velocity
//! by `multiplier`, clamped within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`.

use bevy::prelude::*;

use crate::{
    behaviors::events::EffectFired,
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltVelocity},
    chips::{
        components::BoltSpeedBoost,
        definition::{SpeedBoostTarget, TriggerChain},
    },
};

/// Observer: handles speed boost when an effect fires.
///
/// Self-selects via pattern matching on [`TriggerChain::SpeedBoost`].
/// Scales the specific bolt's velocity by `multiplier` and clamps
/// within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`.
pub(crate) fn handle_speed_boost(
    trigger: On<EffectFired>,
    mut bolt_query: Query<
        (
            &mut BoltVelocity,
            &BoltBaseSpeed,
            &BoltMaxSpeed,
            Option<&BoltSpeedBoost>,
        ),
        With<Bolt>,
    >,
) {
    let TriggerChain::SpeedBoost { target, multiplier } = &trigger.event().effect else {
        return;
    };

    match target {
        SpeedBoostTarget::Bolt => {
            let Some(bolt_entity) = trigger.event().bolt else {
                return;
            };

            let Ok((mut bolt_velocity, base_speed, max_speed, speed_boost)) =
                bolt_query.get_mut(bolt_entity)
            else {
                return;
            };

            bolt_velocity.value *= *multiplier;

            let boost = speed_boost.map_or(0.0, |b| b.0);

            // Floor at effective base speed (base + boost)
            let speed = bolt_velocity.speed();
            if speed > 0.0 && speed < base_speed.0 + boost {
                bolt_velocity.value = bolt_velocity.direction() * (base_speed.0 + boost);
            }

            // Clamp to effective max speed (max + boost)
            let speed = bolt_velocity.speed();
            if speed > max_speed.0 + boost {
                bolt_velocity.value = bolt_velocity.direction() * (max_speed.0 + boost);
            }
        }
        SpeedBoostTarget::Breaker | SpeedBoostTarget::AllBolts => {
            // Future feature — no-op for now
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behaviors::events::EffectFired,
        bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltVelocity},
        chips::{
            components::BoltSpeedBoost,
            definition::{SpeedBoostTarget, TriggerChain},
        },
    };

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_speed_boost);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_bolt(app: &mut App, vx: f32, vy: f32) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(vx, vy),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
            ))
            .id()
    }

    fn spawn_bolt_with_boost(app: &mut App, vx: f32, vy: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(vx, vy),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                BoltSpeedBoost(boost),
            ))
            .id()
    }

    fn trigger_speed_boost(app: &mut App, bolt: Option<Entity>, multiplier: f32) {
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier,
            },
            bolt,
        });
        app.world_mut().flush();
        tick(app);
    }

    fn get_bolt_velocity(app: &mut App, entity: Entity) -> BoltVelocity {
        app.world()
            .entity(entity)
            .get::<BoltVelocity>()
            .expect("bolt should have BoltVelocity")
            .clone()
    }

    // --- Tests ---

    #[test]
    fn handle_speed_boost_scales_bolt_velocity_by_multiplier() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        trigger_speed_boost(&mut app, Some(bolt), 1.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 400.0 * 1.5 = 600.0, within max of 800.0
        assert!(
            (vel.speed() - 600.0).abs() < 1.0,
            "bolt speed should be ~600.0 (400.0 * 1.5), got {:.1}",
            vel.speed()
        );
        assert!(vel.value.y > 0.0, "direction should be preserved (y > 0)");
    }

    #[test]
    fn handle_speed_boost_clamps_to_max_speed() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 700.0);

        trigger_speed_boost(&mut app, Some(bolt), 1.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 700.0 * 1.5 = 1050.0, should clamp to max 800.0
        assert!(
            (vel.speed() - 800.0).abs() < 1.0,
            "bolt speed should be clamped to max 800.0 (not 1050.0), got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn handle_speed_boost_clamps_to_elevated_max_with_amp() {
        let mut app = test_app();
        let bolt = spawn_bolt_with_boost(&mut app, 0.0, 700.0, 100.0);

        trigger_speed_boost(&mut app, Some(bolt), 1.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 700.0 * 1.5 = 1050.0, effective max = 800.0 + 100.0 = 900.0
        assert!(
            (vel.speed() - 900.0).abs() < 1.0,
            "bolt speed should be clamped to elevated max 900.0 (800 + 100 boost), got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn handle_speed_boost_floors_at_base_speed() {
        let mut app = test_app();
        // Start above base so the stub (which does nothing) leaves velocity at 600.0,
        // but a correct implementation would scale 600 * 0.5 = 300.0, floor at 400.0
        let bolt = spawn_bolt(&mut app, 0.0, 600.0);

        trigger_speed_boost(&mut app, Some(bolt), 0.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 600.0 * 0.5 = 300.0, should floor at base 400.0
        assert!(
            (vel.speed() - 400.0).abs() < 1.0,
            "bolt speed should floor at base 400.0 (not 300.0), got {:.1}",
            vel.speed()
        );
        assert!(vel.value.y > 0.0, "direction should be preserved (y > 0)");
    }

    #[test]
    fn handle_speed_boost_floors_at_elevated_base_with_amp() {
        let mut app = test_app();
        // Start above elevated base so stub (which does nothing) leaves at 700.0,
        // but correct implementation: 700 * 0.5 = 350.0, floor at 500.0 (base 400 + boost 100)
        let bolt = spawn_bolt_with_boost(&mut app, 0.0, 700.0, 100.0);

        trigger_speed_boost(&mut app, Some(bolt), 0.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 700.0 * 0.5 = 350.0, effective base = 400.0 + 100.0 = 500.0
        assert!(
            (vel.speed() - 500.0).abs() < 1.0,
            "bolt speed should floor at elevated base 500.0 (400 + 100 boost), got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn handle_speed_boost_identity_multiplier_leaves_velocity_unchanged() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        trigger_speed_boost(&mut app, Some(bolt), 1.0);

        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.value.x).abs() < f32::EPSILON,
            "x should remain 0.0, got {:.4}",
            vel.value.x
        );
        assert!(
            (vel.value.y - 400.0).abs() < 1.0,
            "y should remain ~400.0, got {:.1}",
            vel.value.y
        );
    }

    #[test]
    fn handle_speed_boost_zero_velocity_remains_zero() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);

        trigger_speed_boost(&mut app, Some(bolt), 1.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            vel.speed() < f32::EPSILON,
            "zero velocity should remain zero, got {:.4}",
            vel.speed()
        );
    }

    #[test]
    fn handle_speed_boost_ignores_non_speed_boost_effects() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        // Trigger a Shockwave effect instead of SpeedBoost
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.value.y - 400.0).abs() < f32::EPSILON,
            "non-SpeedBoost effect should not change bolt velocity, got y={:.1}",
            vel.value.y
        );
    }

    #[test]
    fn handle_speed_boost_no_ops_when_bolt_is_none() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        trigger_speed_boost(&mut app, None, 1.5);

        // The bolt that exists should be unchanged
        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.value.y - 400.0).abs() < f32::EPSILON,
            "bolt velocity should be unchanged when event bolt is None, got y={:.1}",
            vel.value.y
        );
    }

    #[test]
    fn handle_speed_boost_no_ops_for_despawned_bolt() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);
        let stale_entity = app.world_mut().spawn_empty().id();
        app.world_mut().despawn(stale_entity);

        // Should not panic when given a stale/despawned entity
        trigger_speed_boost(&mut app, Some(stale_entity), 1.5);

        // Existing bolt should be unaffected
        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.value.y - 400.0).abs() < f32::EPSILON,
            "existing bolt should be unaffected when target entity is despawned, got y={:.1}",
            vel.value.y
        );
    }

    #[test]
    fn handle_speed_boost_targets_only_specific_bolt() {
        let mut app = test_app();
        let bolt_a = spawn_bolt(&mut app, 0.0, 400.0);
        let bolt_b = spawn_bolt(&mut app, 0.0, 500.0);

        trigger_speed_boost(&mut app, Some(bolt_a), 1.5);

        let vel_a = get_bolt_velocity(&mut app, bolt_a);
        let vel_b = get_bolt_velocity(&mut app, bolt_b);

        assert!(
            (vel_a.speed() - 600.0).abs() < 1.0,
            "bolt A should be scaled to ~600.0 (400 * 1.5), got {:.1}",
            vel_a.speed()
        );
        assert!(
            (vel_b.value.y - 500.0).abs() < f32::EPSILON,
            "bolt B should be unchanged at 500.0, got {:.1}",
            vel_b.value.y
        );
    }
}
