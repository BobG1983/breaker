use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use super::{
    super::system::CellsPlugin,
    helpers::{
        PluginTestPendingBoltImpact, PluginTestPendingCellDamage, enqueue_cell_damage_plugin_test,
        enqueue_plugin_bolt_impact, plugin_damage_msg_from,
    },
};
use crate::{
    bolt::components::PiercingRemaining,
    cells::{behaviors::armored::components::ArmorDirection, test_utils::spawn_cell_in_world},
    effect_v3::EffectV3Plugin,
    prelude::*,
    shared::death_pipeline::{
        DeathPipelinePlugin, systems::tests::helpers::register_effect_v3_test_infrastructure,
    },
};

fn spawn_plugin_armored_cell(
    app: &mut App,
    pos: Vec2,
    value: u8,
    facing: ArmorDirection,
    hp: f32,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .armored_facing(value, facing)
            .position(pos)
            .dimensions(10.0, 10.0)
            .hp(hp)
            .headless()
            .spawn(commands)
    });
    app.world_mut()
        .entity_mut(entity)
        .insert(rantzsoft_spatial2d::components::GlobalPosition2D(pos));
    entity
}

fn spawn_plugin_test_bolt(app: &mut App, piercing: u32) -> Entity {
    app.world_mut().spawn(PiercingRemaining(piercing)).id()
}

fn armored_plugin_app_loading() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_sub_state::<RunState>()
        .add_sub_state::<NodeState>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_message::<BoltImpactCell>()
        .add_message::<BreakerImpactCell>()
        .insert_resource(CollisionQuadtree::default())
        .init_resource::<crate::shared::playfield::PlayfieldConfig>();
    // Configure BoltSystems::CellCollision so the set exists without BoltPlugin
    app.configure_sets(FixedUpdate, crate::bolt::sets::BoltSystems::CellCollision);
    app.add_plugins(DeathPipelinePlugin);
    register_effect_v3_test_infrastructure(&mut app);
    app.add_plugins(EffectV3Plugin);
    app.add_plugins(CellsPlugin);
    app.init_resource::<PluginTestPendingBoltImpact>();
    app.init_resource::<PluginTestPendingCellDamage>();
    app.add_systems(
        FixedUpdate,
        enqueue_plugin_bolt_impact.in_set(crate::bolt::sets::BoltSystems::CellCollision),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage_plugin_test.in_set(crate::bolt::sets::BoltSystems::CellCollision),
    );
    app
}

fn armored_plugin_advance_to_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
}

/// Behavior 27: `CellsPlugin` registers `check_armor_direction` ordered
/// `.after(BoltSystems::CellCollision).before(DeathPipelineSystems::ApplyDamage)`.
#[test]
fn cells_plugin_registers_check_armor_direction_before_apply_damage() {
    let mut app = armored_plugin_app_loading();

    let cell = spawn_plugin_armored_cell(
        &mut app,
        Vec2::new(0.0, 0.0),
        2,
        ArmorDirection::Bottom,
        20.0,
    );
    let bolt = spawn_plugin_test_bolt(&mut app, 0);

    armored_plugin_advance_to_playing(&mut app);

    app.world_mut()
        .resource_mut::<PluginTestPendingBoltImpact>()
        .0
        .push(BoltImpactCell {
            bolt,
            cell,
            impact_normal: Vec2::NEG_Y,
            piercing_remaining: 0,
        });
    app.world_mut()
        .resource_mut::<PluginTestPendingCellDamage>()
        .0
        .push(plugin_damage_msg_from(cell, 5.0, bolt));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "CellsPlugin should register check_armor_direction before ApplyDamage; armor blocked, got hp.current == {}",
        hp.current
    );
    assert!(app.world().get::<Dead>(cell).is_none());
}

/// Behavior 27 edge (control): weak face hit passes through, proving
/// the system is genuinely registered and not a stub no-op.
#[test]
fn cells_plugin_armored_weak_face_passes_through_control() {
    let mut app = armored_plugin_app_loading();

    // Armor on Top, hit on Bottom (weak face) via NEG_Y
    let cell =
        spawn_plugin_armored_cell(&mut app, Vec2::new(0.0, 0.0), 2, ArmorDirection::Top, 20.0);
    let bolt = spawn_plugin_test_bolt(&mut app, 0);

    armored_plugin_advance_to_playing(&mut app);

    app.world_mut()
        .resource_mut::<PluginTestPendingBoltImpact>()
        .0
        .push(BoltImpactCell {
            bolt,
            cell,
            impact_normal: Vec2::NEG_Y,
            piercing_remaining: 0,
        });
    app.world_mut()
        .resource_mut::<PluginTestPendingCellDamage>()
        .0
        .push(plugin_damage_msg_from(cell, 5.0, bolt));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).expect("cell should have Hp");
    assert!(
        (hp.current - 15.0).abs() < f32::EPSILON,
        "weak face hit should pass through via plugin-registered system, got hp.current == {}",
        hp.current
    );
}
