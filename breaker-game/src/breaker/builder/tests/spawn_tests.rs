use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::helpers::test_breaker_definition;
use crate::{
    breaker::components::{
        BoltLossBehavior, BreakerBaseY, DashState, PreviousDashState, PrimaryBreaker,
    },
    effect_v3::{
        effects::LoseLifeConfig,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

// ── Behavior 40: spawn() creates entity with all build() components ──

#[test]
fn spawn_creates_entity_with_build_components() {
    let def = test_breaker_definition();
    let mut app = test_app();

    // Use a system to get access to Commands
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    // Since we can't easily get the entity from inside the closure,
    // query for it instead
    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1, "should have spawned exactly 1 entity");
    let entity = entities[0];

    assert!(
        app.world().get::<Breaker>(entity).is_some(),
        "should have Breaker"
    );
    // Guard: check non-#[require] components
    assert!(
        app.world().get::<PrimaryBreaker>(entity).is_some(),
        "should have PrimaryBreaker"
    );
    assert!(
        app.world().get::<MaxSpeed>(entity).is_some(),
        "should have MaxSpeed"
    );
    assert!(
        app.world().get::<BreakerBaseY>(entity).is_some(),
        "should have BreakerBaseY"
    );
}

// ── Behavior 41: spawn() calls dispatch_initial_effects when effects present ──

#[test]
fn spawn_dispatches_effects_when_present() {
    let mut def = test_breaker_definition();
    def.effects = vec![RootNode::Stamp(
        StampTarget::Breaker,
        Tree::When(
            Trigger::BoltLostOccurred,
            Box::new(Tree::Fire(EffectType::LoseLife(LoseLifeConfig {}))),
        ),
    )];

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    assert!(
        bound.is_some(),
        "entity should have BoundEffects after spawn() with effects"
    );
    assert!(
        !bound.unwrap().0.is_empty(),
        "BoundEffects should contain the When/Fire chain"
    );
}

// ── Behavior 42: spawn() does NOT dispatch effects when empty ──

#[test]
fn spawn_does_not_dispatch_effects_when_empty() {
    let def = test_breaker_definition(); // effects: []

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Breaker>(entity).is_some(),
        "entity should have Breaker marker"
    );
    // Guard: check a non-#[require] component to prevent false pass from stub
    assert!(
        app.world().get::<PrimaryBreaker>(entity).is_some(),
        "entity should have PrimaryBreaker (guard against stub false pass)"
    );

    let bound = app.world().get::<BoundEffects>(entity);
    if let Some(bound) = bound {
        assert!(
            bound.0.len() <= 2,
            "BoundEffects should have at most 2 entries (salvo_hit) when effects is empty, got {}",
            bound.0.len()
        );
    }
}

// ── Behavior 43: spawn() passes source: None ──

#[test]
fn spawn_passes_source_none_to_dispatch() {
    let mut def = test_breaker_definition();
    def.effects = vec![RootNode::Stamp(
        StampTarget::Breaker,
        Tree::When(
            Trigger::BoltLostOccurred,
            Box::new(Tree::Fire(EffectType::LoseLife(LoseLifeConfig {}))),
        ),
    )];

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    assert!(bound.is_some(), "should have BoundEffects");
    for (chip_name, _) in &bound.unwrap().0 {
        assert_eq!(
            chip_name, "",
            "chip name should be empty string (source: None)"
        );
    }
}

// ── Wave 3 Behavior 18: spawned breaker carries BoltLossBehavior matching its definition ──

#[test]
fn spawn_inserts_bolt_loss_behavior_time_loss_seven_point_five() {
    let mut def = test_breaker_definition();
    def.bolt_loss_behavior = BoltLossBehavior::TimeLoss(7.5);

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&BoltLossBehavior, With<Breaker>>();
    let behaviors: Vec<BoltLossBehavior> = query.iter(app.world()).copied().collect();
    assert_eq!(
        behaviors.len(),
        1,
        "expected exactly one Breaker with BoltLossBehavior, got {}",
        behaviors.len(),
    );
    assert_eq!(
        behaviors[0],
        BoltLossBehavior::TimeLoss(7.5),
        "spawned breaker should have BoltLossBehavior::TimeLoss(7.5), got {:?}",
        behaviors[0],
    );
}

