use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::system::*;
use crate::{
    breaker::{
        SelectedBreaker,
        components::{Breaker, BreakerBaseY, BreakerReflectionSpread, DashTilt},
        definition::BreakerDefinition,
        registry::BreakerRegistry,
    },
    effect::{
        BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger,
        effects::life_lost::LivesCount,
    },
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BreakerRegistry>()
        .init_resource::<SelectedBreaker>()
        .add_systems(Update, propagate_breaker_changes);
    app
}

fn make_test_def(name: &str, life_pool: Option<u32>) -> BreakerDefinition {
    ron::de::from_str(&format!(
        r#"(name: "{name}", life_pool: {lp}, effects: [])"#,
        lp = life_pool.map_or_else(|| "None".to_string(), |n| format!("Some({n})")),
    ))
    .expect("test RON should parse")
}

#[test]
fn registry_rebuilt_on_modified() {
    let mut app = test_app();

    let def = make_test_def("Test", Some(3));

    // Seed registry with initial definition
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    // Flush Added change detection
    app.update();
    app.update();

    // Mutate registry directly — simulates propagate_registry rebuild
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let updated = make_test_def("Test", Some(5));
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();

    let registry = app.world().resource::<BreakerRegistry>();
    let rebuilt = registry.get("Test").unwrap();
    assert_eq!(rebuilt.life_pool, Some(5));
}

// ── Behavior 13: Hot-reload stamps components from BreakerDefinition ───

#[test]
fn hot_reload_stamps_max_speed_from_definition() {
    let mut app = test_app();

    let def = make_test_def("Test", Some(3));

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    // Spawn breaker with old MaxSpeed
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert(BoundEffects::default());

    // Flush Added
    app.update();
    app.update();

    // Modify registry with new max_speed=800.0
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", Some(3));
        updated.max_speed = 800.0;
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();

    let max_speed = app.world().get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max_speed.0 - 800.0).abs() < f32::EPSILON,
        "MaxSpeed should be updated to 800.0 from definition, got {}",
        max_speed.0
    );
}

// ── Behavior 14: Angle components in radians from definition ───────────

#[test]
fn hot_reload_updates_reflection_spread_in_radians() {
    let mut app = test_app();

    let def = make_test_def("Test", None);

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert((
        BreakerReflectionSpread(999.0),
        DashTilt(999.0),
        BoundEffects::default(),
    ));

    // Flush Added
    app.update();
    app.update();

    // Modify registry: reflection_spread=60.0, dash_tilt_angle=20.0
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", None);
        updated.reflection_spread = 60.0;
        updated.dash_tilt_angle = 20.0;
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();

    let spread = app.world().get::<BreakerReflectionSpread>(entity).unwrap();
    assert!(
        (spread.0 - 60.0_f32.to_radians()).abs() < 1e-5,
        "BreakerReflectionSpread should be 60 degrees in radians, got {}",
        spread.0
    );

    let tilt = app.world().get::<DashTilt>(entity).unwrap();
    assert!(
        (tilt.0 - 20.0_f32.to_radians()).abs() < 1e-5,
        "DashTilt should be 20 degrees in radians, got {}",
        tilt.0
    );
}

// ── Behavior 15: LivesCount from definition ────────────────────────────

#[test]
fn lives_count_reset_on_breaker_change() {
    let mut app = test_app();

    let def = make_test_def("Test", Some(3));

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    // Spawn breaker with 1 life remaining (took damage)
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert((LivesCount(Some(1)), BoundEffects::default()));

    // Flush Added
    app.update();
    app.update();

    // Modify breaker to 5 lives
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let updated = make_test_def("Test", Some(5));
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();
    // Need extra update for commands to flush (insert LivesCount)
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(
        lives.0,
        Some(5),
        "LivesCount should be reset to new life_pool value"
    );
}

#[test]
fn lives_count_reset_to_none_on_breaker_change() {
    let mut app = test_app();

    let def = make_test_def("Test", Some(3));

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    // Spawn breaker with 2 finite lives
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert((LivesCount(Some(2)), BoundEffects::default()));

    // Flush Added
    app.update();
    app.update();

    // Modify breaker to infinite lives (life_pool: None)
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let updated = make_test_def("Test", None);
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();
    // Need extra update for commands to flush (insert LivesCount)
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(
        lives.0, None,
        "LivesCount should be reset to None (infinite lives) when life_pool changes to None"
    );
}

