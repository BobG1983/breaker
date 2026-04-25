//! Burnout — system registration: wires the 5 sub-systems into their
//! schedules and ordering.

use bevy::prelude::*;

use super::{
    amplify::burnout_amplify_damage, cleanup_node::burnout_cleanup_node, on_bump::burnout_on_bump,
    tick_speed_boost::burnout_tick_speed_boost, update_heat::burnout_update_heat,
};
use crate::{
    bolt::sets::BoltSystems,
    breaker::sets::BreakerSystems,
    effect_v3::EffectV3Systems,
    prelude::*,
    protocol::{definition::ProtocolKind, resources::protocol_active},
};

// ── register ────────────────────────────────────────────────────────────────

/// Registers Burnout's runtime systems with the correct schedules, run-ifs,
/// and ordering.
///
/// `FixedUpdate` non-reader systems — gated by
/// `protocol_active(ProtocolKind::Burnout)` AND `in_state(NodeState::Playing)`:
/// - `burnout_tick_speed_boost` — before `BreakerSystems::Move`.
/// - `burnout_update_heat` — after `BreakerSystems::Move`, before
///   `burnout_on_bump`.
///
/// `FixedUpdate` reader systems — intentionally ungated at the tuple level.
/// A `.run_if(...)` gate suppresses the system's *execution* but does NOT
/// advance its `MessageReader` cursor, so messages sent during gated-off
/// frames remain in Bevy's two-frame message buffer and get retroactively
/// consumed the tick the gate opens. Reader systems instead enforce the
/// `ActiveProtocols` / `NodeState::Playing` gate in-body via an immediate
/// `reader.clear()` + return when inactive, draining the buffer every tick:
/// - `burnout_on_bump` — after `BreakerSystems::GradeBump`, after
///   `burnout_update_heat`.
/// - `burnout_amplify_damage` — after `BoltSystems::CellCollision`.
///
/// The explicit `burnout_update_heat → burnout_on_bump` edge is required:
/// `BreakerSystems::Move` and `BreakerSystems::GradeBump` have no relative
/// ordering (both only `.after(update_bump)`), so without this edge the
/// scheduler may place `on_bump` before `update_heat`. In that ordering a
/// stationary-breaker still-timer tick after `on_bump` resets it would
/// advance `still_timer` past 0.0 in the same tick the charge was consumed,
/// breaking the "reset to default" contract. Running `update_heat` first
/// establishes the current-tick heat state; then `on_bump` decides whether
/// to consume `mega_bump_charged` and reset the gauge.
///
/// `OnExit(NodeState::Playing)` (no run-if):
/// - `burnout_cleanup_node` — removes `BurnoutHeat`, `BurnoutSpeedBoost`, and
///   `BurnoutDamageBoost` from every entity that carries them.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            burnout_tick_speed_boost.before(BreakerSystems::Move),
            burnout_update_heat
                .after(BreakerSystems::Move)
                .before(burnout_on_bump),
        )
            .run_if(protocol_active(ProtocolKind::Burnout))
            .run_if(in_state(NodeState::Playing)),
    )
    .add_systems(
        FixedUpdate,
        (
            burnout_on_bump.after(BreakerSystems::GradeBump),
            burnout_amplify_damage
                .after(BoltSystems::CellCollision)
                .before(EffectV3Systems::Bridge),
        ),
    )
    .add_systems(OnExit(NodeState::Playing), burnout_cleanup_node);
}
