use bevy::prelude::*;
use rantzsoft_spatial2d::{
    components::{InterpolateTransform2D, Position2D, Scale2D, Spatial2D, Velocity2D},
    draw_layer::DrawLayer,
};

use super::*;
use crate::{
    bolt::{components::*, messages::SpawnAdditionalBolt, resources::BoltConfig},
    breaker::components::Breaker,
    shared::{CleanupOnNodeExit, GameDrawLayer, GameRng},
};

/// Spawn a breaker entity with `Position2D` at the given position.
fn spawn_breaker_at(app: &mut App, x: f32, y: f32) {
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(x, y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));
}

#[derive(Resource)]
struct SendSpawn(u32);

fn send_spawn(flag: Res<SendSpawn>, mut writer: MessageWriter<SpawnAdditionalBolt>) {
    for _ in 0..flag.0 {
        writer.write(SpawnAdditionalBolt {
            source_chip: None,
            lifespan: None,
            inherit: false,
        });
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BoltConfig>()
        .init_resource::<GameRng>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<SpawnAdditionalBolt>()
        .insert_resource(SendSpawn(0))
        .add_systems(FixedUpdate, (send_spawn, spawn_additional_bolt).chain());
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn spawn_breaker(app: &mut App) {
    spawn_breaker_at(app, 0.0, -250.0);
}

#[test]
fn additional_bolt_has_position2d_above_breaker() {
    // Given: breaker at (42.0, -250.0, 0.0), spawn_offset_y = 30.0
    // When: SpawnAdditionalBolt message received
    // Then: Position2D(Vec2::new(42.0, -220.0))
    let mut app = test_app();
    spawn_breaker_at(&mut app, 42.0, -250.0);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let position = app
        .world_mut()
        .query_filtered::<&Position2D, With<ExtraBolt>>()
        .iter(app.world())
        .next()
        .expect("extra bolt should have Position2D");
    let config = BoltConfig::default();
    let expected = Vec2::new(42.0, -250.0 + config.spawn_offset_y);
    assert!(
        (position.0.x - expected.x).abs() < f32::EPSILON
            && (position.0.y - expected.y).abs() < f32::EPSILON,
        "additional bolt Position2D should be {expected:?}, got {:?}",
        position.0,
    );
}

#[test]
fn additional_bolt_has_spatial2d_interpolate_and_draw_layer() {
    // When: SpawnAdditionalBolt received
    // Then: has Spatial2D, InterpolateTransform2D, GameDrawLayer::Bolt
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    let world = app.world();
    assert!(
        world.get::<Spatial2D>(entity).is_some(),
        "extra bolt should have Spatial2D"
    );
    assert!(
        world.get::<InterpolateTransform2D>(entity).is_some(),
        "extra bolt should have InterpolateTransform2D"
    );
    let layer = world
        .get::<GameDrawLayer>(entity)
        .expect("extra bolt should have GameDrawLayer");
    assert!(
        (layer.z() - 1.0).abs() < f32::EPSILON,
        "GameDrawLayer::Bolt.z() should be 1.0, got {}",
        layer.z(),
    );
}

#[test]
fn additional_bolt_has_scale2d_matching_radius() {
    // Given: BoltConfig with radius = 6.0
    // When: SpawnAdditionalBolt received
    // Then: Scale2D { x: 6.0, y: 6.0 }
    let mut app = test_app();
    app.world_mut().resource_mut::<BoltConfig>().radius = 6.0;
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let scale = app
        .world_mut()
        .query_filtered::<&Scale2D, With<ExtraBolt>>()
        .iter(app.world())
        .next()
        .expect("extra bolt should have Scale2D");
    assert!(
        (scale.x - 6.0).abs() < f32::EPSILON && (scale.y - 6.0).abs() < f32::EPSILON,
        "Scale2D should be (6.0, 6.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// --- Preserved behavioral tests that still apply ---

#[test]
fn creates_new_bolt_entity() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut()
        .spawn((Bolt, Velocity2D(Vec2::new(0.0, 400.0))));
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 2, "should have baseline + 1 additional bolt");
}

#[test]
fn new_bolt_has_extra_bolt_marker() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(extra_count, 1);
}

#[test]
fn new_bolt_has_all_config_components() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    let world = app.world();
    assert!(world.get::<Velocity2D>(entity).is_some());
    assert!(world.get::<BoltBaseSpeed>(entity).is_some());
    assert!(world.get::<BoltMinSpeed>(entity).is_some());
    assert!(world.get::<BoltMaxSpeed>(entity).is_some());
    assert!(world.get::<BoltRadius>(entity).is_some());
    assert!(world.get::<BoltRespawnOffsetY>(entity).is_some());
    assert!(world.get::<BoltRespawnAngleSpread>(entity).is_some());
    assert!(world.get::<CleanupOnNodeExit>(entity).is_some());
}

#[test]
fn new_bolt_launches_upward() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<ExtraBolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "additional bolt should launch upward");
}

#[test]
fn new_bolt_at_base_speed() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let config = app.world().resource::<BoltConfig>();
    let base_speed = config.base_speed;

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<ExtraBolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.speed() - base_speed).abs() < 1.0,
        "speed {:.1} should equal base_speed {base_speed:.1}",
        vel.speed()
    );
}

#[test]
fn no_message_no_spawn() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    tick(&mut app);

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0, "no bolt should spawn without message");
}

#[test]
fn multiple_messages_spawn_multiple() {
    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 2;
    tick(&mut app);

    let count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(count, 2, "2 messages should spawn 2 extra bolts");
}

