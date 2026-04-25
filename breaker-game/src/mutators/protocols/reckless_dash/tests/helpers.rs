//! Shared test fixtures for Reckless Dash protocol tests.
//!
//! App builders, canonical `RecklessDashConfig`, breaker/bolt/cell spawners,
//! `BumpPerformed` / `BoltImpactCell` / `BoltLost` message writers, captured
//! `BoltLost` and sentinel-filtered `DamageDealt<Cell>` collectors, and the
//! `activate_now` CommandQueue-flush helper.
//!
//! Canonical config is `risky_zone_start: 0.7, damage_multiplier: 4.0,
//! double_penalty: true` — the design-doc worked example. The RON asset's
//! current `risky_zone_start: 0.3` is intentionally mismatched; the
//! `ron_asset.rs` drift guard pins the correct design-intent value 0.7 and
//! will FAIL until writer-code updates the RON.

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{
    RecklessDashConfig, RecklessDashDoubledBolts, RiskyDamageBoost, activate, wire,
};
use crate::{
    bolt::components::BoltBaseDamage,
    breaker::{
        components::{DashDuration, DashState, DashStateTimer},
        messages::BumpGrade,
    },
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Reckless Dash test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` initialised, reader messages registered, `BoltLost` and
/// `DamageDealt<Cell>` capture installed, canonical `RecklessDashConfig`
/// inserted, `RecklessDashDoubledBolts` init'd (plugin owns init — helpers
/// mirror the Greed / Siphon / Fission pattern), and `wire` called.
///
/// Does NOT seed `ActiveProtocols` with Reckless Dash — tests that need the
/// protocol active call [`seed_active_protocols_with_reckless_dash`].
pub(super) fn build_reckless_dash_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_message_capture::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_reckless_dash_config());
    app.world_mut().init_resource::<RecklessDashDoubledBolts>();
    wire(&mut app);
    app
}

/// Same as [`build_reckless_dash_app`] but omits `RecklessDashConfig`. Used
/// to exercise the harness-safe `Option<Res<RecklessDashConfig>>` early-return
/// guard paths in each of the three reader systems.
pub(super) fn build_reckless_dash_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_message_capture::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut().init_resource::<RecklessDashDoubledBolts>();
    wire(&mut app);
    app
}

/// Same as [`build_reckless_dash_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for `in_state(NodeState::Playing)`
/// gate tests.
pub(super) fn build_reckless_dash_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_message_capture::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_reckless_dash_config());
    app.world_mut().init_resource::<RecklessDashDoubledBolts>();
    wire(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Reckless Dash config used across all system-behavior tests.
/// Matches the design-doc: `risky_zone_start: 0.7, damage_multiplier: 4.0,
/// double_penalty: true`.
///
/// NOTE: The RON asset is expected to hold `risky_zone_start: 0.7` after the
/// implementation spec updates it from the shipped `0.3`. The `ron_asset.rs`
/// drift guard pins 0.7.
pub(super) const fn canonical_reckless_dash_config() -> RecklessDashConfig {
    RecklessDashConfig {
        risky_zone_start:  0.7,
        damage_multiplier: 4.0,
        double_penalty:    true,
    }
}

/// Inserts the given `RecklessDashConfig` as a resource.
pub(super) fn install_reckless_dash_config(app: &mut App, cfg: RecklessDashConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts a Reckless Dash `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(RecklessDash)` run-condition passes.
pub(super) fn seed_active_protocols_with_reckless_dash(
    app: &mut App,
    risky_zone_start: f32,
    damage_multiplier: f32,
    double_penalty: bool,
) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Reckless Dash".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::RecklessDash {
                risky_zone_start,
                damage_multiplier,
                double_penalty,
            },
        });
}

/// Invokes `reckless_dash::activate` directly via a fresh `CommandQueue` so
/// each call has an independent, deterministic flush. Mirrors the Echo Strike
/// / Iron Curtain / Greed `activate_now` helpers.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Entity spawners ─────────────────────────────────────────────────────────

