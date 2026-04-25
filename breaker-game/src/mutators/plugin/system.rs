//! Mutators domain plugin — protocols and hazards consolidated.
//!
//! Replaces the previous `HazardPlugin` + `ProtocolPlugin` split. The two
//! sub-domains keep their semantic distinction (protocols are positive
//! upgrades, hazards are stacking challenges) but share one plugin so that
//! cross-cutting wiring — most notably the `DmgSystems::MutateDamage` /
//! `DmgSystems::PostApplyDamage` chain participation — can live in a
//! single, central place. Wave 3 will fill in `wire_damage_chain`; until
//! then chain participants wire from their per-mechanic `wire()`
//! functions exactly as before.

use bevy::{ecs::schedule::ApplyDeferred, prelude::*};

use crate::{
    mutators::{
        hazards::{
            self,
            definition::HazardKind,
            diffusion::system::{diffusion_emit_rings, diffusion_reduce_primary},
            messages::HazardSelected,
            resources::{ActiveHazards, HazardOffers, hazard_active},
            systems::dispatch_hazard_selection,
            tether::system::tether_emit_partner,
        },
        protocols::{
            self,
            definition::ProtocolKind,
            echo_strike::system::echo_strike_emit_siblings,
            fission::FissionCounter,
            greed::GreedStacks,
            messages::ProtocolSelected,
            reckless_dash::RecklessDashDoubledBolts,
            resources::{ActiveProtocols, ProtocolOffer, UnlockedProtocols, protocol_active},
            siphon::SiphonStreak,
            systems::{dispatch_protocol_selection, generate_protocol_offering},
        },
    },
    prelude::*,
    state::run::{chip_select::sets::ChipSelectSystems, hazard_select::sets::HazardSelectSystems},
};

/// Registers all protocol + hazard resources, messages, and systems.
///
/// Internally splits into three `wire_*` functions for readability:
/// - `wire_protocols` — protocol registry/messages/dispatch + per-protocol fan-out
/// - `wire_hazards` — hazard registry/messages/dispatch + per-hazard fan-out
/// - `wire_damage_chain` — central `MutateDamage` / `PostApplyDamage`
///   ordering (the single source of truth for the cross-mechanic
///   `diffusion → tether → echo_strike` ripple chain)
///
/// Per-mechanic run-end cleanup remains a per-mechanic responsibility:
/// every `<mechanic>::wire(app)` registers its own
/// `OnExit(NodeState::Playing)` (or equivalent) cleanup system. There
/// is no central `wire_cleanup` driver — centralisation would only
/// justify itself if cross-mechanic cleanup ordering became a concern.
pub struct MutatorsPlugin;

impl Plugin for MutatorsPlugin {
    fn build(&self, app: &mut App) {
        wire_protocols(app);
        wire_hazards(app);
        wire_damage_chain(app);
    }
}

fn wire_protocols(app: &mut App) {
    protocols::wire(app);
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
    hazards::wire(app);
}

/// Central damage-chain assembly — the single source of truth for game-side
/// `MutateDamage` and `PostApplyDamage` ordering.
///
/// **Ordering rationale (`PostApplyDamage` ripple chain):**
///
/// 1. `diffusion_emit_rings` runs FIRST — drains the
///    `PendingDiffusionEmissions` queue populated in `MutateDamage` and
///    emits ring siblings. Running first means downstream emitters see
///    the ring expansion as part of the post-apply state.
/// 2. `tether_emit_partner` runs SECOND — reads `DamageDealt<Cell>`
///    messages and emits partner-target siblings. Running after diffusion
///    means tether sees diffusion's ring siblings as candidates for
///    partner redirection.
/// 3. `echo_strike_emit_siblings` runs LAST — emits echo siblings. Running
///    last means echo picks up everything the chain has produced
///    (primaries, diffusion rings, tether partners).
///
/// `diffusion_reduce_primary` is the only `MutateDamage` participant
/// today; it sits ahead of the entire `PostApplyDamage` ripple chain via
/// the crate's set ordering. Tests gain access via `pub(crate)`.
pub(crate) fn wire_damage_chain(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        diffusion_reduce_primary
            .in_set(DmgSystems::MutateDamage)
            .run_if(in_state(NodeState::Playing))
            .run_if(hazard_active(HazardKind::Diffusion)),
    );
    app.add_systems(
        FixedUpdate,
        (
            diffusion_emit_rings
                .run_if(hazard_active(HazardKind::Diffusion))
                .run_if(in_state(NodeState::Playing)),
            tether_emit_partner
                .run_if(hazard_active(HazardKind::Tether))
                .run_if(in_state(NodeState::Playing)),
            echo_strike_emit_siblings
                .run_if(protocol_active(ProtocolKind::EchoStrike))
                .run_if(in_state(NodeState::Playing)),
        )
            .chain()
            .in_set(DmgSystems::PostApplyDamage),
    );
}
