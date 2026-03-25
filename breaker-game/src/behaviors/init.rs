//! Archetype initialization systems — config overrides and component stamping.

use bevy::prelude::*;
use tracing::warn;

use super::{
    active::ActiveChains, definition::BreakerStatOverrides, effects::life_lost::LivesCount,
    registry::ArchetypeRegistry,
};
use crate::{
    breaker::{
        components::Breaker,
        resources::{BreakerConfig, BreakerDefaults},
    },
    chips::definition::TriggerChain,
    shared::SelectedArchetype,
};

/// Applies optional stat overrides to a `BreakerConfig`.
///
/// Each `Some` field in `overrides` replaces the corresponding field in `config`.
/// Used by both `apply_archetype_config_overrides` (at init) and hot-reload
/// propagation (at runtime).
pub(crate) const fn apply_stat_overrides(
    config: &mut BreakerConfig,
    overrides: &BreakerStatOverrides,
) {
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
pub(crate) fn apply_archetype_config_overrides(
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
    let Some(def) = registry.get(&selected.0) else {
        warn!("Archetype '{}' not found in registry", selected.0);
        return;
    };

    apply_stat_overrides(&mut config, &def.stat_overrides);
}

/// Stamps init-time behavior components and builds `ActiveChains`.
///
/// Runs `OnEnter(GameState::Playing)` AFTER `init_breaker_params`.
/// - Inserts `LivesCount` if archetype has `life_pool`
/// - Builds `ActiveChains` from root fields and `chains`
pub(crate) fn init_archetype(
    mut commands: Commands,
    selected: Res<SelectedArchetype>,
    registry: Res<ArchetypeRegistry>,
    breaker_query: Query<Entity, (With<Breaker>, Without<LivesCount>)>,
    mut active: ResMut<ActiveChains>,
) {
    let Some(def) = registry.get(&selected.0) else {
        warn!("Archetype '{}' not found in registry", selected.0);
        return;
    };

    // Stamp init-time components on breaker entity
    for entity in &breaker_query {
        if let Some(life_pool) = def.life_pool {
            commands.entity(entity).insert(LivesCount(life_pool));
        }
    }

    // Build ActiveChains from root fields + chains
    let mut chains = Vec::new();
    if let Some(chain) = &def.on_bolt_lost {
        chains.push((None, TriggerChain::OnBoltLost(vec![chain.clone()])));
    }
    if let Some(chain) = &def.on_perfect_bump {
        chains.push((None, TriggerChain::OnPerfectBump(vec![chain.clone()])));
    }
    if let Some(chain) = &def.on_early_bump {
        chains.push((None, TriggerChain::OnEarlyBump(vec![chain.clone()])));
    }
    if let Some(chain) = &def.on_late_bump {
        chains.push((None, TriggerChain::OnLateBump(vec![chain.clone()])));
    }
    chains.extend(def.chains.iter().cloned().map(|c| (None, c)));
    *active = ActiveChains(chains);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        behaviors::definition::{ArchetypeDefinition, BreakerStatOverrides},
        chips::definition::{ImpactTarget, Target},
    };

    const TEST_ARCHETYPE_NAME: &str = "TestArchetype";

    fn make_test_archetype() -> ArchetypeDefinition {
        ArchetypeDefinition {
            name: TEST_ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            on_bolt_lost: Some(TriggerChain::LoseLife),
            on_perfect_bump: Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.5,
            }),
            on_early_bump: Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.1,
            }),
            on_late_bump: Some(TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.1,
            }),
            chains: vec![],
        }
    }

    fn test_app_with_archetype(def: ArchetypeDefinition) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut registry = ArchetypeRegistry::default();
        registry.insert(def.name.clone(), def);
        app.insert_resource(registry)
            .insert_resource(SelectedArchetype(TEST_ARCHETYPE_NAME.to_owned()))
            .init_resource::<ActiveChains>()
            .add_systems(Update, init_archetype);
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
    fn init_archetype_builds_active_chains() {
        let mut app = test_app_with_archetype(make_test_archetype());
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveChains>();
        // on_bolt_lost=LoseLife → OnBoltLost(LoseLife)
        // on_perfect_bump=SpeedBoost → OnPerfectBump(SpeedBoost{...})
        // on_early_bump=SpeedBoost → OnEarlyBump(SpeedBoost{...})
        // on_late_bump=SpeedBoost → OnLateBump(SpeedBoost{...})
        assert_eq!(active.0.len(), 4);
        assert!(matches!(
            &active.0[0],
            (None, TriggerChain::OnBoltLost(effects)) if effects.len() == 1 && matches!(effects[0], TriggerChain::LoseLife)
        ));
    }

    #[test]
    fn init_archetype_builds_active_chains_with_non_speed_boost() {
        let def = ArchetypeDefinition {
            name: TEST_ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: Some(TriggerChain::TimePenalty { seconds: 5.0 }),
            on_perfect_bump: Some(TriggerChain::SpawnBolt),
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };
        let mut app = test_app_with_archetype(def);
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveChains>();
        assert_eq!(active.0.len(), 2);
    }

    #[test]
    fn init_archetype_includes_chains_field() {
        let def = ArchetypeDefinition {
            name: TEST_ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::test_shockwave(64.0)],
            )])],
        };
        let mut app = test_app_with_archetype(def);
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveChains>();
        assert_eq!(active.0.len(), 1);
    }

    #[test]
    fn init_archetype_skips_already_initialized() {
        let mut app = test_app_with_archetype(make_test_archetype());
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
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<BreakerDefaults>()
            .init_resource::<BreakerConfig>();

        let def = ArchetypeDefinition {
            name: "Wide".to_owned(),
            stat_overrides: BreakerStatOverrides {
                width: Some(200.0),
                ..default()
            },
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };

        let mut registry = ArchetypeRegistry::default();
        registry.insert("Wide".to_owned(), def);
        app.insert_resource(registry)
            .insert_resource(SelectedArchetype("Wide".to_owned()))
            .add_systems(Update, apply_archetype_config_overrides);
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!((config.width - 200.0).abs() < f32::EPSILON);
        let default_config = BreakerConfig::default();
        assert!((config.max_speed - default_config.max_speed).abs() < f32::EPSILON);
    }

    #[test]
    fn no_life_pool_no_lives_count() {
        let def = ArchetypeDefinition {
            name: TEST_ARCHETYPE_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };

        let mut app = test_app_with_archetype(def);
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();

        assert!(app.world().get::<LivesCount>(entity).is_none());
    }
}
