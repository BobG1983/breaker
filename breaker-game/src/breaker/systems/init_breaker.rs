//! Breaker initialization systems — config overrides and component stamping.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    breaker::{
        components::Breaker,
        definition::BreakerStatOverrides,
        registry::BreakerRegistry,
        resources::{BreakerConfig, BreakerDefaults},
    },
    effect::{
        definition::{EffectChains, EffectNode, RootEffect, Target},
        effects::life_lost::LivesCount,
    },
    shared::SelectedBreaker,
};

/// Applies optional stat overrides to a `BreakerConfig`.
///
/// Each `Some` field in `overrides` replaces the corresponding field in `config`.
/// Used by both `apply_breaker_config_overrides` (at init) and hot-reload propagation (at runtime).
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

/// Resets `BreakerConfig` from defaults and applies breaker stat overrides.
///
/// Runs `OnEnter(GameState::Playing)` BEFORE `init_breaker_params` so that
/// stamped components reflect the overridden config values.
pub(crate) fn apply_breaker_config_overrides(
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    defaults: Res<Assets<BreakerDefaults>>,
    mut config: ResMut<BreakerConfig>,
) {
    // Reset config from loaded RON defaults (not code defaults)
    if let Some(loaded) = defaults.iter().next().map(|(_, d)| d) {
        *config = BreakerConfig::from(loaded.clone());
    }

    // Apply breaker overrides
    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };

    apply_stat_overrides(&mut config, &def.stat_overrides);
}

