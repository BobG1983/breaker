use bevy::prelude::*;

use super::*;
use crate::{
    breaker::{
        SelectedBreaker,
        components::{Breaker, BreakerInitialized},
        definition::BreakerDefinition,
        registry::BreakerRegistry,
    },
    effect::{effects::life_lost::LivesCount, *},
};

const TEST_BREAKER_NAME: &str = "TestBreaker";

fn make_test_breaker() -> BreakerDefinition {
    ron::de::from_str(&format!(
        r#"(name: "{TEST_BREAKER_NAME}", life_pool: Some(3), effects: [
            On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(LoseLife)])]),
            On(target: Breaker, then: [When(trigger: PerfectBump, then: [Do(SpeedBoost(multiplier: 1.5))])]),
            On(target: Breaker, then: [When(trigger: EarlyBump, then: [Do(SpeedBoost(multiplier: 1.1))])]),
            On(target: Breaker, then: [When(trigger: LateBump, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        ])"#
    ))
    .expect("test RON should parse")
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
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(lives.0, Some(3));
}

#[test]
fn init_breaker_stamps_breaker_initialized_marker() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    assert!(
        app.world().get::<BreakerInitialized>(entity).is_some(),
        "init_breaker should insert BreakerInitialized marker component"
    );
}

#[test]
fn init_breaker_does_not_push_effects() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    // init_breaker should NOT push effects to BoundEffects -- that is
    // dispatch_breaker_effects' responsibility
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );
}

#[test]
fn init_breaker_does_not_push_effects_mixed_targets() {
    let def: BreakerDefinition = ron::de::from_str(&format!(
        r#"(name: "{TEST_BREAKER_NAME}", life_pool: None, effects: [
            On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(LoseLife)])]),
            On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
            On(target: AllCells, then: [When(trigger: Impacted(Bolt), then: [Do(Shockwave(base_range: 32.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])]),
        ])"#
    ))
    .expect("test RON should parse");
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects (including Breaker-targeted) to BoundEffects"
    );
}

#[test]
fn init_breaker_does_not_push_effects_chrono_style() {
    let def: BreakerDefinition = ron::de::from_str(&format!(
        r#"(name: "{TEST_BREAKER_NAME}", life_pool: None, effects: [
            On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(TimePenalty(seconds: 5.0))])]),
            On(target: Breaker, then: [When(trigger: PerfectBump, then: [Do(SpawnBolts())])]),
        ])"#
    ))
    .expect("test RON should parse");
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );
}

#[test]
fn init_breaker_does_not_push_nested_chains() {
    let def: BreakerDefinition = ron::de::from_str(&format!(
        r#"(name: "{TEST_BREAKER_NAME}", life_pool: None, effects: [
            On(target: Breaker, then: [When(trigger: PerfectBump, then: [
                When(trigger: Impact(Cell), then: [Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])
            ])]),
        ])"#
    ))
    .expect("test RON should parse");
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );
}

#[test]
fn init_breaker_skips_already_initialized() {
    let mut app = test_app_with_breaker(make_test_breaker());
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BreakerInitialized,
            LivesCount(Some(99)),
            BoundEffects::default(),
        ))
        .id();
    app.update();

    let lives = app.world().get::<LivesCount>(entity).unwrap();
    assert_eq!(
        lives.0,
        Some(99),
        "should not overwrite existing LivesCount"
    );
}

#[test]
fn life_pool_none_stamps_infinite_lives() {
    let def: BreakerDefinition = ron::de::from_str(&format!(
        r#"(name: "{TEST_BREAKER_NAME}", life_pool: None, effects: [])"#
    ))
    .expect("test RON should parse");

    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();
    app.update();

    let lives = app
        .world()
        .get::<LivesCount>(entity)
        .expect("breaker with life_pool: None should still have LivesCount component");
    assert_eq!(
        lives.0, None,
        "breaker with life_pool: None should have LivesCount(None) for infinite lives"
    );
}

#[test]
fn init_breaker_no_duplicate_init_on_reentry() {
    let def: BreakerDefinition = ron::de::from_str(&format!(
        r#"(name: "{TEST_BREAKER_NAME}", life_pool: None, effects: [
            On(target: Breaker, then: [When(trigger: PerfectBump, then: [Do(SpeedBoost(multiplier: 1.5))])]),
        ])"#
    ))
    .expect("test RON should parse");
    let mut app = test_app_with_breaker(def);
    let entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default()))
        .id();

    // First invocation — init_breaker no longer pushes effects
    app.update();
    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "init_breaker should not push any effects to BoundEffects"
    );

    // Second invocation — BreakerInitialized marker should prevent re-entry
    app.update();
    let chains = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        chains.0.len(),
        0,
        "second init should still leave BoundEffects empty"
    );
}