// ── Behavior 16: BoundEffects rebuilt from definition ──────────────────

#[test]
fn active_chains_rebuilt_on_breaker_change() {
    let mut app = test_app();

    let mut def = make_test_def("Test", None);
    def.effects = vec![RootEffect::On {
        target: Target::Breaker,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    let def = BreakerDefinition::default();
    let breaker_entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert(BoundEffects::default());

    // Flush Added
    app.update();
    app.update();

    // Modify: rebuild with 4 effects
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", None);
        updated.effects = vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ];
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();

    let chains = app.world().get::<BoundEffects>(breaker_entity).unwrap();
    assert_eq!(
        chains.0.len(),
        4,
        "should have 4 chains on breaker entity (all included), got {}",
        chains.0.len()
    );
}

// ── Behavior 18: Registry is_added guard ───────────────────────────────

#[test]
fn registry_is_added_guard_prevents_stamping() {
    let mut app = test_app();

    // Spawn breaker with MaxSpeed(500.0)
    let def = make_test_def("Test", Some(3));
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert(BoundEffects::default());

    // Insert registry for the first time — this is an "add", not a "change"
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    // First update: registry is_added() returns true — should NOT stamp
    app.update();

    let max_speed = app.world().get::<MaxSpeed>(entity).unwrap();
    assert!(
        (max_speed.0 - 500.0).abs() < f32::EPSILON,
        "MaxSpeed should remain 500.0 on initial registry add, got {}",
        max_speed.0
    );
}

// ── Behavior 19: BreakerBaseY updated from definition ──────────────────

#[test]
fn hot_reload_updates_breaker_base_y_from_definition() {
    let mut app = test_app();

    let def = make_test_def("Test", None);

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert(BoundEffects::default());

    // Flush Added
    app.update();
    app.update();

    // Modify: y_position=-300.0
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", None);
        updated.y_position = -300.0;
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();

    let base_y = app.world().get::<BreakerBaseY>(entity).unwrap();
    assert!(
        (base_y.0 - (-300.0)).abs() < f32::EPSILON,
        "BreakerBaseY should be updated to -300.0, got {}",
        base_y.0
    );
}

// ── Retained existing tests ────────────────────────────────────────────

#[test]
fn lives_count_inserted_on_entity_without_prior_lives_count() {
    let mut app = test_app();

    let def = make_test_def("Test", None);

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    // Spawn breaker WITHOUT LivesCount component
    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(entity)
        .insert(BoundEffects::default());

    // Flush Added
    app.update();
    app.update();

    // Trigger hot-reload by modifying registry
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let updated = make_test_def("Test", None);
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();
    // Need extra update for commands to flush
    app.update();

    let lives = app
        .world()
        .get::<LivesCount>(entity)
        .expect("LivesCount should be inserted even when entity never had it");
    assert_eq!(
        lives.0, None,
        "LivesCount should be None (infinite) when life_pool is None and component was absent"
    );
}

#[test]
fn speed_boost_chains_appear_in_effect_chains_on_breaker_change() {
    let mut app = test_app();

    let mut def = make_test_def("Test", None);
    def.effects = vec![RootEffect::On {
        target: Target::Breaker,
        then: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];

    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        registry.insert(def.name.clone(), def);
    }

    app.world_mut()
        .insert_resource(SelectedBreaker("Test".to_owned()));

    let def = BreakerDefinition::default();
    let breaker_entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert(BoundEffects::default());

    // Flush Added
    app.update();
    app.update();

    // Modify multiplier
    {
        let mut registry = app.world_mut().resource_mut::<BreakerRegistry>();
        let mut updated = make_test_def("Test", None);
        updated.effects = vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
            }],
        }];
        registry.clear();
        registry.insert(updated.name.clone(), updated);
    }

    app.update();

    let chains = app.world().get::<BoundEffects>(breaker_entity).unwrap();
    assert_eq!(
        chains.0.len(),
        1,
        "should have 1 chain for SpeedBoost on breaker entity, got {}",
        chains.0.len()
    );
    assert!(matches!(
        &chains.0[0],
        (_, EffectNode::When { trigger: Trigger::PerfectBump, then }) if then.len() == 1 && matches!(
            &then[0],
            EffectNode::Do(EffectKind::SpeedBoost { multiplier, .. }) if (*multiplier - 2.0).abs() < f32::EPSILON
        )
    ));
}
