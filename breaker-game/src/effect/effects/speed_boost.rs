//! Speed boost effect handler — scales bolt velocity on trigger.
//!
//! Observes [`SpeedBoostFired`] and pushes a multiplier onto the bolt's
//! [`ActiveSpeedBoosts`] vec. The [`apply_speed_boosts`] system recalculates
//! velocity from base speed * product(boosts), clamped to max.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
    chips::components::BoltSpeedBoost,
    effect::definition::EffectTarget,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a speed boost effect resolves via a triggered chain.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpeedBoostFired {
    /// Multiplier applied to the current velocity magnitude.
    pub multiplier: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Query for bolts needing speed boost handling (velocity, base/max speed, optional boost amp,
/// optional active boost tracking).
type SpeedBoostQuery = (
    &'static mut Velocity2D,
    &'static BoltBaseSpeed,
    &'static BoltMaxSpeed,
    Option<&'static BoltSpeedBoost>,
    Option<&'static mut ActiveSpeedBoosts>,
);

/// Per-bolt tracking of active speed boost multipliers.
///
/// Each entry is a multiplier (e.g. 1.5 for 50% speed increase). The
/// [`apply_speed_boosts`] system recalculates velocity as
/// `base_speed * product(boosts)`, clamped to `[base_speed, max_speed]`.
/// Until reversal removes entries from the vec.
#[derive(Component, Debug, Default, Clone, PartialEq)]
pub(crate) struct ActiveSpeedBoosts(pub Vec<f32>);

/// Recalculates bolt velocity from `BoltBaseSpeed` * product(`ActiveSpeedBoosts`),
/// clamped within [`BoltBaseSpeed`, `BoltMaxSpeed`], preserving direction.
///
/// Skips bolts with zero velocity (cannot determine direction).
pub(crate) fn apply_speed_boosts(
    mut query: Query<
        (
            &mut Velocity2D,
            &BoltBaseSpeed,
            &BoltMaxSpeed,
            &ActiveSpeedBoosts,
        ),
        With<Bolt>,
    >,
) {
    for (mut vel, base_speed, max_speed, active_boosts) in &mut query {
        let current_speed = vel.speed();
        if current_speed < f32::EPSILON {
            continue;
        }

        let direction = vel.0.normalize();
        let product: f32 = active_boosts.0.iter().product();
        let target_speed = (base_speed.0 * product).clamp(base_speed.0, max_speed.0);
        vel.0 = direction * target_speed;
    }
}

/// Observer: handles speed boost when a `SpeedBoostFired` event fires.
///
/// If `targets` contains entity references, applies to those specific bolts.
/// If `targets` is empty, applies to all bolts (`AllBolts` behavior).
/// Clamps within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`.
/// Also pushes the multiplier onto each bolt's [`ActiveSpeedBoosts`] vec
/// (if present) so that Until reversal can remove individual entries.
pub(crate) fn handle_speed_boost(
    trigger: On<SpeedBoostFired>,
    mut bolt_query: Query<SpeedBoostQuery, With<Bolt>>,
) {
    let event = trigger.event();

    let bolt_entities: Vec<_> = event
        .targets
        .iter()
        .filter_map(|t| match t {
            crate::effect::definition::EffectTarget::Entity(e) => Some(*e),
            crate::effect::definition::EffectTarget::Location(_) => None,
        })
        .collect();

    if bolt_entities.is_empty() {
        // No specific targets — apply to all bolts
        for (mut vel, base_speed, max_speed, speed_boost, mut active_boosts) in &mut bolt_query {
            let boost = speed_boost.map_or(0.0, |b| b.0);
            apply_speed_scale(&mut vel, event.multiplier, base_speed.0, max_speed.0, boost);
            if let Some(ref mut boosts) = active_boosts {
                boosts.0.push(event.multiplier);
            }
        }
    } else {
        // Apply to specific bolt entities
        for bolt_entity in bolt_entities {
            let Ok((mut vel, base_speed, max_speed, speed_boost, mut active_boosts)) =
                bolt_query.get_mut(bolt_entity)
            else {
                continue;
            };

            let boost = speed_boost.map_or(0.0, |b| b.0);
            apply_speed_scale(&mut vel, event.multiplier, base_speed.0, max_speed.0, boost);
            if let Some(ref mut boosts) = active_boosts {
                boosts.0.push(event.multiplier);
            }
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

/// Registers all observers and systems for the speed boost effect.
pub(crate) fn register(app: &mut App) {
    use crate::{
        effect::{effect_nodes::until, sets::EffectSystems},
        shared::PlayingState,
    };

    app.add_observer(handle_speed_boost);

    // Speed boost recalculation — after bridge and Until reversal
    app.add_systems(
        FixedUpdate,
        apply_speed_boosts
            .after(EffectSystems::Bridge)
            .after(until::tick_until_timers)
            .after(until::check_until_triggers)
            .run_if(in_state(PlayingState::Active)),
    );
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
        use crate::effect::typed_events::SpeedBoostFired;

        let targets = bolt
            .map(|e| vec![crate::effect::definition::EffectTarget::Entity(e)])
            .unwrap_or_default();
        app.world_mut().commands().trigger(SpeedBoostFired {
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
    fn handle_speed_boost_empty_targets_applies_to_all_bolts() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        trigger_speed_boost(&mut app, None, 1.5);

        // Empty targets = AllBolts — boost should apply
        let vel = get_bolt_velocity(&mut app, bolt);
        assert!(
            (vel.0.y - 600.0).abs() < 1.0,
            "empty targets should apply to all bolts (AllBolts), got y={:.1}",
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
        use crate::effect::typed_events::SpeedBoostFired;

        app.world_mut().commands().trigger(SpeedBoostFired {
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
        use crate::effect::typed_events::SpeedBoostFired;

        let mut app = typed_test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 400.0);

        app.world_mut().commands().trigger(SpeedBoostFired {
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
        use crate::effect::typed_events::SpeedBoostFired;

        let mut app = typed_test_app();
        let bolt_a = spawn_bolt(&mut app, 0.0, 400.0);
        let bolt_b = spawn_bolt(&mut app, 300.0, 400.0);

        app.world_mut().commands().trigger(SpeedBoostFired {
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
        use crate::effect::typed_events::SpeedBoostFired;

        let mut app = typed_test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 700.0);

        app.world_mut().commands().trigger(SpeedBoostFired {
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

    // =========================================================================
    // ActiveSpeedBoosts — vec-based speed boost management
    // =========================================================================

    fn boost_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_speed_boost)
            .add_systems(FixedUpdate, apply_speed_boosts);
        app
    }

    fn spawn_bolt_with_active_boosts(
        app: &mut App,
        vx: f32,
        vy: f32,
        base: f32,
        max: f32,
        boosts: Vec<f32>,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(vx, vy)),
                BoltBaseSpeed(base),
                BoltMaxSpeed(max),
                ActiveSpeedBoosts(boosts),
            ))
            .id()
    }

    fn get_active_speed_boosts(app: &App, entity: Entity) -> Vec<f32> {
        app.world()
            .entity(entity)
            .get::<ActiveSpeedBoosts>()
            .expect("bolt should have ActiveSpeedBoosts")
            .0
            .clone()
    }

    // --- Test 1: handle_speed_boost pushes to ActiveSpeedBoosts ---

    #[test]
    fn handle_speed_boost_pushes_to_active_speed_boosts() {
        use crate::effect::typed_events::SpeedBoostFired;

        let mut app = boost_test_app();
        let bolt = spawn_bolt_with_active_boosts(&mut app, 0.0, 400.0, 400.0, 800.0, vec![]);

        app.world_mut().commands().trigger(SpeedBoostFired {
            multiplier: 1.5,
            targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let boosts = get_active_speed_boosts(&app, bolt);
        assert_eq!(
            boosts,
            vec![1.5],
            "ActiveSpeedBoosts should contain [1.5] after SpeedBoostFired, got {boosts:?}"
        );
    }

    // --- Test 2: handle_speed_boost stacks multiple boosts ---

    #[test]
    fn handle_speed_boost_stacks_multiple_boosts() {
        use crate::effect::typed_events::SpeedBoostFired;

        let mut app = boost_test_app();
        let bolt = spawn_bolt_with_active_boosts(&mut app, 0.0, 400.0, 400.0, 800.0, vec![1.5]);

        app.world_mut().commands().trigger(SpeedBoostFired {
            multiplier: 2.0,
            targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let boosts = get_active_speed_boosts(&app, bolt);
        assert_eq!(
            boosts,
            vec![1.5, 2.0],
            "ActiveSpeedBoosts should contain [1.5, 2.0] after stacking, got {boosts:?}"
        );
    }

    // --- Test 3: apply_speed_boosts recalculates velocity from vec ---

    #[test]
    fn apply_speed_boosts_recalculates_velocity_from_vec() {
        let mut app = boost_test_app();
        let bolt =
            spawn_bolt_with_active_boosts(&mut app, 0.0, 400.0, 400.0, 2000.0, vec![1.5, 2.0]);

        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 400.0 * 1.5 * 2.0 = 1200.0, direction preserved (0, 1)
        assert!(
            (vel.speed() - 1200.0).abs() < 1.0,
            "velocity magnitude should be 1200.0 (400.0 * 1.5 * 2.0), got {:.1}",
            vel.speed()
        );
        assert!(
            (vel.0.y - 1200.0).abs() < 1.0,
            "velocity y should be ~1200.0, got {:.1}",
            vel.0.y
        );
        assert!(
            vel.0.x.abs() < f32::EPSILON,
            "velocity x should be ~0.0, got {:.4}",
            vel.0.x
        );
    }

    // --- Test 4: apply_speed_boosts clamps to max ---

    #[test]
    fn apply_speed_boosts_clamps_to_max() {
        let mut app = boost_test_app();
        let bolt = spawn_bolt_with_active_boosts(&mut app, 0.0, 400.0, 400.0, 800.0, vec![3.0]);

        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        // 400.0 * 3.0 = 1200.0, clamped to max 800.0
        assert!(
            (vel.speed() - 800.0).abs() < 1.0,
            "velocity should be clamped to max 800.0, got {:.1}",
            vel.speed()
        );
    }

    // --- Test 5: apply_speed_boosts with empty vec uses base speed ---

    #[test]
    fn apply_speed_boosts_empty_vec_uses_base_speed() {
        let mut app = boost_test_app();
        // Velocity is currently 600.0, but with empty boosts, apply should
        // recalculate to base_speed * 1.0 = 400.0
        let bolt = spawn_bolt_with_active_boosts(&mut app, 0.0, 600.0, 400.0, 800.0, vec![]);

        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        // product of empty vec = 1.0, so 400.0 * 1.0 = 400.0
        assert!(
            (vel.speed() - 400.0).abs() < 1.0,
            "with empty boosts vec, velocity should be base_speed 400.0, got {:.1}",
            vel.speed()
        );
    }

    // --- Test 6: apply_speed_boosts preserves direction ---

    #[test]
    fn apply_speed_boosts_preserves_direction() {
        let mut app = boost_test_app();
        // Diagonal velocity: (300.0, 400.0) -> magnitude 500.0
        let bolt = spawn_bolt_with_active_boosts(&mut app, 300.0, 400.0, 400.0, 2000.0, vec![2.0]);

        tick(&mut app);

        let vel = get_bolt_velocity(&mut app, bolt);
        // base_speed * 2.0 = 800.0
        assert!(
            (vel.speed() - 800.0).abs() < 1.0,
            "velocity magnitude should be 800.0 (400.0 * 2.0), got {:.1}",
            vel.speed()
        );
        // Direction should be normalized(300, 400) = (0.6, 0.8)
        let dir = vel.0.normalize();
        let expected_dir = Vec2::new(300.0, 400.0).normalize();
        assert!(
            (dir.x - expected_dir.x).abs() < 0.01,
            "direction x should be ~{:.3}, got {:.3}",
            expected_dir.x,
            dir.x
        );
        assert!(
            (dir.y - expected_dir.y).abs() < 0.01,
            "direction y should be ~{:.3}, got {:.3}",
            expected_dir.y,
            dir.y
        );
    }

    // --- Test 7: handle_speed_boost AllBolts target pushes to all bolts ---

    #[test]
    fn handle_speed_boost_all_bolts_pushes_to_all_active_speed_boosts() {
        use crate::effect::typed_events::SpeedBoostFired;

        let mut app = boost_test_app();
        let bolt_a = spawn_bolt_with_active_boosts(&mut app, 0.0, 400.0, 400.0, 2000.0, vec![1.5]);
        let bolt_b = spawn_bolt_with_active_boosts(&mut app, 0.0, 400.0, 400.0, 2000.0, vec![1.5]);

        app.world_mut().commands().trigger(SpeedBoostFired {
            multiplier: 2.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let boosts_a = get_active_speed_boosts(&app, bolt_a);
        let boosts_b = get_active_speed_boosts(&app, bolt_b);
        assert_eq!(
            boosts_a,
            vec![1.5, 2.0],
            "bolt A ActiveSpeedBoosts should be [1.5, 2.0], got {boosts_a:?}"
        );
        assert_eq!(
            boosts_b,
            vec![1.5, 2.0],
            "bolt B ActiveSpeedBoosts should be [1.5, 2.0], got {boosts_b:?}"
        );
    }
}
