use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
    shared::death_pipeline::{
        damage_dealt::DamageDealt,
        destroyed::Destroyed,
        game_entity::GameEntity,
        heal_dealt::{HealCap, HealDealt},
        kill_yourself::KillYourself,
        sets::DeathPipelineSystems,
        systems,
    },
    walls::components::Wall,
};

// Local test entity — `TestEntity` in `systems::tests::helpers` is
// `pub(super)` and unreachable from this module (different subtree).
#[derive(Component)]
pub(super) struct PluginTestEntity;
impl GameEntity for PluginTestEntity {}

/// Builds a test app with `DeathPipelinePlugin` plus `PluginTestEntity`
/// monomorphizations of all four death-pipeline systems (`apply_damage`,
/// `detect_deaths`, `handle_kill`, `apply_heal`) wired with the same
/// `DeathPipelineSystems::*` annotations the plugin uses for production
/// types.
pub(super) fn build_pipeline_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(DeathPipelinePlugin);

    app.add_message::<DamageDealt<PluginTestEntity>>();
    app.add_message::<HealDealt<PluginTestEntity>>();
    app.add_message::<KillYourself<PluginTestEntity>>();
    app.add_message::<Destroyed<PluginTestEntity>>();

    app.add_systems(
        FixedUpdate,
        systems::apply_damage::<PluginTestEntity>.in_set(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        systems::detect_deaths::<PluginTestEntity>.in_set(DeathPipelineSystems::DetectDeaths),
    );
    app.add_systems(
        FixedUpdate,
        systems::handle_kill::<PluginTestEntity>.in_set(DeathPipelineSystems::HandleKill),
    );
    app.add_systems(
        FixedUpdate,
        systems::apply_heal::<PluginTestEntity>.in_set(DeathPipelineSystems::ApplyHeal),
    );

    app
}

// ── Group K / Behavior 34: DeathPipelineSystems exhaustive match ─────

const _: fn(DeathPipelineSystems) = |s| match s {
    DeathPipelineSystems::ApplyDamage
    | DeathPipelineSystems::DetectDeaths
    | DeathPipelineSystems::HandleKill
    | DeathPipelineSystems::ApplyHeal => (),
};

// ── Group K / Behavior 33: per-type enqueue helpers for apply_heal wiring tests ──

#[derive(Resource, Default)]
pub(super) struct PendingCellHeal(pub(super) Vec<HealDealt<Cell>>);
pub(super) fn enqueue_cell_heal(
    pending: Res<PendingCellHeal>,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
pub(super) struct PendingBoltHeal(pub(super) Vec<HealDealt<Bolt>>);
pub(super) fn enqueue_bolt_heal(
    pending: Res<PendingBoltHeal>,
    mut writer: MessageWriter<HealDealt<Bolt>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
pub(super) struct PendingWallHeal(pub(super) Vec<HealDealt<Wall>>);
pub(super) fn enqueue_wall_heal(
    pending: Res<PendingWallHeal>,
    mut writer: MessageWriter<HealDealt<Wall>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
pub(super) struct PendingBreakerHeal(pub(super) Vec<HealDealt<Breaker>>);
pub(super) fn enqueue_breaker_heal(
    pending: Res<PendingBreakerHeal>,
    mut writer: MessageWriter<HealDealt<Breaker>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
pub(super) struct PendingSalvoHealPlugin(pub(super) Vec<HealDealt<Salvo>>);
pub(super) fn enqueue_salvo_heal_plugin(
    pending: Res<PendingSalvoHealPlugin>,
    mut writer: MessageWriter<HealDealt<Salvo>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

// ── Group I / Behaviors 26–29: PluginTestEntity enqueue helpers for pipeline ordering ──

#[derive(Resource, Default)]
pub(super) struct PendingPtDamage(pub(super) Vec<DamageDealt<PluginTestEntity>>);
pub(super) fn enqueue_pt_damage(
    pending: Res<PendingPtDamage>,
    mut writer: MessageWriter<DamageDealt<PluginTestEntity>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
pub(super) struct PendingPtHeal(pub(super) Vec<HealDealt<PluginTestEntity>>);
pub(super) fn enqueue_pt_heal(
    pending: Res<PendingPtHeal>,
    mut writer: MessageWriter<HealDealt<PluginTestEntity>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn pt_damage_msg(target: Entity, amount: f32) -> DamageDealt<PluginTestEntity> {
    DamageDealt {
        dealer: None,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

pub(super) fn pt_heal_msg(
    target: Entity,
    amount: f32,
    cap: HealCap,
) -> HealDealt<PluginTestEntity> {
    HealDealt {
        healer: None,
        target,
        amount,
        source: None,
        cap,
        _marker: PhantomData,
    }
}
