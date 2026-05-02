use bevy::prelude::*;
use rantzsoft_spatial2d::components::MaxSpeed;

use super::helpers::test_breaker_definition;
use crate::{
    breaker::{
        builder::core::types::BreakerPhantomParams,
        components::{
            BoltLossBehavior, BreakerBaseY, BumpEarlyWindow, BumpFeedback, BumpLateWindow,
            BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown, DashState,
            ExtraBreaker, PhantomBreaker, PreviousDashState, PrimaryBreaker,
        },
    },
    effect_v3::{
        effects::LoseLifeConfig,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
    shared::{BaseHeight, BaseWidth, GameDrawLayer, Lifespan, PhantomFlicker},
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

// ── Wave 2 Behavior 1: Rendered .phantom() spawn inserts phantom-specific components ──

fn spawn_rendered_phantom_primary_app() -> (App, Entity) {
    let def = test_breaker_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(
        Update,
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Breaker::builder()
                .definition(&def)
                .phantom(BreakerPhantomParams {
                    lifespan:          2.5,
                    phantom_color_rgb: [0.4, 0.8, 1.0],
                    flicker_frequency: 4.0,
                    flicker_min_alpha: 0.3,
                })
                .rendered(&mut meshes, &mut materials)
                .primary()
                .spawn(&mut commands);
        },
    );
    app.update();
    let mut q = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entity = q.iter(app.world()).next().expect("phantom spawned");
    (app, entity)
}

#[test]
fn phantom_rendered_primary_spawn_inserts_phantom_specific_components() {
    let (app, entity) = spawn_rendered_phantom_primary_app();

    assert!(
        app.world().get::<Breaker>(entity).is_some(),
        "should have Breaker"
    );
    assert!(
        app.world().get::<PhantomBreaker>(entity).is_some(),
        "should have PhantomBreaker"
    );
    let flicker = app
        .world()
        .get::<PhantomFlicker>(entity)
        .expect("should have PhantomFlicker");
    assert!(
        (flicker.frequency - 4.0).abs() < f32::EPSILON,
        "PhantomFlicker.frequency should be 4.0, got {}",
        flicker.frequency
    );
    assert!(
        (flicker.min_alpha - 0.3).abs() < f32::EPSILON,
        "PhantomFlicker.min_alpha should be 0.3, got {}",
        flicker.min_alpha
    );
    let lifespan = app
        .world()
        .get::<Lifespan>(entity)
        .expect("should have Lifespan");
    assert!(
        (lifespan.remaining - 2.5).abs() < f32::EPSILON,
        "Lifespan.remaining should be 2.5, got {}",
        lifespan.remaining
    );
    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "should have MeshMaterial2d<ColorMaterial>"
    );
    assert!(
        matches!(
            app.world().get::<GameDrawLayer>(entity),
            Some(GameDrawLayer::Breaker)
        ),
        "should have GameDrawLayer::Breaker"
    );
}

#[test]
fn phantom_rendered_primary_spawn_inserts_standard_builder_components() {
    let (app, entity) = spawn_rendered_phantom_primary_app();

    assert!(
        app.world().get::<BaseWidth>(entity).is_some(),
        "should have BaseWidth"
    );
    assert!(
        app.world().get::<BaseHeight>(entity).is_some(),
        "should have BaseHeight"
    );
    assert!(
        app.world().get::<BumpState>(entity).is_some(),
        "should have BumpState"
    );
    assert!(
        app.world().get::<BumpPerfectWindow>(entity).is_some(),
        "should have BumpPerfectWindow"
    );
    assert!(
        app.world().get::<BumpEarlyWindow>(entity).is_some(),
        "should have BumpEarlyWindow"
    );
    assert!(
        app.world().get::<BumpLateWindow>(entity).is_some(),
        "should have BumpLateWindow"
    );
    assert!(
        app.world().get::<BumpPerfectCooldown>(entity).is_some(),
        "should have BumpPerfectCooldown"
    );
    assert!(
        app.world().get::<BumpWeakCooldown>(entity).is_some(),
        "should have BumpWeakCooldown"
    );
    assert!(
        app.world().get::<BumpFeedback>(entity).is_some(),
        "should have BumpFeedback"
    );
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("should have CollisionLayers");
    assert_eq!(
        layers.membership, BREAKER_LAYER,
        "membership should be BREAKER_LAYER"
    );
    assert_eq!(layers.mask, BOLT_LAYER, "mask should be BOLT_LAYER");
    assert!(
        app.world().get::<PrimaryBreaker>(entity).is_some(),
        "should have PrimaryBreaker"
    );
    assert!(
        app.world().get::<CleanupOnExit<RunState>>(entity).is_some(),
        "should have CleanupOnExit<RunState>"
    );
    // Edge: negative role checks
    assert!(
        app.world().get::<ExtraBreaker>(entity).is_none(),
        "should NOT have ExtraBreaker"
    );
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(entity)
            .is_none(),
        "should NOT have CleanupOnExit<NodeState>"
    );
}

