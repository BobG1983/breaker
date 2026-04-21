//! Conductor protocol — Perfect-bump-driven primary-bolt swap.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/conductor.md`.
//!
//! A Perfect `BumpPerformed` whose bolt carries `ExtraBolt` (verified via the
//! `extras` query) and is not already the current `PrimaryBolt` swaps the
//! `PrimaryBolt` / `ExtraBolt` markers and the `BoundEffects` /
//! `StagedEffects` components between the bumped bolt and the current
//! `PrimaryBolt`. Bare `(Bolt,)` entities without the `ExtraBolt` marker are
//! rejected — the precondition is that the target is an extra bolt.
//!
//! At most one swap per tick: because the marker/component mutations go
//! through `Commands` and only flush at the end of the system, a second
//! Perfect bump in the same tick would observe the stale pre-flush
//! `PrimaryBolt` and erroneously promote a second entity. The system
//! short-circuits subsequent messages in the same tick via a local
//! `swap_done` flag while still advancing the reader cursor, so buffered
//! messages do not replay next tick.
//!
//! No per-node state, no grace window, no cleanup system.

use bevy::prelude::*;

use crate::{
    bolt::components::{ExtraBolt, PrimaryBolt},
    breaker::{messages::BumpGrade, sets::BreakerSystems},
    effect_v3::sets::EffectV3Systems,
    fx::PunchScale,
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

/// Punch-scale overshoot applied to the newly-promoted `PrimaryBolt` on a
/// Conductor swap. 1.25× scale settling to 1.0 over 0.15 s — visible but
/// short enough not to disrupt the 60Hz gameplay loop. Picked to match the
/// breaker-bump punch intensity at roughly half the duration.
const CONDUCTOR_SWAP_PUNCH_OVERSHOOT: f32 = 1.25;
/// Duration of [`CONDUCTOR_SWAP_PUNCH_OVERSHOOT`] in seconds.
const CONDUCTOR_SWAP_PUNCH_DURATION: f32 = 0.15;

// ── ConductorConfig ─────────────────────────────────────────────────────────

/// Presence marker for the `Option<Res<ConductorConfig>>` harness-safety gate
/// in `conductor_swap_on_perfect_bump`. Inserted by `activate` when the
/// matching `ProtocolTuning::Conductor` variant is selected; the system drains
/// its reader and returns when this resource is absent.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConductorConfig;

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts `ConductorConfig` from `ProtocolTuning::Conductor`. Warns and
/// no-ops on a non-`Conductor` tuning variant, leaving any existing
/// `ConductorConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Conductor = *tuning else {
        warn!("conductor::activate called with non-Conductor tuning");
        return;
    };
    commands.insert_resource(ConductorConfig);
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Conductor's single runtime system with the correct schedule,
/// run-ifs, and ordering.
///
/// `FixedUpdate` gated by `protocol_active(ProtocolKind::Conductor)` AND
/// `in_state(NodeState::Playing)`:
/// - `conductor_swap_on_perfect_bump` — after `BreakerSystems::GradeBump`,
///   **before `EffectV3Systems::Bridge`**. The upper bound is mandatory:
///   `grade_bump` is itself registered `.before(EffectV3Systems::Bridge)`
///   (see `breaker-game/src/breaker/plugin.rs`), which means bump bridges
///   such as `on_perfect_bumped` walk the effect tree of the bolt referenced
///   by the same `BumpPerformed` message later in the same tick. Conductor
///   must mutate `PrimaryBolt` / `BoundEffects` / `StagedEffects` **before**
///   that walk runs, otherwise the bridges read the old (pre-swap) effect
///   tree on the bumped bolt. Resulting ordering:
///   `GradeBump → conductor_swap_on_perfect_bump → EffectV3Systems::Bridge`.
///
/// No `OnExit(NodeState::Playing)` system is registered — Conductor has no
/// owned per-node state. `PrimaryBolt` / `ExtraBolt` / `BoundEffects` /
/// `StagedEffects` lifecycle is managed by their respective domains; the
/// swap is a transient component move that leaves invariants intact.
///
/// `register` does NOT call `init_resource`. `ConductorConfig` is installed
/// by `activate`; there is no per-node tracking resource.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        conductor_swap_on_perfect_bump
            .after(BreakerSystems::GradeBump)
            .before(EffectV3Systems::Bridge)
            .run_if(protocol_active(ProtocolKind::Conductor))
            .run_if(in_state(NodeState::Playing)),
    );
}