/// Spawns `(Breaker, DashState::Dashing, DashDuration(dash_duration),
/// DashStateTimer { remaining })`. Progress is `(dash_duration - remaining) /
/// dash_duration`.
pub(super) fn spawn_breaker_dashing(app: &mut App, dash_duration: f32, remaining: f32) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            DashState::Dashing,
            DashDuration(dash_duration),
            DashStateTimer { remaining },
        ))
        .id()
}

/// Spawns `(Breaker, state, DashDuration(dash_duration), DashStateTimer {
/// remaining })`. Used to exercise non-Dashing states (Idle / Braking /
/// Settling).
pub(super) fn spawn_breaker_in_state(
    app: &mut App,
    state: DashState,
    dash_duration: f32,
    remaining: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            state,
            DashDuration(dash_duration),
            DashStateTimer { remaining },
        ))
        .id()
}

/// Spawns `(Bolt, BoltBaseDamage(base))`.
pub(super) fn spawn_bolt_with_base_damage(app: &mut App, base: f32) -> Entity {
    app.world_mut().spawn((Bolt, BoltBaseDamage(base))).id()
}

/// Spawns a bare empty entity — tests reuse this as a "cell" stand-in.
pub(super) fn spawn_cell_empty(app: &mut App) -> Entity {
    app.world_mut().spawn_empty().id()
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BumpPerformed` message. `breaker` is forwarded verbatim
/// because `reckless_dash_on_bump` must look up the breaker's dash state by
/// `msg.breaker`.
pub(super) fn write_bump_performed(
    app: &mut App,
    breaker: Entity,
    bolt: Option<Entity>,
    grade: BumpGrade,
) {
    app.world_mut()
        .resource_mut::<Messages<BumpPerformed>>()
        .write(BumpPerformed {
            grade,
            bolt,
            breaker,
        });
}

/// Writes a single `BoltImpactCell` message with placeholder `impact_normal`
/// and zero `piercing_remaining` (Reckless Dash does not inspect either).
pub(super) fn write_bolt_impact_cell(app: &mut App, cell: Entity, bolt: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltImpactCell>>()
        .write(BoltImpactCell {
            cell,
            bolt,
            impact_normal: Vec2::ZERO,
            piercing_remaining: 0,
        });
}

/// Writes a single `BoltLost { bolt, breaker }` message.
pub(super) fn write_bolt_lost(app: &mut App, bolt: Entity, breaker: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost { bolt, breaker });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `DamageDealt<Cell>` whose `source` matches the
/// builder-produced `protocol:reckless_dash` source. Construction goes
/// through the builder so the helper fails by construction if `kind_slug()`
/// drifts.
pub(super) fn collected_reckless_dash_damage(app: &App) -> Vec<DamageDealt<Cell>> {
    let rd_source = SourceId::protocol(ProtocolKind::RecklessDash).build();
    app.world()
        .resource::<MessageCollector<DamageDealt<Cell>>>()
        .0
        .iter()
        .filter(|msg| msg.source.as_ref() == Some(&rd_source))
        .cloned()
        .collect()
}

/// Returns all captured `BoltLost` messages (originals + any duplicates
/// emitted by `reckless_dash_double_penalty`).
pub(super) fn captured_bolt_lost(app: &App) -> Vec<BoltLost> {
    app.world()
        .resource::<MessageCollector<BoltLost>>()
        .0
        .clone()
}

/// Returns `Some(boost.multiplier)` if the bolt carries `RiskyDamageBoost`,
/// else `None`. Convenience for asserting on the inserted boost.
pub(super) fn risky_boost(app: &App, bolt: Entity) -> Option<f32> {
    app.world()
        .get::<RiskyDamageBoost>(bolt)
        .map(|b| b.multiplier)
}
