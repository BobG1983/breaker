use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{custom_wall_definition, default_playfield};
use crate::{
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
    shared::GameDrawLayer,
    walls::{components::Wall, definition::WallDefinition},
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

// ── Behavior 29: spawn() creates entity with all build() components ──

#[test]
fn spawn_creates_entity_with_build_components() {
    let pf = default_playfield();
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder().left(&pf).spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(
        entities.len(),
        1,
        "should have spawned exactly 1 wall entity"
    );
    let entity = entities[0];

    assert!(
        app.world().get::<Wall>(entity).is_some(),
        "should have Wall marker"
    );
    let pos = app.world().get::<Position2D>(entity);
    assert!(pos.is_some(), "should have Position2D");
    assert!(
        (pos.unwrap().0.x - (-490.0)).abs() < f32::EPSILON,
        "Position2D.x should be -490.0, got {}",
        pos.unwrap().0.x
    );
    assert!(
        matches!(
            app.world().get::<GameDrawLayer>(entity),
            Some(GameDrawLayer::Wall)
        ),
        "should have GameDrawLayer::Wall"
    );
}

#[test]
fn spawn_multiple_calls_produce_multiple_entities() {
    let pf = default_playfield();
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder().left(&pf).spawn(&mut commands);
        Wall::builder().right(&pf).spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(
        entities.len(),
        2,
        "should have spawned 2 independent wall entities"
    );
}

// ── Behavior 30: spawn() dispatches effects when effects are present ──

#[test]
fn spawn_dispatches_effects_when_definition_has_effects() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // effects: [On { target: Wall, ... }]
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .definition(&def)
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
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
        "BoundEffects should contain the effect chain"
    );
}

// ── Behavior 31: spawn() does NOT dispatch effects when effects are empty ──

#[test]
fn spawn_does_not_dispatch_effects_when_no_definition() {
    let pf = default_playfield();
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder().left(&pf).spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Wall>(entity).is_some(),
        "entity should have Wall marker"
    );

    let bound = app.world().get::<BoundEffects>(entity);
    if let Some(bound) = bound {
        assert!(
            bound.0.is_empty(),
            "BoundEffects should be empty when no effects"
        );
    }
}

#[test]
fn spawn_does_not_dispatch_effects_when_definition_has_empty_effects() {
    let pf = default_playfield();
    let def = WallDefinition::default(); // effects: []
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .definition(&def)
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    if let Some(bound) = bound {
        assert!(
            bound.0.is_empty(),
            "BoundEffects should be empty when definition effects are []"
        );
    }
}

// ── Behavior 32: spawn() passes source_chip: None to dispatch_initial_effects ──

#[test]
fn spawn_passes_source_chip_none() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // effects: [On { target: Wall, ... }]
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .definition(&def)
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    assert!(bound.is_some(), "should have BoundEffects");
    for (chip_name, _) in &bound.unwrap().0 {
        assert_eq!(
            chip_name, "",
            "chip name should be empty string (source_chip: None)"
        );
    }
}

// ── Behavior 33: spawn() dispatches override effects, not definition effects ──

#[test]
fn spawn_dispatches_override_effects_not_definition() {
    let pf = default_playfield();

    // Definition effects: SpeedBoost multiplier 1.5
    let def = custom_wall_definition();

    // Override effects: SpeedBoost multiplier 2.0
    let override_effects = vec![RootEffect::On {
        target: Target::Wall,
        then: vec![EffectNode::When {
            trigger: Trigger::Bumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
        }],
    }];

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .definition(&def)
            .with_effects(override_effects.clone())
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    assert!(
        bound.is_some(),
        "entity should have BoundEffects with override effects"
    );
    assert!(
        !bound.unwrap().0.is_empty(),
        "BoundEffects should contain the override effect chain"
    );
}

#[test]
fn spawn_override_empty_vec_means_no_effects_even_with_definition() {
    let pf = default_playfield();
    let def = custom_wall_definition(); // has effects

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .definition(&def)
            .with_effects(vec![]) // explicitly empty override
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    if let Some(bound) = bound {
        assert!(
            bound.0.is_empty(),
            "override vec![] should mean no effects, even with definition effects"
        );
    }
}

// ── Behavior 34: spawn() dispatches override effects without definition ──

#[test]
fn spawn_dispatches_override_effects_without_definition() {
    let pf = default_playfield();

    let override_effects = vec![RootEffect::On {
        target: Target::Wall,
        then: vec![EffectNode::When {
            trigger: Trigger::Bumped,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
    }];

    let mut app = test_app();
    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .with_effects(override_effects.clone())
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    assert!(
        bound.is_some(),
        "entity should have BoundEffects with override effects (no definition)"
    );
    assert!(
        !bound.unwrap().0.is_empty(),
        "BoundEffects should contain the override effect chain"
    );
}

#[test]
fn spawn_override_empty_without_definition_no_effects() {
    let pf = default_playfield();
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        Wall::builder()
            .left(&pf)
            .with_effects(vec![])
            .spawn(&mut commands);
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let bound = app.world().get::<BoundEffects>(entity);
    if let Some(bound) = bound {
        assert!(
            bound.0.is_empty(),
            "override vec![] without definition should mean no effects"
        );
    }
}

// ── Behavior 35: spawn() returns the Entity id ──

#[test]
fn spawn_returns_entity_id() {
    #[derive(Resource)]
    struct SpawnedEntity(Entity);

    let pf = default_playfield();
    let mut app = test_app();

    app.add_systems(Update, move |mut commands: Commands| {
        let entity = Wall::builder().left(&pf).spawn(&mut commands);
        commands.insert_resource(SpawnedEntity(entity));
    });
    app.update();

    let spawned = app.world().resource::<SpawnedEntity>().0;
    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    assert_eq!(
        spawned, entities[0],
        "returned Entity should match the spawned Wall entity"
    );
}