// ── System — conductor_swap_on_perfect_bump ─────────────────────────────────

/// Swaps `PrimaryBolt`/`ExtraBolt` markers and `BoundEffects`/`StagedEffects`
/// components between the bumped bolt and the current `PrimaryBolt` on every
/// Perfect `BumpPerformed`. Harness-safe: absent `ConductorConfig` drains the
/// reader (matches Reckless Dash / Burnout / Fission precedent) so buffered
/// messages cannot leak into a later frame.
pub(crate) fn conductor_swap_on_perfect_bump(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<ConductorConfig>>,
    primary: Query<Entity, (With<Bolt>, With<PrimaryBolt>)>,
    extras: Query<(), With<ExtraBolt>>,
    effects: Query<(Option<&BoundEffects>, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    if config.is_none() {
        reader.clear();
        return;
    }
    // At most one swap per tick. Subsequent messages in the same tick would
    // observe the stale pre-flush `PrimaryBolt` and erroneously promote a
    // second entity. We still `continue` (rather than `break`) so the reader
    // cursor advances past remaining messages and they don't replay next tick.
    let mut swap_done = false;
    for msg in reader.read() {
        if swap_done {
            continue;
        }
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let Some(bumped) = msg.bolt else { continue };
        if !extras.contains(bumped) {
            continue;
        }
        let Ok(current_primary) = primary.single() else {
            continue;
        };
        if bumped == current_primary {
            continue;
        }
        let Ok((primary_bound, primary_staged)) = effects.get(current_primary) else {
            continue;
        };
        let Ok((bumped_bound, bumped_staged)) = effects.get(bumped) else {
            continue;
        };
        // Clone owned copies before issuing commands — the effects query
        // borrow precludes holding references across the Commands mutation.
        let primary_bound = primary_bound.cloned();
        let primary_staged = primary_staged.cloned();
        let bumped_bound = bumped_bound.cloned();
        let bumped_staged = bumped_staged.cloned();

        commands
            .entity(current_primary)
            .remove::<PrimaryBolt>()
            .insert(ExtraBolt);
        commands
            .entity(bumped)
            .remove::<ExtraBolt>()
            .insert(PrimaryBolt);

        swap_optional_component(
            &mut commands,
            current_primary,
            bumped,
            primary_bound,
            bumped_bound,
        );
        swap_optional_component(
            &mut commands,
            current_primary,
            bumped,
            primary_staged,
            bumped_staged,
        );

        // VFX: punch-scale the newly-promoted primary bolt so the swap reads
        // visually. The `fx` domain's `animate_punch_scale` in `Update` ticks
        // the component back to 1.0× and removes itself. SFX hook pending —
        // `audio::AudioPlugin` is still a Phase-0 stub (breaker-game/src/audio/
        // plugin.rs:12) with no message or resource surface to call into.
        commands.entity(bumped).insert(PunchScale {
            timer:     CONDUCTOR_SWAP_PUNCH_DURATION,
            duration:  CONDUCTOR_SWAP_PUNCH_DURATION,
            overshoot: CONDUCTOR_SWAP_PUNCH_OVERSHOOT,
        });

        swap_done = true;
    }
}

/// Swaps an optional component between two entities.
///
/// `a_val` is the value currently on `a` (already cloned); after the swap it
/// lands on `b`. Symmetrically for `b_val`. `None` on one side means the
/// other side loses the component entirely.
fn swap_optional_component<C: Component + Clone>(
    commands: &mut Commands,
    a: Entity,
    b: Entity,
    a_val: Option<C>,
    b_val: Option<C>,
) {
    commands.entity(a).remove::<C>();
    commands.entity(b).remove::<C>();
    if let Some(v) = b_val {
        commands.entity(a).insert(v);
    }
    if let Some(v) = a_val {
        commands.entity(b).insert(v);
    }
}
