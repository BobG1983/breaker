//! Burnout — System 5: remove all per-node Burnout state on
//! `OnExit(NodeState::Playing)`.

use bevy::prelude::*;

use super::{
    on_bump::BurnoutDamageBoost,
    update_heat::{BurnoutHeat, BurnoutSpeedBoost},
};

// ── System 5 — burnout_cleanup_node ─────────────────────────────────────────

type BurnoutCleanupBreakerQuery<'w, 's> =
    Query<'w, 's, Entity, Or<(With<BurnoutHeat>, With<BurnoutSpeedBoost>)>>;

type BurnoutCleanupBoltQuery<'w, 's> = Query<'w, 's, Entity, With<BurnoutDamageBoost>>;

/// Runs on `OnExit(NodeState::Playing)`. Removes `BurnoutHeat`,
/// `BurnoutSpeedBoost`, and `BurnoutDamageBoost` from every entity that
/// carries them, guaranteeing per-node state does not leak across nodes /
/// runs. Runs unconditionally (no `run_if`).
pub(crate) fn burnout_cleanup_node(
    mut commands: Commands,
    breakers: BurnoutCleanupBreakerQuery,
    bolts: BurnoutCleanupBoltQuery,
) {
    for entity in &breakers {
        commands
            .entity(entity)
            .remove::<BurnoutHeat>()
            .remove::<BurnoutSpeedBoost>();
    }
    for entity in &bolts {
        commands.entity(entity).remove::<BurnoutDamageBoost>();
    }
}
