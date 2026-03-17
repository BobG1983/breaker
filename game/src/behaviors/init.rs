//! Archetype initialization systems — config overrides and component stamping.

use bevy::prelude::*;

use super::{
    active::ActiveBehaviors,
    consequences::{bolt_speed_boost::apply_bolt_speed_boosts, life_lost::LivesCount},
    definition::BreakerStatOverrides,
    registry::ArchetypeRegistry,
};
use crate::{
    breaker::{
        components::Breaker,
        resources::{BreakerConfig, BreakerDefaults},
    },
    shared::SelectedArchetype,
};

/// Applies optional stat overrides to a `BreakerConfig`.
///
/// Each `Some` field in `overrides` replaces the corresponding field in `config`.
/// Used by both `apply_archetype_config_overrides` (at init) and hot-reload
/// propagation (at runtime).
pub fn apply_stat_overrides(config: &mut BreakerConfig, overrides: &BreakerStatOverrides) {
    if let Some(width) = overrides.width {
        config.width = width;
    }
    if let Some(height) = overrides.height {
        config.height = height;
    }
    if let Some(max_speed) = overrides.max_speed {
        config.max_speed = max_speed;
    }
    if let Some(acceleration) = overrides.acceleration {
        config.acceleration = acceleration;
    }
    if let Some(deceleration) = overrides.deceleration {
        config.deceleration = deceleration;
    }
}

/// Resets `BreakerConfig` from defaults and applies archetype stat overrides.
///
/// Runs `OnEnter(GameState::Playing)` BEFORE `init_breaker_params` so that
/// stamped components reflect the overridden config values.
pub fn apply_archetype_config_overrides(
    selected: Res<SelectedArchetype>,
    registry: Res<ArchetypeRegistry>,
    defaults: Res<Assets<BreakerDefaults>>,
    mut config: ResMut<BreakerConfig>,
) {
    // Reset config from loaded RON defaults (not code defaults)
    if let Some(loaded) = defaults.iter().next().map(|(_, d)| d) {
        *config = BreakerConfig::from(loaded.clone());
    }

    // Apply archetype overrides
    let Some(def) = registry.archetypes.get(&selected.0) else {
        warn!("Archetype '{}' not found in registry", selected.0);
        return;
    };

    apply_stat_overrides(&mut config, &def.stat_overrides);
}

