use bevy::prelude::*;

use super::{
    spawn_phantom_bolt::afterimage_spawn_phantom_bolt,
    spawn_phantom_breaker::afterimage_spawn_phantom_breaker,
};
use crate::{
    breaker::sets::BreakerSystems,
    effect_v3::EffectV3Systems,
    mutators::protocols::{definition::ProtocolKind, resources::protocol_active},
    prelude::*,
};

// ── wire ────────────────────────────────────────────────────────────────

/// Registers the afterimage runtime systems with the correct schedules,
/// run-ifs, and ordering.
///
/// `FixedUpdate` non-reader system — gated by
/// `protocol_active(ProtocolKind::Afterimage)` AND
/// `in_state(NodeState::Playing)`:
/// - `afterimage_spawn_phantom_breaker` — spawns the phantom on the
///   `DashState` rising edge into `Dashing`. Lifetime ticking is owned by
///   the canonical `tick_phantom_breaker_lifespan` system (registered
///   elsewhere) acting on the `Lifespan` component the builder inserts via
///   `.phantom(...)`.
///
/// `FixedUpdate` reader system — intentionally ungated at the registration
/// level. A `.run_if(...)` gate suppresses the system's *execution* but does
/// NOT advance its `MessageReader` cursor, so messages sent during gated-off
/// frames remain in Bevy's two-frame message buffer and get retroactively
/// consumed the tick the gate opens. The system enforces the
/// `ActiveProtocols` / `NodeState::Playing` gate in-body via an immediate
/// early-return (`reader.clear()` when a reader is present) when inactive:
/// - `afterimage_spawn_phantom_bolt` — `.after(BreakerSystems::GradeBump)`.
///   Mutates the real bolt into a phantom via `Bolt::become_phantom` and
///   installs `Lifespan` + `LifetimeEndBehavior::RevertToNormalBolt`.
///   `tick_bolt_lifespan` (bolt-domain) decrements the `Lifespan` each tick
///   and calls `PhantomBolt::become_normal` at expiry to restore the real
///   bolt. The installer pre-subtracts one `delta_secs()` so the
///   same-FixedUpdate deferred-flush gap doesn't grant a free first tick.
///
/// Phantom-bolt-vs-phantom-breaker bounces are now handled by the standard
/// `bolt_breaker_collision` path; no synthetic-bounce system is registered
/// here.
///
/// No `OnExit(NodeState::Playing)` system is registered. Cleanup is handled
/// by `CleanupOnExit::<NodeState>::default()` tags attached to every
/// afterimage-spawned entity.
pub(crate) fn wire(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        afterimage_spawn_phantom_breaker
            .run_if(protocol_active(ProtocolKind::Afterimage))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(
        FixedUpdate,
        afterimage_spawn_phantom_bolt
            .after(BreakerSystems::GradeBump)
            .before(EffectV3Systems::Tick),
    );
}
