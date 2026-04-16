use std::{marker::PhantomData, time::Duration};

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use super::super::system::CellsPlugin;
use crate::{
    cells::test_utils::spawn_cell_in_world,
    effect_v3::EffectV3Plugin,
    prelude::*,
    shared::death_pipeline::{
        DeathPipelinePlugin, sets::DeathPipelineSystems,
        systems::tests::helpers::register_effect_v3_test_infrastructure,
    },
};

pub(super) fn cells_plugin_app() -> App {
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
    app.add_plugins(DeathPipelinePlugin);
    register_effect_v3_test_infrastructure(&mut app);
    app.add_plugins(EffectV3Plugin);
    app.add_plugins(CellsPlugin);
    // Navigate through state hierarchy to reach NodeState::Playing
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
    app
}

pub(super) fn tick_cells(app: &mut App, dt: Duration) {
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .set_timestep(dt);
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(dt);
    app.update();
}

/// Resource seeding `DamageDealt<Cell>` through a one-shot enqueue system
/// registered `before(ApplyDamage)`. Mirrors the scaffold in
/// `behaviors/sequence/tests/helpers.rs` so the cross-plugin tests can
/// deliver damage without depending on bolt collision.
#[derive(Resource, Default)]
pub(super) struct PluginTestPendingCellDamage(pub(super) Vec<DamageDealt<Cell>>);

pub(super) fn enqueue_cell_damage_plugin_test(
    mut pending: ResMut<PluginTestPendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Builds a `cells_plugin_app`-style App but does NOT navigate into
/// `NodeState::Playing`. Tests drive the transition after spawning.
pub(super) fn sequence_plugin_app_loading() -> App {
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
    app.add_plugins(DeathPipelinePlugin);
    register_effect_v3_test_infrastructure(&mut app);
    app.add_plugins(EffectV3Plugin);
    app.add_plugins(CellsPlugin);
    app.init_resource::<PluginTestPendingCellDamage>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage_plugin_test.before(DeathPipelineSystems::ApplyDamage),
    );
    app
}

pub(super) fn sequence_plugin_advance_to_playing(app: &mut App) {
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

pub(super) fn spawn_plugin_sequence_cell(
    app: &mut App,
    pos: Vec2,
    group: u32,
    position: u32,
    hp: f32,
) -> Entity {
    let entity = spawn_cell_in_world(app.world_mut(), |commands| {
        Cell::builder()
            .sequence(group, position)
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

pub(super) fn plugin_damage_msg(target: Entity, amount: f32) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: None,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

pub(super) fn plugin_damage_msg_from(
    target: Entity,
    amount: f32,
    dealer: Entity,
) -> DamageDealt<Cell> {
    DamageDealt {
        dealer: Some(dealer),
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Resource seeding `BoltImpactCell` through a one-shot enqueue system
/// so armored cross-plugin tests can deliver impact events without
/// depending on bolt collision.
#[derive(Resource, Default)]
pub(super) struct PluginTestPendingBoltImpact(pub(super) Vec<BoltImpactCell>);

pub(super) fn enqueue_plugin_bolt_impact(
    mut pending: ResMut<PluginTestPendingBoltImpact>,
    mut writer: MessageWriter<BoltImpactCell>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

/// Resource seeding `BreakerImpactCell` through a one-shot enqueue system.
#[derive(Resource, Default)]
pub(super) struct PluginTestPendingBreakerImpact(pub(super) Vec<BreakerImpactCell>);

pub(super) fn enqueue_plugin_breaker_impact(
    mut pending: ResMut<PluginTestPendingBreakerImpact>,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}