/// Stamps init-time behavior components and builds `ActiveBehaviors`.
///
/// Runs `OnEnter(GameState::Playing)` AFTER `init_breaker_params`.
/// - Inserts `LivesCount` if any binding uses `LoseLife`
/// - Applies `BoltSpeedBoost` bindings as multiplier components
/// - Builds `ActiveBehaviors` with ALL bindings for runtime bridge dispatch
pub fn init_archetype(
    mut commands: Commands,
    selected: Res<SelectedArchetype>,
    registry: Res<ArchetypeRegistry>,
    breaker_query: Query<Entity, (With<Breaker>, Without<LivesCount>)>,
    mut active: ResMut<ActiveBehaviors>,
) {
    let Some(def) = registry.archetypes.get(&selected.0) else {
        warn!("Archetype '{}' not found in registry", selected.0);
        return;
    };

    // Stamp init-time components on breaker entity
    for entity in &breaker_query {
        // Lives
        if let Some(life_pool) = def.life_pool {
            commands.entity(entity).insert(LivesCount(life_pool));
        }

        // Bolt speed boosts → stamp multiplier components
        apply_bolt_speed_boosts(&mut commands, entity, &def.behaviors);
    }

    // Build ActiveBehaviors — flatten multi-trigger bindings
    let mut bindings = Vec::new();
    for behavior in &def.behaviors {
        for trigger in &behavior.triggers {
            bindings.push((trigger.clone(), behavior.consequence.clone()));
        }
    }
    *active = ActiveBehaviors(bindings);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behaviors::definition::{
            ArchetypeDefinition, BehaviorBinding, BreakerStatOverrides, Consequence, Trigger,
        },
        breaker::components::{BumpPerfectMultiplier, BumpWeakMultiplier},
    };

    const TEST_ARCHETYPE_NAME: &str = "TestArchetype";

    fn make_test_archetype() -> ArchetypeDefinition {
        ArchetypeDefinition {
            name: TEST_ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            behaviors: vec![
                BehaviorBinding {
                    triggers: vec![Trigger::BoltLost],
                    consequence: Consequence::LoseLife,
                },
                BehaviorBinding {
                    triggers: vec![Trigger::PerfectBump],
                    consequence: Consequence::BoltSpeedBoost(1.5),
                },
                BehaviorBinding {
                    triggers: vec![Trigger::EarlyBump, Trigger::LateBump],
                    consequence: Consequence::BoltSpeedBoost(1.1),
                },
            ],
        }
    }

    fn test_app_with_archetype(def: ArchetypeDefinition) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut registry = ArchetypeRegistry::default();
        registry.archetypes.insert(def.name.clone(), def);
        app.insert_resource(registry);
        app.insert_resource(SelectedArchetype(TEST_ARCHETYPE_NAME.to_owned()));
        app.init_resource::<ActiveBehaviors>();
        app.add_systems(Update, init_archetype);
        app
    }

    #[test]
    fn init_archetype_stamps_lives_count() {
        let mut app = test_app_with_archetype(make_test_archetype());
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 3);
    }

    #[test]
    fn init_archetype_stamps_bump_multipliers() {
        let mut app = test_app_with_archetype(make_test_archetype());
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();

        let perfect = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!((perfect.0 - 1.5).abs() < f32::EPSILON);

        let weak = app.world().get::<BumpWeakMultiplier>(entity).unwrap();
        assert!((weak.0 - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn init_archetype_builds_active_behaviors() {
        let mut app = test_app_with_archetype(make_test_archetype());
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveBehaviors>();
        // 3 bindings: BoltLost, PerfectBump, EarlyBump, LateBump (multi-trigger expanded)
        assert_eq!(active.0.len(), 4);
        assert!(active.has_trigger(Trigger::BoltLost));
        assert!(active.has_trigger(Trigger::PerfectBump));
        assert!(active.has_trigger(Trigger::EarlyBump));
        assert!(active.has_trigger(Trigger::LateBump));
    }

    #[test]
    fn init_archetype_skips_already_initialized() {
        let mut app = test_app_with_archetype(make_test_archetype());
        // Entity already has LivesCount → should skip
        let entity = app.world_mut().spawn((Breaker, LivesCount(99))).id();
        app.update();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 99, "should not overwrite existing LivesCount");
    }

    #[test]
    fn apply_stat_overrides_partial() {
        let mut config = BreakerConfig::default();
        let original_max_speed = config.max_speed;
        let original_accel = config.acceleration;

        let overrides = BreakerStatOverrides {
            width: Some(200.0),
            height: Some(30.0),
            ..default()
        };

        apply_stat_overrides(&mut config, &overrides);

        assert!((config.width - 200.0).abs() < f32::EPSILON);
        assert!((config.height - 30.0).abs() < f32::EPSILON);
        assert!(
            (config.max_speed - original_max_speed).abs() < f32::EPSILON,
            "unset fields should remain unchanged"
        );
        assert!(
            (config.acceleration - original_accel).abs() < f32::EPSILON,
            "unset fields should remain unchanged"
        );
    }

    #[test]
    fn apply_stat_overrides_all_fields() {
        let mut config = BreakerConfig::default();
        let overrides = BreakerStatOverrides {
            width: Some(100.0),
            height: Some(20.0),
            max_speed: Some(500.0),
            acceleration: Some(1000.0),
            deceleration: Some(2000.0),
        };

        apply_stat_overrides(&mut config, &overrides);

        assert!((config.width - 100.0).abs() < f32::EPSILON);
        assert!((config.height - 20.0).abs() < f32::EPSILON);
        assert!((config.max_speed - 500.0).abs() < f32::EPSILON);
        assert!((config.acceleration - 1000.0).abs() < f32::EPSILON);
        assert!((config.deceleration - 2000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_stat_overrides_empty_is_noop() {
        let original = BreakerConfig::default();
        let mut config = BreakerConfig::default();
        let overrides = BreakerStatOverrides::default();

        apply_stat_overrides(&mut config, &overrides);

        assert!((config.width - original.width).abs() < f32::EPSILON);
        assert!((config.height - original.height).abs() < f32::EPSILON);
        assert!((config.max_speed - original.max_speed).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_overrides_modifies_config() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<BreakerDefaults>();
        app.init_resource::<BreakerConfig>();

        let def = ArchetypeDefinition {
            name: "Wide".to_owned(),
            stat_overrides: BreakerStatOverrides {
                width: Some(200.0),
                ..default()
            },
            life_pool: None,
            behaviors: vec![],
        };

        let mut registry = ArchetypeRegistry::default();
        registry.archetypes.insert("Wide".to_owned(), def);
        app.insert_resource(registry);
        app.insert_resource(SelectedArchetype("Wide".to_owned()));

        app.add_systems(Update, apply_archetype_config_overrides);
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!((config.width - 200.0).abs() < f32::EPSILON);
        // Other values should be defaults
        let default_config = BreakerConfig::default();
        assert!((config.max_speed - default_config.max_speed).abs() < f32::EPSILON);
    }

    #[test]
    fn no_life_pool_no_lives_count() {
        let def = ArchetypeDefinition {
            name: "Aegis".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            behaviors: vec![],
        };

        let mut app = test_app_with_archetype(def);
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();

        assert!(app.world().get::<LivesCount>(entity).is_none());
    }
}
