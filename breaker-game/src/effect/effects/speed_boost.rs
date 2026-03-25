//! Speed boost effect handler — scales bolt velocity on trigger.
//!
//! Observes [`SpeedBoostFired`] and scales the specific bolt's velocity
//! by `multiplier`, clamped within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
    chips::components::BoltSpeedBoost,
    effect::{definition::Target, typed_events::SpeedBoostFired},
};

/// Observer: handles speed boost when a `SpeedBoostFired` event fires.
///
/// For `Bolt` target: scales the specific bolt's velocity by `multiplier`.
/// For `AllBolts` target: scales every bolt's velocity by `multiplier`.
/// Both clamp within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`.
pub(crate) fn handle_speed_boost(
    trigger: On<SpeedBoostFired>,
    mut bolt_query: Query<
        (
            &mut Velocity2D,
            &BoltBaseSpeed,
            &BoltMaxSpeed,
            Option<&BoltSpeedBoost>,
        ),
        With<Bolt>,
    >,
) {
    let event = trigger.event();

    match event.target {
        Target::Bolt => {
            let Some(bolt_entity) = event.targets.iter().find_map(|t| match t {
                crate::effect::definition::EffectTarget::Entity(e) => Some(*e),
                _ => None,
            }) else {
                return;
            };

            let Ok((mut vel, base_speed, max_speed, speed_boost)) = bolt_query.get_mut(bolt_entity)
            else {
                return;
            };

            let boost = speed_boost.map_or(0.0, |b| b.0);
            apply_speed_scale(&mut vel, event.multiplier, base_speed.0, max_speed.0, boost);
        }
        Target::AllBolts => {
            for (mut vel, base_speed, max_speed, speed_boost) in &mut bolt_query {
                let boost = speed_boost.map_or(0.0, |b| b.0);
                apply_speed_scale(&mut vel, event.multiplier, base_speed.0, max_speed.0, boost);
            }
        }
        Target::Breaker => {
            // Future feature — no-op for now
        }
    }
}

