//! Fission protocol — production code.
//!
//! Owns:
//! - [`FissionConfig`] — per-run tuning (inserted by [`activate`]).
//! - [`FissionCounter`] — persistent-across-nodes cell-kill tracker. Init'd by
//!   `ProtocolPlugin::build` (Siphon precedent) and removed by
//!   [`fission_cleanup_run`] on `OnExit(MenuState::Main)`.
//! - [`FISSION_DIVERGENCE_ANGLE_RAD`] — 15° in radians, pinned by drift guard.
//! - [`activate`] — parses `ProtocolTuning::Fission`, inserts `FissionConfig`.
//! - [`register`] — wires `fission_on_cell_destroyed` into `FixedUpdate` and
//!   `fission_cleanup_run` into `OnExit(MenuState::Main)`.
//! - [`fission_on_cell_destroyed`] — counts `Destroyed<Cell>` messages and
//!   spawns a new (`ExtraBolt`, headless) bolt at the parent's position with
//!   velocity rotated by [`FISSION_DIVERGENCE_ANGLE_RAD`] every Nth kill.
//! - [`fission_cleanup_run`] — removes both resources on run-start
//!   (`OnExit(MenuState::Main)`).

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltDefinitionRef, registry::BoltRegistry},
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Constants ───────────────────────────────────────────────────────────────

/// Clockwise rotation (radians) applied to the parent bolt's velocity to
/// produce the new bolt's velocity on split. Pinned at 15 degrees.
///
/// Design doc: "small angle (e.g., 15-20 degrees). Exact angle is a tuning
/// value but not in RON config for now — hardcoded as a small constant."
pub(crate) const FISSION_DIVERGENCE_ANGLE_RAD: f32 = 15.0_f32 * std::f32::consts::PI / 180.0;

// ── FissionConfig ───────────────────────────────────────────────────────────

/// Per-run Fission tuning extracted from [`ProtocolTuning::Fission`] at
/// activation time. Inserted by [`activate`], removed by
/// [`fission_cleanup_run`] on `OnExit(MenuState::Main)`.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FissionConfig {
    /// Cell kills required to trigger a split.
    pub(crate) kills_per_split: u32,
}

// ── FissionCounter ──────────────────────────────────────────────────────────

/// Persistent-across-nodes cell-kill tracker for the Fission protocol.
///
/// Init'd by `ProtocolPlugin::build` (matches Siphon precedent — the plugin
/// owns `init_resource`, not `register`). Removed on `OnExit(MenuState::Main)`
/// by [`fission_cleanup_run`]. NOT reset on node exit — persists across nodes
/// per design.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FissionCounter {
    /// Cell kills accumulated since the last split (or run start).
    pub(crate) kills: u32,
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts [`FissionConfig`] from `ProtocolTuning::Fission`. Called by the
/// parent `protocols::activate` dispatch on Fission pick; last write wins.
/// Warns and no-ops on a non-Fission tuning variant, leaving any existing
/// `FissionConfig` intact.
///
/// Does NOT touch `FissionCounter` — the counter is plugin-initialised and
/// persists across activations (re-activating mid-run must not reset progress).
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Fission { kills_per_split } = *tuning else {
        warn!("fission::activate called with non-Fission tuning");
        return;
    };
    commands.insert_resource(FissionConfig { kills_per_split });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Fission's runtime systems.
///
/// - `fission_on_cell_destroyed` → `FixedUpdate`, ordered
///   `.after(DeathPipelineSystems::HandleKill)`, gated by
///   `protocol_active(Fission)` + `in_state(NodeState::Playing)`.
/// - `fission_cleanup_run` → `OnExit(MenuState::Main)` with NO run-if.
///
/// NOTE: `FissionCounter` is NOT inserted here; `ProtocolPlugin::build` owns
/// `init_resource::<FissionCounter>()` (Siphon precedent).
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        fission_on_cell_destroyed
            .after(DeathPipelineSystems::HandleKill)
            .run_if(protocol_active(ProtocolKind::Fission))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(OnExit(MenuState::Main), fission_cleanup_run);
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Query alias for all live bolt entities with the components Fission needs.
/// `Without<Dead>` excludes bolts in the death pipeline so they aren't picked
/// as a split parent. Factored out to satisfy `clippy::type_complexity`.
type FissionBoltQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static Velocity2D,
        &'static BoltDefinitionRef,
        Option<&'static BoundEffects>,
        Option<&'static StagedEffects>,
    ),
    (With<Bolt>, Without<Dead>),
