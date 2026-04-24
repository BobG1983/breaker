//! Cross-domain test-infrastructure registration for `EffectV3Plugin`.
//!
//! `EffectV3Plugin`'s impact, bump, and bolt-lost bridges consume messages
//! owned by the `bolt`, `breaker`, and `cells` domain plugins. Integration
//! tests that add `EffectV3Plugin` but not those full domain plugins hit
//! Bevy 0.18's system parameter validation ("Message not initialized")
//! unless the message types are initialized directly.
//!
//! In addition, `tick_chain_lightning` and `tick_entropy_engine` take
//! `ResMut<GameRng>` but `GameRng` is inserted by the game setup pipeline,
//! not `EffectV3Plugin`. Bevy 0.18 also validates parameter existence and
//! panics with "Resource does not exist" unless the resource is inserted.
//! A deterministic seed (`42`) is used to match the pattern in existing
//! effect tests (see `chain_lightning/systems.rs`, `spawn_bolts/config.rs`,
//! etc.).

use bevy::prelude::*;

use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall, BumpPerformed, BumpWhiffed, NoBump},
    cells::messages::{CellImpactWall, SalvoImpactBreaker},
    shared::rng::GameRng,
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

    // `tick_chain_lightning` + `tick_entropy_engine` require `ResMut<GameRng>`.
    // Deterministic seed `42` matches existing effect tests.
    app.insert_resource(GameRng::from_seed(42));
}