/// Scales a bolt's velocity by `multiplier` and clamps the resulting speed
/// within `[base + boost, max + boost]`. Zero velocity remains zero.
fn apply_speed_scale(vel: &mut Velocity2D, multiplier: f32, base: f32, max: f32, boost: f32) {
    let current = vel.speed();
    if current < f32::EPSILON {
        return;
    }

    vel.0 *= multiplier;

    // Floor at effective base speed (base + boost)
    let speed = vel.speed();
    if speed > 0.0 && speed < base + boost {
        vel.0 = vel.0.normalize_or_zero() * (base + boost);
    }

    // Clamp to effective max speed (max + boost)
    let speed = vel.speed();
    if speed > max + boost {
        vel.0 = vel.0.normalize_or_zero() * (max + boost);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
        chips::components::BoltSpeedBoost,
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
                Velocity2D(Vec2::new(vx, vy)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
            ))
            .id()
    }

    fn spawn_bolt_with_boost(app: &mut App, vx: f32, vy: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(vx, vy)),
                BoltBaseSpeed(400.0),
                BoltMaxSpeed(800.0),
                BoltSpeedBoost(boost),
            ))
            .id()
    }

    fn trigger_speed_boost(app: &mut App, bolt: Option<Entity>, multiplier: f32) {
        use crate::effect::{definition::Target as EffTarget, typed_events::SpeedBoostFired};

        let targets = bolt
            .map(|e| vec![crate::effect::definition::EffectTarget::Entity(e)])
            .unwrap_or_default();
        app.world_mut().commands().trigger(SpeedBoostFired {
            target: EffTarget::Bolt,
            multiplier,
            targets,
            source_chip: None,
        });
        app.world_mut().flush();
        tick(app);
    }

    fn get_bolt_velocity(app: &mut App, entity: Entity) -> Velocity2D {
        *app.world()
            .entity(entity)
            .get::<Velocity2D>()
            .expect("bolt should have Velocity2D")
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
        assert!(vel.0.y > 0.0, "direction should be preserved (y > 0)");
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
        assert!(vel.0.y > 0.0, "direction should be preserved (y > 0)");
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
            (vel.0.x).abs() < f32::EPSILON,
            "x should remain 0.0, got {:.4}",
            vel.0.x
        );
        assert!(
            (vel.0.y - 400.0).abs() < 1.0,
            "y should remain ~400.0, got {:.1}",
            vel.0.y
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
    fn handle_speed_boost_no_ops_when_bolt_is_none() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        trigger_speed_boost(&mut app, None, 1.5);

        // The bolt that exists should be unchanged
        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "bolt velocity should be unchanged when event bolt is None, got y={:.1}",
            vel.0.y
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
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "existing bolt should be unaffected when target entity is despawned, got y={:.1}",
            vel.0.y
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
            (vel_b.0.y - 500.0).abs() < f32::EPSILON,
            "bolt B should be unchanged at 500.0, got {:.1}",
            vel_b.0.y
        );
    }

    // =========================================================================
    // AllBolts target tests
    // =========================================================================

    fn trigger_all_bolts_speed_boost(app: &mut App, multiplier: f32) {
        use crate::effect::{definition::Target as EffTarget, typed_events::SpeedBoostFired};

        app.world_mut().commands().trigger(SpeedBoostFired {
            target: EffTarget::AllBolts,
            multiplier,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(app);
    }

    #[test]
    fn all_bolts_scales_all_bolt_velocities() {
        let mut app = test_app();
        // Bolt A: speed 400.0, Bolt B: speed 500.0
        let bolt_a = spawn_bolt(&mut app, 0.0, 400.0);
        let bolt_b = spawn_bolt(&mut app, 300.0, 400.0);

        trigger_all_bolts_speed_boost(&mut app, 1.3);

        let vel_a = get_bolt_velocity(&mut app, bolt_a);
        let vel_b = get_bolt_velocity(&mut app, bolt_b);

        // Bolt A: 400.0 * 1.3 = 520.0
        assert!(
            (vel_a.speed() - 520.0).abs() < 1.0,
            "bolt A speed should be ~520.0 (400.0 * 1.3), got {:.1}",
            vel_a.speed()
        );
        // Bolt B: 500.0 * 1.3 = 650.0
        assert!(
            (vel_b.speed() - 650.0).abs() < 1.0,
            "bolt B speed should be ~650.0 (500.0 * 1.3), got {:.1}",
            vel_b.speed()
        );
        // Directions preserved
        assert!(
            vel_a.0.y > 0.0,
            "bolt A direction should be preserved (y > 0)"
        );
        assert!(
            vel_b.0.x > 0.0 && vel_b.0.y > 0.0,
            "bolt B direction should be preserved"
        );
    }

    #[test]
    fn all_bolts_clamps_to_max() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 700.0);

        trigger_all_bolts_speed_boost(&mut app, 1.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 700.0 * 1.5 = 1050.0, should clamp to max 800.0
        assert!(
            (vel.speed() - 800.0).abs() < 1.0,
            "AllBolts should clamp speed to max 800.0, got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn all_bolts_floors_at_base() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 600.0);

        trigger_all_bolts_speed_boost(&mut app, 0.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 600.0 * 0.5 = 300.0, should floor at base 400.0
        assert!(
            (vel.speed() - 400.0).abs() < 1.0,
            "AllBolts should floor speed at base 400.0, got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn all_bolts_respects_bolt_speed_boost() {
        let mut app = test_app();
        let bolt = spawn_bolt_with_boost(&mut app, 0.0, 700.0, 100.0);

        trigger_all_bolts_speed_boost(&mut app, 1.5);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 700.0 * 1.5 = 1050.0, effective max = 800.0 + 100.0 = 900.0
        assert!(
            (vel.speed() - 900.0).abs() < 1.0,
            "AllBolts should respect BoltSpeedBoost, clamping to 900.0 (800+100), got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn all_bolts_zero_velocity_stays_zero() {
        let mut app = test_app();
        let bolt_zero = spawn_bolt(&mut app, 0.0, 0.0);
        let bolt_moving = spawn_bolt(&mut app, 0.0, 400.0);

        trigger_all_bolts_speed_boost(&mut app, 1.5);

        let vel_zero = get_bolt_velocity(&mut app, bolt_zero);
        let vel_moving = get_bolt_velocity(&mut app, bolt_moving);

        assert!(
            vel_zero.speed() < f32::EPSILON,
            "zero velocity bolt should remain zero, got {:.4}",
            vel_zero.speed()
        );
        assert!(
            (vel_moving.speed() - 600.0).abs() < 1.0,
            "moving bolt should be scaled to ~600.0 (400 * 1.5), got {:.1}",
            vel_moving.speed()
        );
    }

    #[test]
    fn all_bolts_with_no_bolts_does_not_panic() {
        let mut app = test_app();
        // No bolts spawned

        trigger_all_bolts_speed_boost(&mut app, 1.5);

        // Should not panic — the test passing without panic is the assertion.
    }

    // =========================================================================
    // B12c: handle_speed_boost observes SpeedBoostFired (not EffectFired) (behavior 23)
    // =========================================================================

    fn typed_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_speed_boost);
        app
    }

    #[test]
    fn speed_boost_fired_scales_bolt_velocity() {
        use crate::effect::{definition::Target as EffectTarget, typed_events::SpeedBoostFired};

        let mut app = typed_test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        app.world_mut().commands().trigger(SpeedBoostFired {
            target: EffectTarget::Bolt,
            multiplier: 1.5,
            targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.speed() - 600.0).abs() < 1.0,
            "SpeedBoostFired should scale bolt speed to ~600.0 (400.0 * 1.5), got {:.1}",
            vel.speed()
        );
    }

    #[test]
    fn speed_boost_fired_all_bolts_scales_all() {
        use crate::effect::{definition::Target as EffectTarget, typed_events::SpeedBoostFired};

        let mut app = typed_test_app();
        let bolt_a = spawn_bolt(&mut app, 0.0, 400.0);
        let bolt_b = spawn_bolt(&mut app, 300.0, 400.0);

        app.world_mut().commands().trigger(SpeedBoostFired {
            target: EffectTarget::AllBolts,
            multiplier: 1.3,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let vel_a = get_bolt_velocity(&mut app, bolt_a);
        let vel_b = get_bolt_velocity(&mut app, bolt_b);

        assert!(
            (vel_a.speed() - 520.0).abs() < 1.0,
            "AllBolts SpeedBoostFired: bolt A speed should be ~520.0, got {:.1}",
            vel_a.speed()
        );
        assert!(
            (vel_b.speed() - 650.0).abs() < 1.0,
            "AllBolts SpeedBoostFired: bolt B speed should be ~650.0, got {:.1}",
            vel_b.speed()
        );
    }

    #[test]
    fn speed_boost_fired_clamps_to_max() {
        use crate::effect::{definition::Target as EffectTarget, typed_events::SpeedBoostFired};

        let mut app = typed_test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 700.0);

        app.world_mut().commands().trigger(SpeedBoostFired {
            target: EffectTarget::Bolt,
            multiplier: 1.5,
            targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.speed() - 800.0).abs() < 1.0,
            "SpeedBoostFired should clamp to max 800.0, got {:.1}",
            vel.speed()
        );
    }
}