#[test]
fn spawned_bolt_inherits_entity_scale_from_active_node_layout() {
    use crate::{
        run::node::{ActiveNodeLayout, NodeLayout, definition::NodePool},
        shared::EntityScale,
    };

    let mut app = test_app();
    app.insert_resource(ActiveNodeLayout(NodeLayout {
        name: "test".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!['S', 'S']],
        pool: NodePool::default(),
        entity_scale: 0.7,
    }));
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    let scale = app.world().get::<EntityScale>(entity).unwrap();
    assert!(
        (scale.0 - 0.7).abs() < f32::EPSILON,
        "EntityScale should be 0.7 from ActiveNodeLayout, got {}",
        scale.0,
    );
}

#[test]
fn spawned_bolt_defaults_entity_scale_without_active_node_layout() {
    use crate::shared::EntityScale;

    let mut app = test_app();
    spawn_breaker(&mut app);
    app.world_mut().resource_mut::<SendSpawn>().0 = 1;
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    let scale = app.world().get::<EntityScale>(entity).unwrap();
    assert!(
        (scale.0 - 1.0).abs() < f32::EPSILON,
        "EntityScale should default to 1.0, got {}",
        scale.0,
    );
}

// -- SpawnedByEvolution attribution tests --

/// Sends specific `SpawnAdditionalBolt` messages with custom `source_chip` values.
#[derive(Resource, Default)]
struct SendSpawnWithAttribution(Vec<SpawnAdditionalBolt>);

fn send_spawn_with_attribution(
    mut flag: ResMut<SendSpawnWithAttribution>,
    mut writer: MessageWriter<SpawnAdditionalBolt>,
) {
    for msg in flag.0.drain(..) {
        writer.write(msg);
    }
}

/// Test app variant that uses `SendSpawnWithAttribution` for custom messages.
fn test_app_with_attribution() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BoltConfig>()
        .init_resource::<GameRng>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<SpawnAdditionalBolt>()
        .insert_resource(SendSpawnWithAttribution::default())
        .add_systems(
            FixedUpdate,
            (send_spawn_with_attribution, spawn_additional_bolt).chain(),
        );
    app
}

#[test]
fn additional_bolt_receives_spawned_by_evolution_when_source_chip_is_some() {
    let mut app = test_app_with_attribution();
    spawn_breaker_at(&mut app, 0.0, -250.0);
    app.world_mut()
        .resource_mut::<SendSpawnWithAttribution>()
        .0
        .push(SpawnAdditionalBolt {
            source_chip: Some("chain_lightning".to_owned()),
            lifespan: None,
            inherit: false,
        });
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    let spawned_by = app
        .world()
        .get::<SpawnedByEvolution>(entity)
        .expect("ExtraBolt should have SpawnedByEvolution when source_chip is Some");
    assert_eq!(
        spawned_by.0, "chain_lightning",
        "SpawnedByEvolution should carry the source chip name"
    );
}

#[test]
fn additional_bolt_receives_spawned_by_evolution_with_empty_string() {
    let mut app = test_app_with_attribution();
    spawn_breaker_at(&mut app, 0.0, -250.0);
    app.world_mut()
        .resource_mut::<SendSpawnWithAttribution>()
        .0
        .push(SpawnAdditionalBolt {
            source_chip: Some(String::new()),
            lifespan: None,
            inherit: false,
        });
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    let spawned_by = app
        .world()
        .get::<SpawnedByEvolution>(entity)
        .expect("ExtraBolt should have SpawnedByEvolution even with empty string source_chip");
    assert_eq!(
        spawned_by.0, "",
        "SpawnedByEvolution should carry an empty string when source_chip is empty"
    );
}

#[test]
fn additional_bolt_has_no_spawned_by_evolution_when_source_chip_is_none() {
    let mut app = test_app_with_attribution();
    spawn_breaker_at(&mut app, 0.0, -250.0);
    app.world_mut()
        .resource_mut::<SendSpawnWithAttribution>()
        .0
        .push(SpawnAdditionalBolt {
            source_chip: None,
            lifespan: None,
            inherit: false,
        });
    tick(&mut app);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .next()
        .expect("extra bolt should exist");

    assert!(
        app.world().get::<SpawnedByEvolution>(entity).is_none(),
        "ExtraBolt should NOT have SpawnedByEvolution when source_chip is None"
    );
}

#[test]
fn multiple_spawns_with_different_attributions_produce_correctly_attributed_bolts() {
    let mut app = test_app_with_attribution();
    spawn_breaker_at(&mut app, 0.0, -250.0);
    {
        let mut res = app.world_mut().resource_mut::<SendSpawnWithAttribution>();
        res.0.push(SpawnAdditionalBolt {
            source_chip: Some("alpha".to_owned()),
            lifespan: None,
            inherit: false,
        });
        res.0.push(SpawnAdditionalBolt {
            source_chip: Some("beta".to_owned()),
            lifespan: None,
            inherit: false,
        });
    }
    tick(&mut app);

    let attributions: Vec<String> = app
        .world_mut()
        .query_filtered::<&SpawnedByEvolution, With<ExtraBolt>>()
        .iter(app.world())
        .map(|s| s.0.clone())
        .collect();
    assert_eq!(
        attributions.len(),
        2,
        "should have two ExtraBolts with SpawnedByEvolution, got {}",
        attributions.len()
    );
    assert!(
        attributions.contains(&"alpha".to_owned()),
        "one bolt should be attributed to 'alpha', got {attributions:?}"
    );
    assert!(
        attributions.contains(&"beta".to_owned()),
        "one bolt should be attributed to 'beta', got {attributions:?}"
    );
}
