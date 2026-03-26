//! Breaker initialization re-exports (canonical location: `breaker/systems/init_breaker.rs`).

// Re-export production functions (canonical location: breaker/systems/init_breaker.rs)
// Re-exports for test module compatibility (tests use `super::*`)
#[cfg(test)]
pub(crate) use bevy::prelude::*;

pub(crate) use crate::breaker::systems::init_breaker::{
    apply_breaker_config_overrides, apply_stat_overrides, init_breaker,
};
#[cfg(test)]
pub(crate) use crate::{
    breaker::{
        components::Breaker,
        registry::BreakerRegistry,
        resources::{BreakerConfig, BreakerDefaults},
    },
    effect::{active::ActiveEffects, effects::life_lost::LivesCount},
    shared::SelectedBreaker,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{
        BreakerDefinition, BreakerStatOverrides, Effect, EffectNode, ImpactTarget, Target, Trigger,
    };

    const TEST_BREAKER_NAME: &str = "TestBreaker";

    fn make_test_breaker() -> BreakerDefinition {
        BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            on_bolt_lost: Some(EffectNode::Do(Effect::LoseLife)),
            on_perfect_bump: Some(EffectNode::Do(Effect::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.5,
            })),
            on_early_bump: Some(EffectNode::Do(Effect::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.1,
            })),
            on_late_bump: Some(EffectNode::Do(Effect::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.1,
            })),
            chains: vec![],
        }
    }

    fn test_app_with_breaker(def: BreakerDefinition) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let mut registry = BreakerRegistry::default();
        registry.insert(def.name.clone(), def);
        app.insert_resource(registry)
            .insert_resource(SelectedBreaker(TEST_BREAKER_NAME.to_owned()))
            .init_resource::<ActiveEffects>()
            .add_systems(Update, init_breaker);
        app
    }

    #[test]
    fn init_breaker_stamps_lives_count() {
        let mut app = test_app_with_breaker(make_test_breaker());
        let entity = app.world_mut().spawn(Breaker).id();
        app.update();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 3);
    }

    #[test]
    fn init_breaker_builds_active_chains() {
        let mut app = test_app_with_breaker(make_test_breaker());
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveEffects>();
        // on_bolt_lost=Do(LoseLife) → When { BoltLost, [Do(LoseLife)] }
        // on_perfect_bump=Do(SpeedBoost) → When { PerfectBump, [Do(SpeedBoost)] }
        // on_early_bump=Do(SpeedBoost) → When { EarlyBump, [Do(SpeedBoost)] }
        // on_late_bump=Do(SpeedBoost) → When { LateBump, [Do(SpeedBoost)] }
        assert_eq!(active.0.len(), 4);
        assert!(matches!(
            &active.0[0],
            (None, EffectNode::When { trigger: Trigger::BoltLost, then }) if then.len() == 1 && matches!(then[0], EffectNode::Do(Effect::LoseLife))
        ));
    }

    #[test]
    fn init_breaker_builds_active_chains_with_non_speed_boost() {
        let def = BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: Some(EffectNode::Do(Effect::TimePenalty { seconds: 5.0 })),
            on_perfect_bump: Some(EffectNode::Do(Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            })),
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };
        let mut app = test_app_with_breaker(def);
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveEffects>();
        assert_eq!(active.0.len(), 2);
    }

    #[test]
    fn init_breaker_includes_chains_field() {
        let def = BreakerDefinition {
            name: TEST_BREAKER_NAME.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }],
        };
        let mut app = test_app_with_breaker(def);
        app.world_mut().spawn(Breaker);
        app.update();

        let active = app.world().resource::<ActiveEffects>();
        assert_eq!(active.0.len(), 1);
    }

    #[test]
    fn init_breaker_skips_already_initialized() {
        let mut app = test_app_with_breaker(make_test_breaker());
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

        let def = BreakerDefinition {
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
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: vec![],
        };

        let mut app = test_app_with_breaker(def);
        let entity = app.world_mut().spawn(Breaker).id();
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
            then: vec![EffectNode::Do(Effect::SpeedBoost {
                target: crate::effect::definition::Target::Bolt,
                multiplier: 1.5,
            })],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::SpeedBoost {
                target: crate::effect::definition::Target::Bolt,
                multiplier: 1.5,
            })],
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
