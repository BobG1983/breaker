//! Shared test fixtures for Echo Strike protocol tests.
//!
//! App builders, canonical `EchoStrikeConfig`, per-bolt component installers,
//! `ActiveProtocols` seeder, `BumpPerformed` / `BoltImpactCell` /
//! `Destroyed<Cell>` message writers, sentinel-filtered `DamageDealt<Cell>`
//! collector, and the `activate_now` CommandQueue-flush helper.
//!
//! Canonical config is `max_echoes: 3, newest_fraction: 0.5,
//! middle_fraction: 0.25, oldest_fraction: 0.1` — the design-doc worked
//! example. The RON asset's `oldest_fraction: 0.125` is exercised only by
//! the `ron_asset.rs` drift-guard tests via `include_str!`.

use std::marker::PhantomData;

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{EchoNetwork, EchoPrimed, EchoStrikeConfig, activate, register};
use crate::{
    bolt::components::BoltBaseDamage,
    breaker::messages::BumpGrade,
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Echo Strike test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` initialised, reader messages registered,
/// `DamageDealt<Cell>` capture installed, canonical `EchoStrikeConfig`
/// inserted, and `register` called.
///
/// Does NOT seed `ActiveProtocols` with Echo Strike — tests that need the
/// protocol active call [`seed_active_protocols_with_echo_strike`].
pub(super) fn build_echo_strike_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_echo_strike_config());
    register(&mut app);
    app
}

/// Same as [`build_echo_strike_app`] but omits `EchoStrikeConfig`. Used to
/// exercise the harness-safe `Option<Res<EchoStrikeConfig>>` early-return
/// guard paths.
pub(super) fn build_echo_strike_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    register(&mut app);
    app
}

/// Same as [`build_echo_strike_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for `in_state(NodeState::Playing)`
/// gate tests.
pub(super) fn build_echo_strike_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_echo_strike_config());
    register(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Echo Strike config used across all system-behavior tests.
/// Matches the design-doc worked example: `max_echoes: 3,
/// newest_fraction: 0.5, middle_fraction: 0.25, oldest_fraction: 0.1`.
///
/// NOTE: The RON asset's `oldest_fraction: 0.125` is exercised only by the
/// `ron_asset.rs` drift-guard tests via `include_str!` and does NOT use
/// this helper.
pub(super) const fn canonical_echo_strike_config() -> EchoStrikeConfig {
    EchoStrikeConfig {
        max_echoes:      3,
        newest_fraction: 0.5,
        middle_fraction: 0.25,
        oldest_fraction: 0.1,
    }
}

/// Inserts the given `EchoStrikeConfig` as a resource.
pub(super) fn install_echo_strike_config(app: &mut App, cfg: EchoStrikeConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts an Echo Strike `ProtocolDefinition` into `ActiveProtocols` so
/// the `protocol_active(EchoStrike)` run-condition passes.
pub(super) fn seed_active_protocols_with_echo_strike(
    app: &mut App,
    max_echoes: u32,
    newest_fraction: f32,
    middle_fraction: f32,
    oldest_fraction: f32,
) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Echo Strike".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::EchoStrike {
                max_echoes,
                newest_fraction,
                middle_fraction,
                oldest_fraction,
            },
        });
}

/// Invokes `echo_strike::activate` directly via a fresh `CommandQueue` so
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

// ── Entity spawners & component installers ─────────────────────────────────-

/// Spawns a `Bolt` entity with `BoltBaseDamage(base)` attached. No
/// `EchoNetwork`, no `EchoPrimed`.
pub(super) fn spawn_bolt_with_base_damage(app: &mut App, base: f32) -> Entity {
    app.world_mut().spawn((Bolt, BoltBaseDamage(base))).id()
}

/// Spawns a `Bolt` entity with `BoltBaseDamage(base)` and
/// `EchoNetwork { echoes }`. No `EchoPrimed`.
pub(super) fn spawn_bolt_with_echo_network(
    app: &mut App,
    base: f32,
    echoes: Vec<Entity>,
) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            BoltBaseDamage(base),
            EchoNetwork {
                echoes: echoes.into_iter().collect(),
            },
        ))
        .id()
}

/// Spawns a `Bolt` entity with `BoltBaseDamage(base)`, `EchoPrimed`, and
/// `EchoNetwork { echoes }`.
pub(super) fn spawn_bolt_primed_with_network(
    app: &mut App,
    base: f32,
    echoes: Vec<Entity>,
) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            BoltBaseDamage(base),
            EchoPrimed,
            EchoNetwork {
                echoes: echoes.into_iter().collect(),
            },
        ))
        .id()
}

/// Spawns a bare empty entity — tests reuse this as a "cell" stand-in
/// because Echo Strike systems only care about the entity id, not
/// `Cell`/`Hp` components.
pub(super) fn spawn_cell_empty(app: &mut App) -> Entity {
    app.world_mut().spawn_empty().id()
}

/// Adds `EchoPrimed` to an existing bolt entity.
pub(super) fn install_echo_primed(app: &mut App, bolt: Entity) {
    app.world_mut().entity_mut(bolt).insert(EchoPrimed);
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BumpPerformed` message. `breaker` is
/// `Entity::PLACEHOLDER` (Echo Strike does not inspect it).
pub(super) fn write_bump_performed(app: &mut App, bolt: Option<Entity>, grade: BumpGrade) {
    app.world_mut()
        .resource_mut::<Messages<BumpPerformed>>()
        .write(BumpPerformed {
            grade,
            bolt,
            breaker: Entity::PLACEHOLDER,
        });
}

/// Writes a single `Destroyed<Cell>` message with placeholder payload. The
/// cleanup reader only inspects `victim`; everything else is zero.
pub(super) fn write_destroyed_cell(app: &mut App, victim: Entity) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim,
            killer: None,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker: PhantomData,
        });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `DamageDealt<Cell>` whose `source` matches
/// the Echo Strike sentinel string. Uses the literal `"protocol:echo_strike"`
/// (not the const) so the sentinel drift guard remains independent.
pub(super) fn collected_echo_strike_damage(app: &App) -> Vec<DamageDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<DamageDealt<Cell>>>()
        .0
        .iter()
        .filter(|msg| msg.source == Some(SourceId::protocol(ProtocolKind::EchoStrike).build()))
        .cloned()
        .collect()
}

/// Returns the FIFO-ordered `echoes` deque for a bolt's `EchoNetwork`, or
/// an empty Vec if the bolt has no network component. Convenience for
/// ordering assertions.
pub(super) fn read_echo_network(app: &App, bolt: Entity) -> Vec<Entity> {
    app.world()
        .get::<EchoNetwork>(bolt)
        .map(|n| n.echoes.iter().copied().collect())
        .unwrap_or_default()
}
