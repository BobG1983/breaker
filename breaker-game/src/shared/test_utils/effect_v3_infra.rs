//! Cross-domain test-infrastructure registration for `EffectV3Plugin`.
//!
//! `EffectV3Plugin`'s impact, bump, and bolt-lost bridges consume messages
//! owned by the `bolt`, `breaker`, and `cells` domain plugins. Integration
//! tests that add `EffectV3Plugin` but not those full domain plugins hit
//! Bevy 0.18's system parameter validation ("Message not initialized")
//! unless the message types are initialized directly.

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall, BumpPerformed, BumpWhiffed, NoBump},
    cells::messages::{CellImpactWall, SalvoImpactBreaker},
};

/// Registers every cross-domain message and resource that `EffectV3Plugin`'s
/// systems read but which `EffectV3Plugin` does not register itself.
///
/// Call this **before** adding `EffectV3Plugin`.
pub(crate) fn register_effect_v3_test_infrastructure(app: &mut App) {
    // Impact bridges (`impact/bridges.rs::ImpactReaders`)
    app.add_message::<BoltImpactCell>();
    app.add_message::<BoltImpactWall>();
    app.add_message::<BoltImpactBreaker>();
    app.add_message::<BreakerImpactCell>();
    app.add_message::<BreakerImpactWall>();
    app.add_message::<CellImpactWall>();
    app.add_message::<SalvoImpactBreaker>();

    // Bump bridges (`bump/bridges.rs`)
    app.add_message::<BumpPerformed>();
    app.add_message::<BumpWhiffed>();
    app.add_message::<NoBump>();

    // Bolt-lost bridge (`bolt_lost/bridges.rs`)
    app.add_message::<BoltLost>();
}

#[cfg(test)]
mod tests {
    // Behavior 5 (FAILS at RED): effect_v3_infra.rs must not reference the old
    // monolithic RNG type after Wave 2D migrated tick_chain_lightning +
    // tick_entropy_engine to EffectBaseSeed / EffectEventCounter. The file currently
    // contains five occurrences (doc comments, import, insert_resource call).
    // Writer-code removes all five at GREEN.
    //
    // Self-referential guard: built at compile time to avoid false-positive here.
    const OLD_INFRA_RNG: &str = concat!("Game", "Rng");

    #[test]
    fn effect_v3_infra_rs_has_no_game_rng_reference_after_wave2d_cleanup() {
        let source = include_str!("effect_v3_infra.rs");
        assert!(
            !source.contains(OLD_INFRA_RNG),
            "shared/test_utils/effect_v3_infra.rs must not reference the old monolithic RNG type; \
             remove the import, insert_resource call, and stale doc comments — \
             tick_chain_lightning and tick_entropy_engine no longer need it \
             after Wave 2D migration to EffectBaseSeed/EffectEventCounter"
        );
    }
}