/// Stamps init-time behavior components and populates entity `EffectChains`.
///
/// Runs `OnEnter(GameState::Playing)` AFTER `init_breaker_params`.
/// - Inserts `LivesCount` if breaker has `life_pool`
/// - Resolves `On` targets to entity `EffectChains` for breaker and bolt entities
pub(crate) fn init_breaker(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    registry: Res<BreakerRegistry>,
    breaker_query: Query<Entity, (With<Breaker>, Without<LivesCount>)>,
    mut breaker_chains_query: Query<&mut EffectChains, With<Breaker>>,
) {
    let Some(def) = registry.get(&selected.0) else {
        warn!("Breaker '{}' not found in registry", selected.0);
        return;
    };

    // Stamp init-time components on breaker entity
    for entity in &breaker_query {
        if let Some(life_pool) = def.life_pool {
            commands.entity(entity).insert(LivesCount(life_pool));
        }
    }

    // Resolve On targets to entity EffectChains
    for root in &def.effects {
        let RootEffect::On { target, then } = root;
        match target {
            Target::Breaker => {
                for mut chains in &mut breaker_chains_query {
                    for child in then {
                        chains.0.push((None, child.clone()));
                    }
                }
            }
            // At init time, bolt/cell/wall targets are not yet available
            Target::Bolt | Target::AllBolts | Target::Cell | Target::Wall | Target::AllCells => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{breaker::definition::BreakerDefinition, effect::definition::*};

    const TEST_BREAKER_NAME: &str = "TestBreaker";

    fn make_test_breaker() -> BreakerDefinition {
        BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            effects: vec![
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(Effect::LoseLife)],
                    }],
                },
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::PerfectBump,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                    }],
                },
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::EarlyBump,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                    }],
                },
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::LateBump,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                    }],
                },
            ],
        }
    }

    fn test_app_with_breaker(def: BreakerDefinition) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut registry = BreakerRegistry::default();
        registry.insert(def.name.clone(), def);
        app.insert_resource(registry)
            .insert_resource(SelectedBreaker(TEST_BREAKER_NAME.to_owned()))
            .add_systems(Update, init_breaker);
        app
    }

    #[test]
    fn init_breaker_stamps_lives_count() {
        let mut app = test_app_with_breaker(make_test_breaker());
        let entity = app
            .world_mut()
            .spawn((Breaker, EffectChains::default()))
            .id();
        app.update();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 3);
    }

    #[test]
    fn init_breaker_builds_active_chains() {
        let mut app = test_app_with_breaker(make_test_breaker());
        let entity = app
            .world_mut()
            .spawn((Breaker, EffectChains::default()))
            .id();
        app.update();

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        // 4 On(Breaker) entries → 4 chains on breaker entity
        // When { BoltLost, [Do(LoseLife)] }
        // When { PerfectBump, [Do(SpeedBoost)] }
        // When { EarlyBump, [Do(SpeedBoost)] }
        // When { LateBump, [Do(SpeedBoost)] }
        assert_eq!(chains.0.len(), 4);
        assert!(matches!(
            &chains.0[0],
            (None, EffectNode::When { trigger: Trigger::BoltLost, then }) if then.len() == 1 && matches!(then[0], EffectNode::Do(Effect::LoseLife))
        ));
    }

    #[test]
    fn init_breaker_builds_active_chains_with_non_speed_boost() {
        let def = BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(Effect::TimePenalty { seconds: 5.0 })],
                    }],
                },
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::PerfectBump,
                        then: vec![EffectNode::Do(Effect::SpawnBolts {
                            count: 1,
                            lifespan: None,
                            inherit: false,
                        })],
                    }],
                },
            ],
        };
        let mut app = test_app_with_breaker(def);
        let entity = app
            .world_mut()
            .spawn((Breaker, EffectChains::default()))
            .id();
        app.update();

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert_eq!(chains.0.len(), 2);
    }

    #[test]
    fn init_breaker_includes_chains_field() {
        let def = BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                    }],
                }],
            }],
        };
        let mut app = test_app_with_breaker(def);
        let entity = app
            .world_mut()
            .spawn((Breaker, EffectChains::default()))
            .id();
        app.update();

        let chains = app.world().get::<EffectChains>(entity).unwrap();
        assert_eq!(chains.0.len(), 1);
    }

    #[test]
    fn init_breaker_skips_already_initialized() {
        let mut app = test_app_with_breaker(make_test_breaker());
        let entity = app
            .world_mut()
            .spawn((Breaker, LivesCount(99), EffectChains::default()))
            .id();
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

        let def = BreakerDefinition {
            name: "Wide".to_owned(),
            stat_overrides: BreakerStatOverrides {
                width: Some(200.0),
                ..default()
            },
            life_pool: None,
            effects: vec![],
        };

        let mut registry = BreakerRegistry::default();
        registry.insert("Wide".to_owned(), def);
        app.insert_resource(registry)
            .insert_resource(SelectedBreaker("Wide".to_owned()))
            .add_systems(Update, apply_breaker_config_overrides);
        app.update();

        let config = app.world().resource::<BreakerConfig>();
        assert!((config.width - 200.0).abs() < f32::EPSILON);
        let default_config = BreakerConfig::default();
        assert!((config.max_speed - default_config.max_speed).abs() < f32::EPSILON);
    }

    #[test]
    fn no_life_pool_no_lives_count() {
        let def = BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![],
        };

        let mut app = test_app_with_breaker(def);
        let entity = app
            .world_mut()
            .spawn((Breaker, EffectChains::default()))
            .id();
        app.update();

        assert!(app.world().get::<LivesCount>(entity).is_none());
    }

    // =========================================================================
    // B12b: init_breaker pass-through behavior with EffectNode (behavior 22)
    // =========================================================================

    #[test]
    fn effect_node_pass_through_bolt_lost_fires_correctly() {
        use crate::effect::evaluate::{NodeEvalResult, evaluate_node};

        // After migration, init_breaker pushes EffectNode directly
        let node = EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::LoseLife)],
        };
        let result = evaluate_node(Trigger::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::LoseLife)],
            "pass-through EffectNode should evaluate to Fire(LoseLife) on BoltLost"
        );
    }

    #[test]
    fn effect_node_pass_through_perfect_bump_fires_correctly() {
        use crate::effect::evaluate::{NodeEvalResult, evaluate_node};

        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::SpeedBoost { multiplier: 1.5 })],
            "pass-through EffectNode should evaluate to Fire(SpeedBoost) on PerfectBump"
        );
    }

    #[test]
    fn effect_node_pass_through_chains_field_evaluates_correctly() {
        use crate::effect::evaluate::{NodeEvalResult, evaluate_node};

        // chains field entries are already full EffectNode trees — verify evaluation
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(crate::effect::definition::ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(result.len(), 1);
        assert!(
            matches!(
                &result[0],
                NodeEvalResult::Arm(EffectNode::When {
                    trigger: Trigger::Impact(..),
                    ..
                })
            ),
            "nested EffectNode should arm on first trigger match"
        );
    }
}
