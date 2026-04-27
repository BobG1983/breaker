use bevy::prelude::*;

use crate::{Dmgable, sets::DmgSystems};

// ── Dummy `Dmgable` types — used by message_registration and isolation tests ──

#[derive(Component)]
pub(super) struct TestT;
impl Dmgable for TestT {}

#[derive(Component)]
pub(super) struct TestU;
impl Dmgable for TestU {}

// ── Execution-log infrastructure for chain-order tests ──

#[derive(Resource, Default)]
pub(super) struct ExecutionLog(pub(super) Vec<DmgSystems>);

fn record_emit_damage(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::EmitDamage);
}
fn record_post_emit(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::PostEmitDamage);
}
fn record_apply_damage_boosts(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::ApplyDamageBoosts);
}
fn record_mutate_damage(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::MutateDamage);
}
fn record_post_mutate(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::PostMutateDamage);
}
fn record_apply_vulnerable(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::ApplyVulnerable);
}
fn record_apply_damage(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::ApplyDamage);
}
fn record_post_apply(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::PostApplyDamage);
}
fn record_emit_kill(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::EmitKill);
}
fn record_mutate_kill(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::MutateKill);
}
fn record_apply_kill(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::ApplyKill);
}
fn record_post_kill(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::PostApplyKill);
}
fn record_emit_heal(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::EmitHeal);
}
fn record_mutate_heal(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::MutateHeal);
}
fn record_apply_heal(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::ApplyHeal);
}
fn record_post_heal(mut log: ResMut<ExecutionLog>) {
    log.0.push(DmgSystems::PostApplyHeal);
}

pub(super) fn build_app_with_recorders() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(crate::RantzDmgPlugin);
    app.init_resource::<ExecutionLog>();
    app.add_systems(
        FixedUpdate,
        (
            (
                record_emit_damage.in_set(DmgSystems::EmitDamage),
                record_post_emit.in_set(DmgSystems::PostEmitDamage),
                record_apply_damage_boosts.in_set(DmgSystems::ApplyDamageBoosts),
                record_mutate_damage.in_set(DmgSystems::MutateDamage),
                record_post_mutate.in_set(DmgSystems::PostMutateDamage),
                record_apply_vulnerable.in_set(DmgSystems::ApplyVulnerable),
                record_apply_damage.in_set(DmgSystems::ApplyDamage),
                record_post_apply.in_set(DmgSystems::PostApplyDamage),
            ),
            (
                record_emit_kill.in_set(DmgSystems::EmitKill),
                record_mutate_kill.in_set(DmgSystems::MutateKill),
                record_apply_kill.in_set(DmgSystems::ApplyKill),
                record_post_kill.in_set(DmgSystems::PostApplyKill),
                record_emit_heal.in_set(DmgSystems::EmitHeal),
                record_mutate_heal.in_set(DmgSystems::MutateHeal),
                record_apply_heal.in_set(DmgSystems::ApplyHeal),
                record_post_heal.in_set(DmgSystems::PostApplyHeal),
            ),
        ),
    );
    app
}

// ── Counters used across despawn_processing and isolation tests ──

#[derive(Resource, Default)]
pub(super) struct PreDespawnWitnessCount(pub(super) u32);

pub(super) fn pre_despawn_witness(mut c: ResMut<PreDespawnWitnessCount>) {
    c.0 += 1;
}

#[derive(Resource, Default)]
pub(super) struct FixedTickCount(pub(super) u32);

pub(super) fn fixed_tick_witness(mut c: ResMut<FixedTickCount>) {
    c.0 += 1;
}

#[derive(Resource, Default)]
pub(super) struct PostCounters {
    pub(super) emit:   u32,
    pub(super) mutate: u32,
    pub(super) apply:  u32,
    pub(super) kill:   u32,
    pub(super) heal:   u32,
}

pub(super) fn bump_post_emit(mut c: ResMut<PostCounters>) {
    c.emit += 1;
}
pub(super) fn bump_post_mutate(mut c: ResMut<PostCounters>) {
    c.mutate += 1;
}
pub(super) fn bump_post_apply(mut c: ResMut<PostCounters>) {
    c.apply += 1;
}
pub(super) fn bump_post_kill(mut c: ResMut<PostCounters>) {
    c.kill += 1;
}
pub(super) fn bump_post_heal(mut c: ResMut<PostCounters>) {
    c.heal += 1;
}
