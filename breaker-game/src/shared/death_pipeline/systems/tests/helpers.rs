//! Shared test helpers for the death pipeline `systems` tests.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::{apply_damage, detect_deaths, handle_kill, process_despawn_requests};
use crate::{
    bolt::{
        components::Bolt,
        messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost},
    },
    breaker::messages::{BreakerImpactCell, BreakerImpactWall, BumpPerformed, BumpWhiffed, NoBump},
    cells::{components::Cell, messages::CellImpactWall},
    shared::{
        death_pipeline::{
            damage_dealt::DamageDealt, despawn_entity::DespawnEntity, destroyed::Destroyed,
            game_entity::GameEntity, hp::Hp, invulnerable::Invulnerable,
            kill_yourself::KillYourself, killed_by::KilledBy,
        },
        rng::GameRng,
        test_utils::{TestAppBuilder, attach_message_capture},
    },
    walls::components::Wall,
};

/// Registers every cross-domain message and resource that `EffectV3Plugin`'s
/// systems read but which `EffectV3Plugin` does not register itself.
///
/// `EffectV3Plugin`'s impact, bump, and bolt-lost bridges consume messages owned
/// by the `bolt`, `breaker`, and `cells` domain plugins. Integration tests that
/// add `EffectV3Plugin` but not those full domain plugins hit Bevy 0.18's
/// system parameter validation ("Message not initialized") unless the message
/// types are initialized directly.
///
/// In addition, `tick_chain_lightning` and `tick_entropy_engine` take
/// `ResMut<GameRng>` but `GameRng` is inserted by the game setup pipeline, not
/// `EffectV3Plugin`. Bevy 0.18 also validates parameter existence and panics
/// with "Resource does not exist" unless the resource is inserted. A
/// deterministic seed (`42`) is used to match the pattern in existing effect
/// tests (see `chain_lightning/systems.rs`, `spawn_bolts/config.rs`, etc.).
///
/// Call this **before** adding `EffectV3Plugin`.
///
/// Messages NOT registered here (intentional):
/// - `Destroyed<Cell|Bolt|Wall|Breaker>` — registered by `DeathPipelinePlugin`
/// - `NodeTimerThresholdCrossed` — registered by `node::register::register`
/// - `EffectTimerExpired` — registered by `time::register::register`
///
/// Resources NOT registered here (intentional):
/// - `Time` — provided by `TestAppBuilder`'s plugin group
/// - `SpawnStampRegistry` — init'd by `EffectV3Plugin::build`
/// - `NodeTimerThresholdRegistry` — init'd by `node::register::register`
/// - `NodeTimer` — systems read it as `Option<Res<NodeTimer>>` and no-op when absent
pub(crate) fn register_effect_v3_test_infrastructure(app: &mut App) {
    // Impact bridges (`impact/bridges.rs::ImpactReaders`)
    app.add_message::<BoltImpactCell>();
    app.add_message::<BoltImpactWall>();
    app.add_message::<BoltImpactBreaker>();
    app.add_message::<BreakerImpactCell>();
    app.add_message::<BreakerImpactWall>();
    app.add_message::<CellImpactWall>();

    // Bump bridges (`bump/bridges.rs`)
    app.add_message::<BumpPerformed>();
    app.add_message::<BumpWhiffed>();
    app.add_message::<NoBump>();

    // Bolt-lost bridge (`bolt_lost/bridges.rs`)
    app.add_message::<BoltLost>();

    // `tick_chain_lightning` + `tick_entropy_engine` require `ResMut<GameRng>`.
    // Deterministic seed `42` matches existing effect tests.
    app.insert_resource(GameRng::from_seed(42));
}

// ── Test-only entity type ────────────────────────────────────────────

#[derive(Component, Default, Clone)]
pub(super) struct TestEntity;

impl GameEntity for TestEntity {}

// ── Test helpers ─────────────────────────────────────────────────────

/// Resource to hold damage messages that should be enqueued before the system runs.
#[derive(Resource, Default)]
pub(super) struct PendingDamage(pub Vec<DamageDealt<TestEntity>>);