>;

/// Consumes `Destroyed<Cell>` messages and maintains [`FissionCounter`];
/// spawns a new bolt whenever the counter reaches
/// `FissionConfig.kills_per_split`.
///
/// Per-message flow:
/// 1. Increment `counter.kills` by 1 (saturating).
/// 2. If still below `config.kills_per_split`, continue.
/// 3. Threshold reached: reset `counter.kills` to 0.
/// 4. Pick the parent bolt — `msg.killer` if it resolves to a live bolt in
///    the query, else the first bolt the query yields. If no bolt is alive,
///    skip the spawn (counter already reset).
/// 5. Look up the parent's `BoltDefinition` in `BoltRegistry` via
///    `BoltDefinitionRef.0`. If absent, log a warning and skip.
/// 6. Compute the new bolt's velocity by rotating the parent's velocity
///    clockwise by [`FISSION_DIVERGENCE_ANGLE_RAD`] via
///    [`Velocity2D::rotate_by`] (preserves magnitude).
/// 7. Spawn the new bolt at the parent's position with role `.extra()` and
///    visual `.headless()`. The single-`PrimaryBolt` invariant is preserved.
/// 8. Clone parent's `BoundEffects` / `StagedEffects` (if present) onto the
///    new bolt so the split inherits parent state.
///
/// Harness-safe: if `FissionConfig`, `FissionCounter`, or `BoltRegistry` is
/// absent, the reader is drained and the system returns without inserting any
/// resource as a side effect.
pub(crate) fn fission_on_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    config: Option<Res<FissionConfig>>,
    counter: Option<ResMut<FissionCounter>>,
    bolts: FissionBoltQuery,
    registry: Option<Res<BoltRegistry>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    let Some(mut counter) = counter else {
        reader.clear();
        return;
    };
    let Some(registry) = registry else {
        reader.clear();
        return;
    };

    for msg in reader.read() {
        counter.kills = counter.kills.saturating_add(1);
        if counter.kills < config.kills_per_split {
            continue;
        }

        // Threshold hit — reset counter regardless of whether a split spawns.
        counter.kills = 0;

        // Pick parent: msg.killer if it matches a live bolt, else first bolt.
        let parent = msg
            .killer
            .and_then(|k| bolts.get(k).ok())
            .or_else(|| bolts.iter().next());

        let Some((_, parent_pos, parent_vel, parent_def_ref, parent_bound, parent_staged)) = parent
        else {
            // No live bolt — skip split. Counter already reset.
            continue;
        };

        let Some(bolt_def) = registry.get(&parent_def_ref.0) else {
            warn!(
                "fission: bolt definition '{}' not found in BoltRegistry — skipping split",
                parent_def_ref.0
            );
            continue;
        };

        // Clockwise rotation by FISSION_DIVERGENCE_ANGLE_RAD; magnitude
        // preserved by `Velocity2D::rotate_by`.
        let new_velocity = parent_vel.rotate_by(FISSION_DIVERGENCE_ANGLE_RAD);

        let new_bolt = Bolt::builder()
            .at_position(parent_pos.0)
            .definition(bolt_def)
            .with_velocity(new_velocity)
            .extra()
            .headless()
            .spawn(&mut commands);

        // Clone BoundEffects / StagedEffects onto the new bolt after spawn.
        // The builder's `.with_inherited_effects(..)` is BoundEffects-only and
        // requires `&BoundEffects`; for symmetry with StagedEffects (which has
        // no builder entry point), insert both directly here.
        if let Some(bound) = parent_bound {
            commands.entity(new_bolt).insert(bound.clone());
        }
        if let Some(staged) = parent_staged {
            commands.entity(new_bolt).insert(staged.clone());
        }
    }
}

/// Removes `FissionConfig` and `FissionCounter` on `OnExit(MenuState::Main)`
/// — i.e., at run-start.
///
/// Harness-safe: `remove_resource` on an absent resource is a no-op.
pub(crate) fn fission_cleanup_run(mut commands: Commands) {
    commands.remove_resource::<FissionConfig>();
    commands.remove_resource::<FissionCounter>();
}
