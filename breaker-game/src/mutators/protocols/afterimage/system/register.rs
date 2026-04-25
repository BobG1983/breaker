use bevy::prelude::*;

use super::{
    check_phantom_bounce::afterimage_check_phantom_bounce,
    spawn_phantom_bolt::afterimage_spawn_phantom_bolt,
    spawn_phantom_breaker::afterimage_spawn_phantom_breaker,
    tick_phantom_breaker::afterimage_tick_phantom_breaker,
};
use crate::{
    bolt::sets::BoltSystems,
    breaker::sets::BreakerSystems,
    effect_v3::EffectV3Systems,
    mutators::protocols::{definition::ProtocolKind, resources::protocol_active},
    prelude::*,
};

// ── register ────────────────────────────────────────────────────────────────

/// Registers the afterimage runtime systems with the correct schedules,
/// run-ifs, and ordering.
///
/// `FixedUpdate` non-reader systems — gated by
/// `protocol_active(ProtocolKind::Afterimage)` AND
/// `in_state(NodeState::Playing)`, chained in this exact order:
/// 1. `afterimage_tick_phantom_breaker` — ticks lifetimes FIRST so the
///    newly-spawned phantom on step 2 keeps its full lifetime on the spawn
///    tick (otherwise the tick would decrement the freshly-written
///    `PhantomBreakerLifetime` before any system could observe the initial
///    value).
/// 2. `afterimage_spawn_phantom_breaker` — after tick.
///
/// `FixedUpdate` reader systems — intentionally ungated at the registration
/// level. A `.run_if(...)` gate suppresses the system's *execution* but does
/// NOT advance its `MessageReader` cursor, so messages sent during gated-off
/// frames remain in Bevy's two-frame message buffer and get retroactively
/// consumed the tick the gate opens. These systems enforce the
/// `ActiveProtocols` / `NodeState::Playing` gate in-body via an immediate
/// early-return (`reader.clear()` when a reader is present) when inactive:
/// 3. `afterimage_check_phantom_bounce` — after `afterimage_spawn_phantom_breaker`;
///    also `.before(BreakerSystems::GradeBump)` so the synthetic
///    `BumpPerformed` from the bounce is consumed in the same tick, and
///    `.after(BoltSystems::CellCollision)` for collision ordering.
/// 4. `afterimage_spawn_phantom_bolt` — after `afterimage_check_phantom_bounce`;
///    also `.after(BreakerSystems::GradeBump)` AND
///    `.before(EffectV3Systems::Tick)` so the spawned phantom bolt's
///    `PhantomLifetime` is decremented once on the same tick it was spawned
///    (required by test I14).
///
/// No `OnExit(NodeState::Playing)` system is registered. Cleanup is handled
/// by `CleanupOnExit::<NodeState>::default()` tags attached to every
/// afterimage-spawned entity.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            // Tick runs before spawn so the newly-spawned phantom keeps its full lifetime on the spawn tick — otherwise the tick system decrements the freshly-written PhantomBreakerLifetime before the test (or game) can observe the initial value.
            afterimage_tick_phantom_breaker,
            afterimage_spawn_phantom_breaker,
        )
            .chain()
            .run_if(protocol_active(ProtocolKind::Afterimage))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(
        FixedUpdate,
        (
            afterimage_check_phantom_bounce
                .after(afterimage_spawn_phantom_breaker)
                .before(BreakerSystems::GradeBump)
                .after(BoltSystems::CellCollision),
            afterimage_spawn_phantom_bolt
                .after(afterimage_check_phantom_bounce)
                .after(BreakerSystems::GradeBump)
                .before(EffectV3Systems::Tick),
        ),
    );
}