// ── Wave 2 Behavior 2: Headless .phantom() spawn omits visual and PhantomFlicker ──

#[test]
fn phantom_headless_primary_spawn_omits_visual_and_phantom_flicker_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .phantom(BreakerPhantomParams {
            lifespan:          2.5,
            phantom_color_rgb: [0.4, 0.8, 1.0],
            flicker_frequency: 4.0,
            flicker_min_alpha: 0.3,
        })
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<Breaker>(entity).is_some(),
        "should have Breaker"
    );
    assert!(
        world.get::<PhantomBreaker>(entity).is_some(),
        "should have PhantomBreaker"
    );
    let lifespan = world.get::<Lifespan>(entity).expect("should have Lifespan");
    assert!(
        (lifespan.remaining - 2.5).abs() < f32::EPSILON,
        "Lifespan.remaining should be 2.5, got {}",
        lifespan.remaining
    );
    assert!(
        world.get::<BaseWidth>(entity).is_some(),
        "should have BaseWidth"
    );
    assert!(
        world.get::<BaseHeight>(entity).is_some(),
        "should have BaseHeight"
    );
    assert!(
        world.get::<BumpState>(entity).is_some(),
        "should have BumpState"
    );
    assert!(
        world.get::<BumpPerfectWindow>(entity).is_some(),
        "should have BumpPerfectWindow"
    );
    assert!(
        world.get::<BumpEarlyWindow>(entity).is_some(),
        "should have BumpEarlyWindow"
    );
    assert!(
        world.get::<BumpLateWindow>(entity).is_some(),
        "should have BumpLateWindow"
    );
    assert!(
        world.get::<BumpPerfectCooldown>(entity).is_some(),
        "should have BumpPerfectCooldown"
    );
    assert!(
        world.get::<BumpWeakCooldown>(entity).is_some(),
        "should have BumpWeakCooldown"
    );
    assert!(
        world.get::<BumpFeedback>(entity).is_some(),
        "should have BumpFeedback"
    );
    let layers = world
        .get::<CollisionLayers>(entity)
        .expect("should have CollisionLayers");
    assert_eq!(
        layers.membership, BREAKER_LAYER,
        "membership should be BREAKER_LAYER"
    );
    assert_eq!(layers.mask, BOLT_LAYER, "mask should be BOLT_LAYER");
    assert!(
        world.get::<PrimaryBreaker>(entity).is_some(),
        "should have PrimaryBreaker"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_some(),
        "should have CleanupOnExit<RunState>"
    );
    // Edge: headless MUST NOT have visual components or PhantomFlicker
    assert!(
        world.get::<PhantomFlicker>(entity).is_none(),
        "headless should NOT have PhantomFlicker"
    );
    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "headless should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "headless should NOT have MeshMaterial2d<ColorMaterial>"
    );
    assert!(
        world.get::<GameDrawLayer>(entity).is_none(),
        "headless should NOT have GameDrawLayer"
    );
}

// ── Wave 2 Behavior 3: With<Breaker> query finds a phantom-spawned entity ──

#[test]
fn phantom_extra_spawn_is_visible_to_with_breaker_query() {
    let def = test_breaker_definition();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, move |mut commands: Commands| {
        Breaker::builder()
            .definition(&def)
            .phantom(BreakerPhantomParams {
                lifespan:          2.5,
                phantom_color_rgb: [0.4, 0.8, 1.0],
                flicker_frequency: 4.0,
                flicker_min_alpha: 0.3,
            })
            .headless()
            .extra()
            .spawn(&mut commands);
    });
    app.update();

    let mut q = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let breakers: Vec<Entity> = q.iter(app.world()).collect();
    assert_eq!(
        breakers.len(),
        1,
        "With<Breaker> query should find exactly 1 entity (the phantom)"
    );
    let entity = breakers[0];

    // Edge: phantom IS a Breaker
    assert!(
        app.world().get::<Breaker>(entity).is_some(),
        "phantom entity should have Breaker marker"
    );
    assert!(
        app.world().get::<PhantomBreaker>(entity).is_some(),
        "phantom entity should have PhantomBreaker marker"
    );
    assert!(
        app.world().get::<ExtraBreaker>(entity).is_some(),
        "phantom extra spawn should have ExtraBreaker"
    );
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(entity)
            .is_some(),
        "phantom extra spawn should have CleanupOnExit<NodeState>"
    );
}

// ── Wave 2 Behavior 4: Rendered .phantom() mixes phantom_color_rgb into material ──

#[derive(Resource, Default)]
struct SpawnedEntities {
    real:    Option<Entity>,
    phantom: Option<Entity>,
    same:    Option<Entity>,
}