#[test]
fn spawn_inserts_bolt_loss_behavior_none() {
    let mut def = test_breaker_definition();
    def.bolt_loss_behavior = BoltLossBehavior::None;

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&BoltLossBehavior, With<Breaker>>();
    let behaviors: Vec<BoltLossBehavior> = query.iter(app.world()).copied().collect();
    assert_eq!(behaviors.len(), 1, "expected exactly one BoltLossBehavior");
    assert_eq!(behaviors[0], BoltLossBehavior::None);
}

#[test]
fn spawn_inserts_bolt_loss_behavior_default_life_loss_one() {
    // test_breaker_definition() uses BreakerDefinition::default() values, so
    // bolt_loss_behavior defaults to LifeLoss(1).
    let def = test_breaker_definition();

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<&BoltLossBehavior, With<Breaker>>();
    let behaviors: Vec<BoltLossBehavior> = query.iter(app.world()).copied().collect();
    assert_eq!(behaviors.len(), 1, "expected exactly one BoltLossBehavior");
    assert_eq!(
        behaviors[0],
        BoltLossBehavior::LifeLoss(1),
        "default BreakerDefinition should produce LifeLoss(1), got {:?}",
        behaviors[0],
    );
}

// ── Wave 3 Behavior 18 (variant): spawn() propagates non-default BoltLossBehavior ──

#[test]
fn spawn_propagates_bolt_loss_behavior_from_definition() {
    let mut def = test_breaker_definition();
    def.bolt_loss_behavior = BoltLossBehavior::TimeLoss(5.0);

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1, "should have spawned exactly 1 entity");
    let entity = entities[0];

    let behavior = app
        .world()
        .get::<BoltLossBehavior>(entity)
        .expect("spawned entity should have BoltLossBehavior component");
    assert_eq!(
        *behavior,
        BoltLossBehavior::TimeLoss(5.0),
        "spawned breaker should propagate BoltLossBehavior::TimeLoss(5.0) from its definition, got {:?}",
        *behavior,
    );
}

// ── Wave 4 Behaviors 10, 11: spawn() inserts PreviousDashState::default() ──

#[test]
fn spawn_inserts_previous_dash_state_default_idle_primary() {
    let def = test_breaker_definition();

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1, "should have spawned exactly 1 entity");
    let entity = entities[0];

    let previous = app
        .world()
        .get::<PreviousDashState>(entity)
        .expect("spawned primary breaker should have PreviousDashState component");
    assert_eq!(
        previous.0,
        DashState::Idle,
        "spawned primary breaker should have PreviousDashState(Idle) (== PreviousDashState::default()), got {:?}",
        previous.0,
    );

    // Edge case: DashState::default() and PreviousDashState::default() agree
    // at spawn — first FixedUpdate tick will report no transition (matches
    // Behavior 8).
    let state = app
        .world()
        .get::<DashState>(entity)
        .expect("spawned primary breaker should have DashState component");
    assert_eq!(
        *state,
        DashState::Idle,
        "spawned primary breaker should have DashState::Idle so first-tick window reports no transition",
    );
}

#[test]
fn spawn_inserts_previous_dash_state_default_idle_extra() {
    let def = test_breaker_definition();

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .extra()
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1, "should have spawned exactly 1 entity");
    let entity = entities[0];

    let previous = app
        .world()
        .get::<PreviousDashState>(entity)
        .expect("spawned extra breaker should have PreviousDashState component");
    assert_eq!(
        previous.0,
        DashState::Idle,
        "spawned extra breaker should have PreviousDashState(Idle) (== PreviousDashState::default()), got {:?}",
        previous.0,
    );

    // Edge case: DashState::default() and PreviousDashState::default() agree
    // at spawn.
    let state = app
        .world()
        .get::<DashState>(entity)
        .expect("spawned extra breaker should have DashState component");
    assert_eq!(
        *state,
        DashState::Idle,
        "spawned extra breaker should have DashState::Idle so first-tick window reports no transition",
    );
}