/// System that writes `DamageDealt<TestEntity>` from the `PendingDamage` resource.
pub(super) fn enqueue_damage(
    pending: Res<PendingDamage>,
    mut writer: MessageWriter<DamageDealt<TestEntity>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// Resource to hold despawn messages that should be enqueued before the system runs.
#[derive(Resource, Default)]
pub(super) struct PendingDespawns(pub Vec<DespawnEntity>);

/// System that writes `DespawnEntity` from the `PendingDespawns` resource.
pub(super) fn enqueue_despawns(
    pending: Res<PendingDespawns>,
    mut writer: MessageWriter<DespawnEntity>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// Resource to hold `KillYourself<TestEntity>` messages that should be enqueued
/// before `handle_kill::<TestEntity>` runs each tick.
#[derive(Resource, Default)]
pub(super) struct PendingKillRequests(pub Vec<KillYourself<TestEntity>>);

/// System that writes `KillYourself<TestEntity>` from the `PendingKillRequests`
/// resource. Ordered `.before(handle_kill::<TestEntity>)` in the test app.
pub(super) fn enqueue_kill_requests(
    pending: Res<PendingKillRequests>,
    mut writer: MessageWriter<KillYourself<TestEntity>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn build_apply_damage_app() -> App {
    TestAppBuilder::new()
        .with_message::<DamageDealt<TestEntity>>()
        .with_resource::<PendingDamage>()
        .with_system(
            FixedUpdate,
            enqueue_damage.before(apply_damage::<TestEntity>),
        )
        .with_system(FixedUpdate, apply_damage::<TestEntity>)
        .build()
}

pub(super) fn build_detect_deaths_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<KillYourself<TestEntity>>()
        .with_system(FixedUpdate, detect_deaths::<TestEntity>)
        .build()
}

pub(super) fn build_despawn_app() -> App {
    TestAppBuilder::new()
        .with_message::<DespawnEntity>()
        .with_resource::<PendingDespawns>()
        .with_system(
            FixedUpdate,
            enqueue_despawns.before(process_despawn_requests),
        )
        .with_system(FixedUpdate, process_despawn_requests)
        .build()
}

/// Builds a test app for `handle_kill::<TestEntity>`. Wires:
/// - `KillYourself<TestEntity>` message (via `PendingKillRequests` + enqueue system)
/// - `Destroyed<TestEntity>` captured in `MessageCollector<Destroyed<TestEntity>>`
/// - `DespawnEntity` captured in `MessageCollector<DespawnEntity>`
/// - `handle_kill::<TestEntity>` ordered after `enqueue_kill_requests`
pub(super) fn build_handle_kill_app() -> App {
    TestAppBuilder::new()
        .with_message::<KillYourself<TestEntity>>()
        .with_message_capture::<Destroyed<TestEntity>>()
        .with_message_capture::<DespawnEntity>()
        .with_resource::<PendingKillRequests>()
        .with_system(
            FixedUpdate,
            enqueue_kill_requests.before(handle_kill::<TestEntity>),
        )
        .with_system(FixedUpdate, handle_kill::<TestEntity>)
        .build()
}

pub(super) fn spawn_test_entity(app: &mut App, hp_value: f32) -> Entity {
    app.world_mut()
        .spawn((TestEntity, Hp::new(hp_value), KilledBy::default()))
        .id()
}

/// Spawns a `TestEntity` with `Invulnerable` — used by Group I tests to
/// exercise the `Without<Invulnerable>` filter on `apply_damage<T>`.
pub(super) fn spawn_test_entity_invulnerable(app: &mut App, hp_value: f32) -> Entity {
    app.world_mut()
        .spawn((
            TestEntity,
            Hp::new(hp_value),
            KilledBy::default(),
            Invulnerable,
        ))
        .id()
}

pub(super) fn damage_msg(
    target: Entity,
    amount: f32,
    dealer: Option<Entity>,
) -> DamageDealt<TestEntity> {
    DamageDealt {
        dealer,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Helper to build a `KillYourself<TestEntity>` message.
pub(super) fn kill_msg(victim: Entity, killer: Option<Entity>) -> KillYourself<TestEntity> {
    KillYourself {
        victim,
        killer,
        _marker: PhantomData,
    }
}

// ── Plugin-integration test helpers ────────────────────────────────────────
//
// Used by `handle_kill_integration.rs` and `handle_kill_bridge.rs`. Keeps the
// builder, pending-kill resources, enqueue systems, and collector wiring in
// one place so the test files stay focused on scenario-specific setup and
// assertions.

/// Builds a plugin-integration app with the full effects pipeline
/// (`DeathPipelinePlugin` + cross-domain messages + `EffectV3Plugin`).
pub(super) fn build_plugin_integration_app() -> App {
    TestAppBuilder::new().with_effects_pipeline().build()
}

/// Pending `DamageDealt<Cell>` messages to enqueue each tick. Used by
/// plugin-integration tests to inject damage at the top of the pipeline and
/// verify the full `apply_damage<Cell>` → `detect_deaths<Cell>` →
/// `handle_kill<Cell>` → `process_despawn_requests` chain in one tick.
#[derive(Resource, Default)]
pub(super) struct PendingCellDamage(pub Vec<DamageDealt<Cell>>);

/// System that writes `DamageDealt<Cell>` from `PendingCellDamage` each tick.
pub(super) fn enqueue_cell_damage(
    pending: Res<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// Pending `KillYourself<Cell>` messages to enqueue each tick.
#[derive(Resource, Default)]
pub(super) struct PendingCellKills(pub Vec<KillYourself<Cell>>);

/// Pending `KillYourself<Bolt>` messages to enqueue each tick.
#[derive(Resource, Default)]
pub(super) struct PendingBoltKills(pub Vec<KillYourself<Bolt>>);

/// System that writes `KillYourself<Cell>` from `PendingCellKills` each tick.
pub(super) fn enqueue_cell_kills(
    pending: Res<PendingCellKills>,
    mut writer: MessageWriter<KillYourself<Cell>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// System that writes `KillYourself<Bolt>` from `PendingBoltKills` each tick.
pub(super) fn enqueue_bolt_kills(
    pending: Res<PendingBoltKills>,
    mut writer: MessageWriter<KillYourself<Bolt>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// Attaches a `MessageCollector<Destroyed<Cell>>` to an app that already has
/// `Destroyed<Cell>` registered (e.g., by `DeathPipelinePlugin`).
pub(super) fn attach_cell_destroyed_collector(app: &mut App) {
    attach_message_capture::<Destroyed<Cell>>(app);
}

/// Attaches a `MessageCollector<Destroyed<Bolt>>` to an app that already has
/// `Destroyed<Bolt>` registered (e.g., by `DeathPipelinePlugin`).
pub(super) fn attach_bolt_destroyed_collector(app: &mut App) {
    attach_message_capture::<Destroyed<Bolt>>(app);
}

/// Attaches a `MessageCollector<DespawnEntity>` to an app that already has
/// `DespawnEntity` registered (e.g., by `DeathPipelinePlugin`).
pub(super) fn attach_despawn_collector(app: &mut App) {
    attach_message_capture::<DespawnEntity>(app);
}

/// Pending `KillYourself<Wall>` messages to enqueue each tick.
#[derive(Resource, Default)]
pub(super) struct PendingWallKills(pub Vec<KillYourself<Wall>>);

/// System that writes `KillYourself<Wall>` from `PendingWallKills` each tick.
pub(super) fn enqueue_wall_kills(
    pending: Res<PendingWallKills>,
    mut writer: MessageWriter<KillYourself<Wall>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// Attaches a `MessageCollector<Destroyed<Wall>>` to an app that already has
/// `Destroyed<Wall>` registered (e.g., by `DeathPipelinePlugin`).
pub(super) fn attach_wall_destroyed_collector(app: &mut App) {
    attach_message_capture::<Destroyed<Wall>>(app);
}