fn spawn_color_mix_app() -> App {
    let def = test_breaker_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>()
        .init_resource::<SpawnedEntities>();
    app.add_systems(
        Update,
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>,
              mut spawned: ResMut<SpawnedEntities>| {
            spawned.real = Some(
                Breaker::builder()
                    .definition(&def)
                    .rendered(&mut meshes, &mut materials)
                    .primary()
                    .spawn(&mut commands),
            );
            spawned.phantom = Some(
                Breaker::builder()
                    .definition(&def)
                    .phantom(BreakerPhantomParams {
                        lifespan:          2.5,
                        phantom_color_rgb: [0.4, 0.8, 1.0],
                        flicker_frequency: 4.0,
                        flicker_min_alpha: 0.3,
                    })
                    .rendered(&mut meshes, &mut materials)
                    .extra()
                    .spawn(&mut commands),
            );
            let base_rgb = crate::breaker::definition::DEFAULT_COLOR_RGB;
            spawned.same = Some(
                Breaker::builder()
                    .definition(&def)
                    .phantom(BreakerPhantomParams {
                        lifespan:          2.5,
                        phantom_color_rgb: base_rgb,
                        flicker_frequency: 4.0,
                        flicker_min_alpha: 0.3,
                    })
                    .rendered(&mut meshes, &mut materials)
                    .extra()
                    .spawn(&mut commands),
            );
        },
    );
    app.update();
    app
}

fn get_srgba(app: &App, entity: Entity) -> Srgba {
    let handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .expect("entity should have MeshMaterial2d")
        .0
        .clone();
    app.world()
        .resource::<Assets<ColorMaterial>>()
        .get(&handle)
        .expect("material exists")
        .color
        .to_srgba()
}

#[test]
fn phantom_rendered_spawn_mixes_phantom_color_rgb_into_material() {
    let app = spawn_color_mix_app();
    let spawned = app.world().resource::<SpawnedEntities>();
    let real_entity = spawned.real.expect("real spawned");
    let phantom_entity = spawned.phantom.expect("phantom spawned");

    let rc = get_srgba(&app, real_entity);
    let pc = get_srgba(&app, phantom_entity);
    let base_rgb = crate::breaker::definition::DEFAULT_COLOR_RGB;

    assert!(
        (pc.red - rc.red).abs() > 1e-3
            || (pc.green - rc.green).abs() > 1e-3
            || (pc.blue - rc.blue).abs() > 1e-3,
        "phantom material color should differ from real breaker color (mix must happen); \
         real={:?}, phantom={:?}",
        (rc.red, rc.green, rc.blue),
        (pc.red, pc.green, pc.blue),
    );
    assert!(
        (pc.red - 0.4).abs() > 1e-3
            || (pc.green - 0.8).abs() > 1e-3
            || (pc.blue - 1.0).abs() > 1e-3,
        "phantom material color should NOT exactly equal phantom_color_rgb [0.4, 0.8, 1.0] (must be a blend); \
         phantom={:?}",
        (pc.red, pc.green, pc.blue),
    );
    assert!(
        (rc.red - base_rgb[0]).abs() <= 1e-3
            && (rc.green - base_rgb[1]).abs() <= 1e-3
            && (rc.blue - base_rgb[2]).abs() <= 1e-3,
        "real breaker color should approximately equal DEFAULT_COLOR_RGB {:?}; got ({}, {}, {})",
        base_rgb,
        rc.red,
        rc.green,
        rc.blue,
    );
}

#[test]
fn phantom_rendered_spawn_mixing_base_rgb_with_itself_is_idempotent() {
    let app = spawn_color_mix_app();
    let spawned = app.world().resource::<SpawnedEntities>();
    let same_entity = spawned.same.expect("same-color phantom spawned");

    let sc = get_srgba(&app, same_entity);
    let base_rgb = crate::breaker::definition::DEFAULT_COLOR_RGB;

    assert!(
        (sc.red - base_rgb[0]).abs() <= 1e-3
            && (sc.green - base_rgb[1]).abs() <= 1e-3
            && (sc.blue - base_rgb[2]).abs() <= 1e-3,
        "mixing DEFAULT_COLOR_RGB with itself should yield DEFAULT_COLOR_RGB {:?}; \
         got ({}, {}, {})",
        base_rgb,
        sc.red,
        sc.green,
        sc.blue,
    );
}

// ── Wave 2 Behavior 5: Non-phantom spawn does NOT insert phantom components ──

#[test]
fn non_phantom_spawn_does_not_insert_phantom_components() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    // Sanity check: the non-phantom spawn still produces a valid Breaker
    assert!(
        world.get::<Breaker>(entity).is_some(),
        "non-phantom spawn should still have Breaker marker"
    );
    // Edge: phantom components must NOT appear on a non-phantom spawn
    assert!(
        world.get::<PhantomBreaker>(entity).is_none(),
        "non-phantom spawn should NOT have PhantomBreaker"
    );
    assert!(
        world.get::<Lifespan>(entity).is_none(),
        "non-phantom spawn should NOT have Lifespan"
    );
    assert!(
        world.get::<PhantomFlicker>(entity).is_none(),
        "non-phantom spawn should NOT have PhantomFlicker"
    );
}
