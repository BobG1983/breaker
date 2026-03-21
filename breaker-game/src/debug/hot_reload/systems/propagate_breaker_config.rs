//! System to propagate `BreakerConfig` resource changes to breaker entity components.

use bevy::prelude::*;

use crate::{
    behaviors::registry::ArchetypeRegistry,
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
            BreakerHeight, BreakerMaxSpeed, BreakerWidth, BumpEarlyWindow, BumpLateWindow,
            BumpPerfectCooldown, BumpPerfectMultiplier, BumpPerfectWindow, BumpVisualParams,
            BumpWeakCooldown, BumpWeakMultiplier, DashDuration, DashSpeedMultiplier, DashTilt,
            DashTiltEase, DecelEasing, MaxReflectionAngle, MinAngleFromHorizontal, SettleDuration,
            SettleTiltEase,
        },
        resources::BreakerConfig,
    },
    chips::definition::TriggerChain,
    shared::SelectedArchetype,
};

/// Force-overwrites breaker components on all breaker entities when `BreakerConfig` changes.
///
/// Runs in `Update` in the `HotReloadSystems::PropagateConfig` system set,
/// conditioned on `resource_changed::<BreakerConfig>`. Unlike `init_breaker_params`,
/// this system has no `Without<BreakerMaxSpeed>` filter — it always overwrites.
///
/// After stamping config-derived components, re-applies archetype bolt speed
/// multipliers from the new `ArchetypeDefinition` format.
pub(crate) fn propagate_breaker_config(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    selected: Res<SelectedArchetype>,
    registry: Res<ArchetypeRegistry>,
    query: Query<Entity, With<Breaker>>,
) {
    for entity in &query {
        commands
            .entity(entity)
            .insert((
                BreakerWidth(config.width),
                BreakerHeight(config.height),
                BreakerBaseY(config.y_position),
                BreakerMaxSpeed(config.max_speed),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                DashSpeedMultiplier(config.dash_speed_multiplier),
                DashDuration(config.dash_duration),
                DashTilt(config.dash_tilt_angle.to_radians()),
                DashTiltEase(config.dash_tilt_ease),
                BrakeTilt {
                    angle: config.brake_tilt_angle.to_radians(),
                    duration: config.brake_tilt_duration,
                    ease: config.brake_tilt_ease,
                },
                BrakeDecel(config.brake_decel_multiplier),
                MaxReflectionAngle(config.max_reflection_angle.to_radians()),
                MinAngleFromHorizontal(config.min_angle_from_horizontal.to_radians()),
            ))
            .insert((
                SettleDuration(config.settle_duration),
                SettleTiltEase(config.settle_tilt_ease),
                BumpPerfectWindow(config.perfect_window),
                BumpEarlyWindow(config.early_window),
                BumpLateWindow(config.late_window),
                BumpPerfectCooldown(config.perfect_bump_cooldown),
                BumpWeakCooldown(config.weak_bump_cooldown),
                BumpVisualParams {
                    duration: config.bump_visual_duration,
                    peak: config.bump_visual_peak,
                    peak_fraction: config.bump_visual_peak_fraction,
                    rise_ease: config.bump_visual_rise_ease,
                    fall_ease: config.bump_visual_fall_ease,
                },
            ));

        if let Some(def) = registry.get(&selected.0) {
            // Pre-stamp BoltSpeedBoost multipliers from archetype definition
            if let Some(TriggerChain::BoltSpeedBoost { multiplier }) = &def.on_perfect_bump {
                commands
                    .entity(entity)
                    .insert(BumpPerfectMultiplier(*multiplier));
            }
            if let Some(TriggerChain::BoltSpeedBoost { multiplier }) = &def.on_early_bump {
                commands
                    .entity(entity)
                    .insert(BumpWeakMultiplier(*multiplier));
            }
            if let Some(TriggerChain::BoltSpeedBoost { multiplier }) = &def.on_late_bump {
                commands
                    .entity(entity)
                    .insert(BumpWeakMultiplier(*multiplier));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behaviors::{
            definition::{ArchetypeDefinition, BreakerStatOverrides},
            effects::life_lost::LivesCount,
            registry::ArchetypeRegistry,
        },
        breaker::{
            components::{
                BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY,
                BreakerDeceleration, BreakerHeight, BreakerMaxSpeed, BreakerWidth, BumpEarlyWindow,
                BumpLateWindow, BumpPerfectCooldown, BumpPerfectMultiplier, BumpPerfectWindow,
                BumpVisualParams, BumpWeakCooldown, BumpWeakMultiplier, DashDuration,
                DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing, MaxReflectionAngle,
                MinAngleFromHorizontal, SettleDuration, SettleTiltEase,
            },
            resources::BreakerConfig,
        },
        shared::SelectedArchetype,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .init_resource::<SelectedArchetype>()
            .init_resource::<ArchetypeRegistry>()
            .add_systems(Update, propagate_breaker_config);
        app
    }

    fn spawn_breaker_with_config(world: &mut World, config: &BreakerConfig) -> Entity {
        let entity = world
            .spawn((
                Breaker,
                BreakerWidth(config.width),
                BreakerHeight(config.height),
                BreakerBaseY(config.y_position),
                BreakerMaxSpeed(config.max_speed),
                BreakerAcceleration(config.acceleration),
                BreakerDeceleration(config.deceleration),
                DecelEasing {
                    ease: config.decel_ease,
                    strength: config.decel_ease_strength,
                },
                DashSpeedMultiplier(config.dash_speed_multiplier),
                DashDuration(config.dash_duration),
                DashTilt(config.dash_tilt_angle.to_radians()),
                DashTiltEase(config.dash_tilt_ease),
            ))
            .id();
        world.entity_mut(entity).insert((
            BrakeTilt {
                angle: config.brake_tilt_angle.to_radians(),
                duration: config.brake_tilt_duration,
                ease: config.brake_tilt_ease,
            },
            BrakeDecel(config.brake_decel_multiplier),
            MaxReflectionAngle(config.max_reflection_angle.to_radians()),
            MinAngleFromHorizontal(config.min_angle_from_horizontal.to_radians()),
            SettleDuration(config.settle_duration),
            SettleTiltEase(config.settle_tilt_ease),
            BumpPerfectWindow(config.perfect_window),
            BumpEarlyWindow(config.early_window),
            BumpLateWindow(config.late_window),
            BumpPerfectCooldown(config.perfect_bump_cooldown),
            BumpWeakCooldown(config.weak_bump_cooldown),
            BumpVisualParams {
                duration: config.bump_visual_duration,
                peak: config.bump_visual_peak,
                peak_fraction: config.bump_visual_peak_fraction,
                rise_ease: config.bump_visual_rise_ease,
                fall_ease: config.bump_visual_fall_ease,
            },
            BumpPerfectMultiplier(1.0),
            BumpWeakMultiplier(1.0),
        ));
        entity
    }

    #[test]
    fn force_overwrites_max_speed_when_config_changes() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut()
            .get_mut::<BreakerMaxSpeed>(entity)
            .unwrap()
            .0 = 500.0;
        app.world_mut().resource_mut::<BreakerConfig>().max_speed = 800.0;
        app.update();

        let max_speed = app.world().get::<BreakerMaxSpeed>(entity).unwrap();
        assert!(
            (max_speed.0 - 800.0).abs() < f32::EPSILON,
            "BreakerMaxSpeed should be 800.0 after config change, got {}",
            max_speed.0
        );
    }

    #[test]
    fn force_overwrites_width_when_config_changes() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut().resource_mut::<BreakerConfig>().width = 200.0;
        app.update();

        let width = app.world().get::<BreakerWidth>(entity).unwrap();
        assert!(
            (width.0 - 200.0).abs() < f32::EPSILON,
            "BreakerWidth should be 200.0 after config change, got {}",
            width.0
        );
    }

    #[test]
    fn dash_tilt_is_stored_in_radians() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut().get_mut::<DashTilt>(entity).unwrap().0 = 999.0;
        {
            let mut c = app.world_mut().resource_mut::<BreakerConfig>();
            c.dash_tilt_angle = 15.0;
        }
        app.update();

        let tilt = app.world().get::<DashTilt>(entity).unwrap();
        let expected = 15.0_f32.to_radians();
        assert!(
            (tilt.0 - expected).abs() < 1e-5,
            "DashTilt should be {} radians (15.0 degrees), got {}",
            expected,
            tilt.0
        );
    }

    #[test]
    fn angle_components_converted_to_radians() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut()
            .get_mut::<MaxReflectionAngle>(entity)
            .unwrap()
            .0 = 999.0;
        app.world_mut()
            .get_mut::<MinAngleFromHorizontal>(entity)
            .unwrap()
            .0 = 999.0;
        {
            let mut c = app.world_mut().resource_mut::<BreakerConfig>();
            c.max_reflection_angle = 75.0;
            c.min_angle_from_horizontal = 10.0;
        }
        app.update();

        let world = app.world();
        let max_refl = world.get::<MaxReflectionAngle>(entity).unwrap();
        assert!(
            (max_refl.0 - 75.0_f32.to_radians()).abs() < 1e-5,
            "MaxReflectionAngle should be {} (75 degrees in radians), got {}",
            75.0_f32.to_radians(),
            max_refl.0
        );
        let min_angle = world.get::<MinAngleFromHorizontal>(entity).unwrap();
        assert!(
            (min_angle.0 - 10.0_f32.to_radians()).abs() < 1e-5,
            "MinAngleFromHorizontal should be {} (10 degrees in radians), got {}",
            10.0_f32.to_radians(),
            min_angle.0
        );
    }

    #[test]
    fn brake_tilt_angle_converted_to_radians() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut().get_mut::<BrakeTilt>(entity).unwrap().angle = 999.0;
        {
            let mut c = app.world_mut().resource_mut::<BreakerConfig>();
            c.brake_tilt_angle = 25.0;
        }
        app.update();

        let brake_tilt = app.world().get::<BrakeTilt>(entity).unwrap();
        assert!(
            (brake_tilt.angle - 25.0_f32.to_radians()).abs() < 1e-5,
            "BrakeTilt.angle should be {} (25 degrees in radians), got {}",
            25.0_f32.to_radians(),
            brake_tilt.angle
        );
    }

    #[test]
    fn bump_perfect_window_overwritten_from_config() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut()
            .resource_mut::<BreakerConfig>()
            .perfect_window = 0.25;
        app.update();

        let window = app.world().get::<BumpPerfectWindow>(entity).unwrap();
        assert!(
            (window.0 - 0.25).abs() < f32::EPSILON,
            "BumpPerfectWindow should be 0.25 after config change, got {}",
            window.0
        );
    }

    #[test]
    fn re_stamps_archetype_bolt_speed_multipliers() {
        const ARCHETYPE_NAME: &str = "Test";

        let def = ArchetypeDefinition {
            name: ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: Some(TriggerChain::BoltSpeedBoost { multiplier: 1.5 }),
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };

        let mut app = test_app();
        {
            let mut registry = app.world_mut().resource_mut::<ArchetypeRegistry>();
            registry.insert(ARCHETYPE_NAME.to_owned(), def);
        }
        app.world_mut()
            .insert_resource(SelectedArchetype(ARCHETYPE_NAME.to_owned()));

        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut().resource_mut::<BreakerConfig>().max_speed = 600.0;
        app.update();

        let mult = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!(
            (mult.0 - 1.5).abs() < f32::EPSILON,
            "BumpPerfectMultiplier should be re-stamped to 1.5, got {}",
            mult.0
        );
    }

    #[test]
    fn does_not_reset_lives_count_on_config_change() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let entity = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut().entity_mut(entity).insert(LivesCount(2));
        app.world_mut()
            .get_mut::<BreakerMaxSpeed>(entity)
            .unwrap()
            .0 = 999.0;
        {
            let mut c = app.world_mut().resource_mut::<BreakerConfig>();
            c.max_speed = 600.0;
        }
        app.update();

        let max_speed = app.world().get::<BreakerMaxSpeed>(entity).unwrap();
        assert!(
            (max_speed.0 - 600.0).abs() < f32::EPSILON,
            "BreakerMaxSpeed should be 600.0, confirming the system ran; got {}",
            max_speed.0
        );
        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(
            lives.0, 2,
            "LivesCount should remain 2 after config change, not be reset"
        );
    }

    #[test]
    fn updates_all_breaker_entities() {
        let mut app = test_app();
        let config = app.world().resource::<BreakerConfig>().clone();
        let e1 = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        let e2 = {
            let world = app.world_mut();
            spawn_breaker_with_config(world, &config)
        };
        app.world_mut().get_mut::<BreakerMaxSpeed>(e1).unwrap().0 = 111.0;
        app.world_mut().get_mut::<BreakerMaxSpeed>(e2).unwrap().0 = 222.0;
        app.world_mut().resource_mut::<BreakerConfig>().max_speed = 750.0;
        app.update();

        let world = app.world();
        assert!(
            (world.get::<BreakerMaxSpeed>(e1).unwrap().0 - 750.0).abs() < f32::EPSILON,
            "entity 1 BreakerMaxSpeed should be 750.0"
        );
        assert!(
            (world.get::<BreakerMaxSpeed>(e2).unwrap().0 - 750.0).abs() < f32::EPSILON,
            "entity 2 BreakerMaxSpeed should be 750.0"
        );
    }

    #[test]
    fn handles_no_breaker_entities() {
        let mut app = test_app();
        app.world_mut().resource_mut::<BreakerConfig>().max_speed = 800.0;
        app.update();
    }
}
