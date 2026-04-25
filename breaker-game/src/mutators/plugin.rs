//! Mutators domain plugin — protocols and hazards consolidated.
//!
//! Replaces the previous `HazardPlugin` + `ProtocolPlugin` split. The two
//! sub-domains keep their semantic distinction (protocols are positive
//! upgrades, hazards are stacking challenges) but share one plugin so that
//! cross-cutting wiring — most notably the `DmgSystems::MutateDamage` /
//! `DmgSystems::PostApplyDamage` chain participation — can live in a
//! single, central place. Wave 3 will fill in `wire_damage_chain`; until
//! then chain participants register from their per-mechanic `register()`
//! functions exactly as before.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use crate::{
    mutators::{
        hazards::{
            self,
            messages::HazardSelected,
            resources::{ActiveHazards, HazardOffers},
            systems::dispatch_hazard_selection,
        },
        protocols::{
            self,
            fission::FissionCounter,
            greed::GreedStacks,
            messages::ProtocolSelected,
            reckless_dash::RecklessDashDoubledBolts,
            resources::{ActiveProtocols, ProtocolOffer, UnlockedProtocols},
            siphon::SiphonStreak,
            systems::{dispatch_protocol_selection, generate_protocol_offering},
        },
    },
    prelude::*,
    state::run::{chip_select::sets::ChipSelectSystems, hazard_select::sets::HazardSelectSystems},
};

/// Registers all protocol + hazard resources, messages, and systems.
///
/// Internally splits into four `wire_*` functions for readability:
/// - `wire_protocols` — protocol registry/messages/dispatch + per-protocol fan-out
/// - `wire_hazards` — hazard registry/messages/dispatch + per-hazard fan-out
/// - `wire_damage_chain` — central `MutateDamage` / `PostApplyDamage`
///   ordering (Wave 3 fills this in; today empty because chain participants
///   register from their own `register()`s)
/// - `wire_cleanup` — run-end cleanup driver (each mechanic still owns its
///   own `OnExit(RunState::Finished)` registration; this fn stays a stub
///   unless centralisation becomes worthwhile)
pub struct MutatorsPlugin;

impl Plugin for MutatorsPlugin {
    fn build(&self, app: &mut App) {
        wire_protocols(app);
        wire_hazards(app);
        wire_damage_chain(app);
        wire_cleanup(app);
    }
}

fn wire_protocols(app: &mut App) {
    protocols::register(app);
    app.init_resource::<ActiveProtocols>()
        .init_resource::<UnlockedProtocols>()
        .init_resource::<ProtocolOffer>()
        .init_resource::<GreedStacks>()
        .init_resource::<SiphonStreak>()
        .init_resource::<FissionCounter>()
        .init_resource::<RecklessDashDoubledBolts>()
        .add_message::<ProtocolSelected>()
        .add_systems(
            OnEnter(ChipSelectState::Selecting),
            (generate_protocol_offering, ApplyDeferred)
                .chain()
                .after(ChipSelectSystems::GenerateOfferings)
                .before(ChipSelectSystems::SpawnScreen),
        )
        .add_systems(
            Update,
            dispatch_protocol_selection
                .after(ChipSelectSystems::HandleInput)
                .run_if(in_state(ChipSelectState::Selecting)),
        );
}

fn wire_hazards(app: &mut App) {
    app.init_resource::<ActiveHazards>()
        .init_resource::<HazardOffers>()
        .add_message::<HazardSelected>()
        .add_systems(
            Update,
            dispatch_hazard_selection
                .after(HazardSelectSystems::HandleInput)
                .after(HazardSelectSystems::TickTimer)
                .run_if(in_state(HazardSelectState::Selecting)),
        );
    hazards::register(app);
}

const fn wire_damage_chain(_app: &mut App) {
    // Wave 3: central MutateDamage / PostApplyDamage ordering goes here.
    // Today, chain participants still register from per-mechanic
    // `register()` functions — see `mutators::hazards::diffusion::register`,
    // `mutators::hazards::tether::register`,
    // `mutators::protocols::echo_strike::register`. Game-side ordering is
    // currently held by `DmgGameOrderingPlugin`'s `PostApplyRipple` chain
    // in `breaker-game/src/game/system.rs`.
}

const fn wire_cleanup(_app: &mut App) {
    // Each mechanic registers its own `OnExit(RunState::Finished)` cleanup
    // system from its `register(app)`. Centralised dispatch remains
    // unimplemented; revisit only if a cross-mechanic cleanup ordering
    // need surfaces.
}
