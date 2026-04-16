use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use super::{
    super::system::CellsPlugin,
    helpers::{
        PluginTestPendingBoltImpact, PluginTestPendingBreakerImpact, PluginTestPendingCellDamage,
        enqueue_cell_damage_plugin_test, enqueue_plugin_bolt_impact, enqueue_plugin_breaker_impact,
        plugin_damage_msg_from,
    },
};
use crate::{
    cells::test_utils::spawn_cell_in_world,
    effect_v3::EffectV3Plugin,
    prelude::*,
    shared::death_pipeline::{
        DeathPipelinePlugin, sets::DeathPipelineSystems,
        systems::tests::helpers::register_effect_v3_test_infrastructure,
    },
};

/// Builds a plugin app that stays in Loading state.
fn survival_plugin_app_loading() -> App {
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
    app.init_resource::<PluginTestPendingBreakerImpact>();
    app.add_systems(
        FixedUpdate,
        enqueue_plugin_bolt_impact.in_set(crate::bolt::sets::BoltSystems::CellCollision),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage_plugin_test.in_set(crate::bolt::sets::BoltSystems::CellCollision),
    );
    app.add_systems(
        FixedUpdate,
        enqueue_plugin_breaker_impact.before(DeathPipelineSystems::ApplyDamage),
    );
    app
}

fn survival_plugin_advance_to_playing(app: &mut App) {
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

fn spawn_plugin_survival_cell(app: &mut App, hp: f32) -> Entity {
    spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .survival(crate::cells::definition::AttackPattern::StraightDown, 10.0)
            .position(Vec2::ZERO)
            .dimensions(10.0, 10.0)
            .hp(hp)
            .headless()
            .spawn(commands)
    })
}

/// Behavior 55: `CellsPlugin` registers `suppress_bolt_immune_damage` in `FixedUpdate`.
#[test]
fn cells_plugin_registers_suppress_bolt_immune_damage_before_apply_damage() {
    let mut app = survival_plugin_app_loading();

    let cell = spawn_plugin_survival_cell(&mut app, 20.0);
    let bolt = app.world_mut().spawn(Bolt).id();

    survival_plugin_advance_to_playing(&mut app);

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
        "CellsPlugin should register suppress_bolt_immune_damage before ApplyDamage; \
         BoltImmune cell should retain full HP, got hp.current == {}",
        hp.current
    );
}

/// Behavior 56: `suppress_bolt_immune_damage` does not run in `NodeState::Loading`.
#[test]
fn cells_plugin_suppress_bolt_immune_does_not_run_in_loading() {
    let mut app = survival_plugin_app_loading();

    let cell = spawn_plugin_survival_cell(&mut app, 20.0);
    let bolt = app.world_mut().spawn(Bolt).id();

    // Do NOT advance to Playing — stay in Loading
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

    // No crash — system didn't run (gated by run_if). Damage also doesn't
    // apply since death pipeline is gated too.
}
