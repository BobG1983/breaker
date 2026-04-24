//! Shared test fixtures for Iron Curtain protocol tests.
//!
//! App builders, canonical `IronCurtainConfig`, entity spawners (breaker,
//! bolt, cell — with and without markers), `BoltLost` writer, `ActiveProtocols`
//! seeder, `DamageDealt<Cell>` Iron-Curtain-filtered collector reader, and the
//! `activate_now` CommandQueue-flush helper.
//!
//! Canonical config is `damage_fraction: 0.5`, `falloff_start: 50.0` — the
//! design-doc worked-example values. The RON asset's `0.25` / `0.5` is
//! exercised only by the `ron_asset.rs` drift-guard tests via `include_str!`.

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{IRON_CURTAIN_SENTINEL, IronCurtainConfig, activate, register};
use crate::{
    bolt::components::BoltBaseDamage,
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Iron Curtain test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` initialised, `BoltLost` registered, `DamageDealt<Cell>`
/// capture installed, canonical `IronCurtainConfig` inserted,
/// `PlayfieldConfig::default()` inserted (height `600.0`), and `register`
/// called.
///
/// Does NOT seed `ActiveProtocols` with Iron Curtain — tests that need the
/// protocol active call [`seed_active_protocols_with_iron_curtain`].
pub(super) fn build_iron_curtain_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_iron_curtain_config());
    app.world_mut().insert_resource(PlayfieldConfig::default());
    register(&mut app);
    app
}

/// Same as [`build_iron_curtain_app`] but omits `IronCurtainConfig` insertion.
/// Used to exercise the harness-safe `Option<Res<IronCurtainConfig>>` guard.
pub(super) fn build_iron_curtain_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut().insert_resource(PlayfieldConfig::default());
    register(&mut app);
    app
}

/// Same as [`build_iron_curtain_app`] but omits `PlayfieldConfig` insertion.
/// Used to exercise the harness-safe `Option<Res<PlayfieldConfig>>` guard.
pub(super) fn build_iron_curtain_app_no_playfield() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_iron_curtain_config());
    register(&mut app);
    app
}

/// Same as [`build_iron_curtain_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for the `in_state(NodeState::Playing)`
/// gate test.
pub(super) fn build_iron_curtain_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_iron_curtain_config());
    app.world_mut().insert_resource(PlayfieldConfig::default());
    register(&mut app);
    app
}

/// Same as [`build_iron_curtain_app`] but overrides the inserted
/// `PlayfieldConfig` to `PlayfieldConfig { height, ..Default::default() }`.
/// Used by falloff-math tests that reproduce the design doc's worked examples
/// (which assume `height: 400.0`).
pub(super) fn build_iron_curtain_app_with_playfield_height(height: f32) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_iron_curtain_config());
    app.world_mut().insert_resource(PlayfieldConfig {
        height,
        ..Default::default()
    });
    register(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Iron Curtain config used across all system-behavior tests.
/// Matches the design-doc worked example: `damage_fraction: 0.5,
/// falloff_start: 50.0`.
///
/// NOTE: The RON asset's `damage_fraction: 0.25, falloff_start: 0.5` is
/// exercised only by the `ron_asset.rs` drift-guard tests via `include_str!`
/// and does NOT use this helper.
pub(super) const fn canonical_iron_curtain_config() -> IronCurtainConfig {
    IronCurtainConfig {
        damage_fraction: 0.5,
        falloff_start:   50.0,
    }
}

/// Inserts the given `IronCurtainConfig` as a resource.
pub(super) fn install_iron_curtain_config(app: &mut App, cfg: IronCurtainConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts an Iron Curtain `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(IronCurtain)` run-condition passes.
pub(super) fn seed_active_protocols_with_iron_curtain(
    app: &mut App,
    damage_fraction: f32,
    falloff_start: f32,
) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Iron Curtain".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::IronCurtain {
                damage_fraction,
                falloff_start,
            },
        });
}

/// Invokes `iron_curtain::activate` directly via a fresh `CommandQueue` so
/// each call has an independent, deterministic flush. Mirrors
/// `debt_collector::tests::helpers::activate_now`.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Entity spawners ─────────────────────────────────────────────────────────

/// Spawns `(Breaker, Position2D(pos))`.
pub(super) fn spawn_breaker_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Breaker, Position2D(pos))).id()
}

/// Spawns `(Bolt, BoltBaseDamage(base))`.
pub(super) fn spawn_bolt_with_base_damage(app: &mut App, base: f32) -> Entity {
    app.world_mut().spawn((Bolt, BoltBaseDamage(base))).id()
}

/// Spawns `(Bolt,)` with NO `BoltBaseDamage` component — used to exercise the
/// `DEFAULT_BOLT_BASE_DAMAGE` fallback path.
pub(super) fn spawn_bolt_without_base_damage(app: &mut App) -> Entity {
    app.world_mut().spawn(Bolt).id()
}

/// Spawns `(Cell, Position2D(pos))`.
pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut().spawn((Cell, Position2D(pos))).id()
}

/// Spawns `(Cell, Position2D(pos))` and conditionally inserts `Dead` and/or
/// `Invulnerable` markers. Used by the exclusion tests.
pub(super) fn spawn_cell_at_with_markers(
    app: &mut App,
    pos: Vec2,
    dead: bool,
    invulnerable: bool,
) -> Entity {
    let entity = app.world_mut().spawn((Cell, Position2D(pos))).id();
    if dead {
        app.world_mut().entity_mut(entity).insert(Dead);
    }
    if invulnerable {
        app.world_mut().entity_mut(entity).insert(Invulnerable);
    }
    entity
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BoltLost` message. `breaker` field is
/// `Entity::PLACEHOLDER` — Iron Curtain never inspects `msg.breaker` (it
/// queries `With<Breaker>`), so the placeholder is safe.
pub(super) fn write_bolt_lost(app: &mut App, bolt: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt,
            breaker: Entity::PLACEHOLDER,
        });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `DamageDealt<Cell>` whose `source` matches the
/// Iron Curtain sentinel string. Isolates Iron Curtain's wave emissions from
/// any other `DamageDealt<Cell>` messages.
pub(super) fn collected_iron_curtain_damage(app: &App) -> Vec<DamageDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<DamageDealt<Cell>>>()
        .0
        .iter()
        .filter(|msg| msg.source == Some(SourceId::from(IRON_CURTAIN_SENTINEL)))
        .cloned()
        .collect()
}

/// Scans the provided slice and returns the `amount` field of the first
/// `DamageDealt<Cell>` message whose `target` matches the given entity, else
/// `None`. Used by the multi-cell wave test to assert per-target amounts by
/// entity rather than by message order.
pub(super) fn amount_for_target(msgs: &[DamageDealt<Cell>], target: Entity) -> Option<f32> {
    msgs.iter().find(|m| m.target == target).map(|m| m.amount)
}
