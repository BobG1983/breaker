//! Run plugin registration.

use bevy::prelude::*;

use super::{
    chip_select::systems::{
        detect_first_evolution, snapshot_node_highlights, track_chips_collected,
    },
    definition::HighlightConfig,
    loading::systems::{capture_run_seed, generate_node_sequence_system, reset_run_state},
    messages::{HighlightTriggered, RunLost},
    node::{
        NodePlugin, NodeSystems,
        highlights::systems::{
            detect_close_save, detect_combo_king, detect_mass_destruction, detect_nail_biter,
            detect_pinball_wizard,
        },
        lifecycle::systems::{
            handle_breaker_death, handle_node_cleared, handle_run_lost, handle_timer_expired,
            reset_highlight_tracker, spawn_highlight_text,
        },
        systems::reset_bolt,
        tracking::systems::{
            track_bolts_lost, track_bumps, track_cells_destroyed, track_evolution_damage,
            track_node_cleared_stats, track_time_elapsed,
        },
    },
    resources::{DifficultyCurve, HighlightTracker, NodeOutcome},
    run_end::systems::detect_most_powerful_evolution,
    systems::{advance_node, hide_gameplay_entities, setup_run, show_gameplay_entities},
};
use crate::{
    bolt::BoltSystems,
    breaker::BreakerSystems,
    prelude::*,
    shared::{
        RunSeed,
        rng::{EffectBaseSeed, FxRng, NodeSequenceRng, SeedNodeRngSystems, seed_node_rng},
    },
};

/// Ordering set for systems that run in `OnExit(MenuState::Main)` at run start.
///
/// Writer-code wires `ResetState → CaptureSeed → GenerateSequence`. The stub
/// variants exist so RED-phase tests compile; wiring is added in Wave 2E GREEN.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunStartSystems {
    /// Resets run state before seeding begins.
    ResetState,
    /// Captures the run seed for downstream seeding.
    CaptureSeed,
    /// Generates the node sequence from the captured seed.
    GenerateSequence,
}

/// Plugin for the run domain.
///
/// Owns run state, node sequencing, and delegates node internals to [`NodePlugin`].
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NodeOutcome>()
            .init_resource::<DifficultyCurve>()
            .init_resource::<GameRng>()
            .init_resource::<NodeSequenceRng>()
            .init_resource::<FxRng>()
            .init_resource::<EffectBaseSeed>()
            .init_resource::<RunSeed>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightConfig>()
            .init_resource::<HighlightTracker>()
            .add_plugins(NodePlugin)
            .add_message::<RunLost>()
            .add_message::<HighlightTriggered>()
            // Breaker death handler — runs AFTER `EmitKill` and BEFORE
            // `ApplyKill`. This ordering matters: handle_breaker_death
            // inserts `Dead` on the breaker via Commands; the command flush
            // point between sets commits the marker before the crate's
            // generic `handle_kill::<Breaker>` (also in `ApplyKill`) runs.
            // Its victim query `With<Breaker>, Without<Dead>` then skips the
            // breaker, preventing the double `Destroyed<Breaker>` emit and
            // avoiding the generic `DespawnEntity` that would otherwise
            // despawn the breaker before the end-of-run animation plays.
            .add_systems(
                FixedUpdate,
                handle_breaker_death
                    .after(DmgSystems::EmitKill)
                    .before(DmgSystems::ApplyKill),
            )
            .add_systems(
                FixedUpdate,
                (
                    handle_node_cleared.after(NodeSystems::TrackCompletion),
                    handle_timer_expired
                        .after(NodeSystems::ReduceNodeTimer)
                        .after(handle_node_cleared),
                    handle_run_lost
                        .after(handle_node_cleared)
                        .after(handle_timer_expired),
                    // Stats accumulation (passive message readers)
                    track_cells_destroyed.after(DmgSystems::ApplyKill),
                    track_bumps,
                    track_bolts_lost,
                    track_time_elapsed,
                    track_evolution_damage.after(DmgSystems::ApplyDamage),
                    track_node_cleared_stats.after(NodeSystems::TrackCompletion),
                    // Highlight detection
                    detect_mass_destruction.after(DmgSystems::ApplyKill),
                    detect_close_save.after(crate::breaker::BreakerSystems::GradeBump),
                    detect_combo_king.after(DmgSystems::ApplyKill),
                    detect_pinball_wizard,
                    detect_nail_biter.after(NodeSystems::TrackCompletion),
                )
                    .run_if(in_state(NodeState::Playing)),
            )
            // In-game highlight juice (Update, NodeState::Playing)
            .add_systems(
                Update,
                spawn_highlight_text.run_if(in_state(NodeState::Playing)),
            )
            // Chip selection tracking + evolution detection (Update, ChipSelect state)
            .add_systems(
                Update,
                (
                    track_chips_collected,
                    detect_first_evolution,
                    snapshot_node_highlights,
                )
                    .run_if(in_state(ChipSelectState::Selecting)),
            )
            .add_systems(
                OnEnter(NodeState::Loading),
                (
                    reset_highlight_tracker,
                    seed_node_rng
                        .in_set(SeedNodeRngSystems::Seed)
                        .after(SeedNodeRngSystems::PrepareSeed),
                    setup_run
                        .in_set(SeedNodeRngSystems::ConsumeSeed)
                        .after(SeedNodeRngSystems::Seed),
                    reset_bolt
                        .after(BreakerSystems::Reset)
                        .in_set(BoltSystems::Reset)
                        .in_set(SeedNodeRngSystems::ConsumeSeed)
                        .after(SeedNodeRngSystems::Seed),
                ),
            )
            .add_systems(OnEnter(RunEndState::Active), detect_most_powerful_evolution)
            .add_systems(OnExit(RunState::Node), hide_gameplay_entities)
            .add_systems(
                OnEnter(RunState::Node),
                (
                    advance_node.in_set(NodeSystems::AdvanceNode),
                    show_gameplay_entities,
                ),
            )
            .add_systems(
                OnExit(MenuState::Main),
                (
                    reset_run_state.in_set(RunStartSystems::ResetState),
                    capture_run_seed
                        .in_set(RunStartSystems::CaptureSeed)
                        .after(RunStartSystems::ResetState),
                    generate_node_sequence_system
                        .in_set(RunStartSystems::GenerateSequence)
                        .after(RunStartSystems::CaptureSeed),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_stateflow::RoutingTable;

    use super::RunPlugin;
    use crate::{
        chips::inventory::ChipInventory,
        mutators::{
            hazards::resources::ActiveHazards,
            protocols::{
                greed::GreedStacks,
                resources::{ActiveProtocols, ProtocolOffer, ProtocolOfferingCount},
                siphon::SiphonStreak,
            },
        },
        prelude::{AppState, GameState, MenuState, NodeState},
        shared::{rng::ChipSelectCount, test_utils::builder::TestAppBuilder},
        state::run::NodeLayoutRegistry,
    };

    // Regression test: RunPlugin must init_resource::<EffectBaseSeed>() so that
    // reset_run_state (registered on OnExit(MenuState::Main)) can resolve its
    // ResMut<EffectBaseSeed> system parameter.
    //
    // FAILS at RED: RunPlugin does not init EffectBaseSeed, so the system param
    // validation panics with "Resource does not exist" on the first menu exit.
    // Writer-code adds the missing init_resource at GREEN.
    #[test]
    fn run_plugin_inits_effect_base_seed_for_real_menu_exit() {
        let mut app = TestAppBuilder::new().with_state_hierarchy().build();

        // Init all RunInventories resources that reset_run_state requires —
        // intentionally EXCLUDING EffectBaseSeed to expose the missing init_resource.
        app.init_resource::<ChipInventory>();
        app.init_resource::<ActiveProtocols>();
        app.init_resource::<ActiveHazards>();
        app.init_resource::<ProtocolOffer>();
        app.init_resource::<GreedStacks>();
        app.init_resource::<SiphonStreak>();
        app.init_resource::<ProtocolOfferingCount>();
        app.init_resource::<ChipSelectCount>();
        app.init_resource::<RoutingTable<NodeState>>();
        app.init_resource::<NodeLayoutRegistry>();

        app.add_plugins(RunPlugin);

        // Drive into GameState::Menu (default sub-state of AppState::Game).
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        app.update();
        // Now drive MenuState from Loading → Main
        app.world_mut()
            .resource_mut::<NextState<MenuState>>()
            .set(MenuState::Main);
        app.update();

        // Exit MenuState::Main → triggers OnExit(MenuState::Main) → reset_run_state fires.
        // Without RunPlugin::build calling init_resource::<EffectBaseSeed>(), this panics.
        app.world_mut()
            .resource_mut::<NextState<MenuState>>()
            .set(MenuState::Teardown);
        app.update();

        // If we reach here without panic, EffectBaseSeed was initialized by RunPlugin.
        assert!(
            app.world()
                .get_resource::<crate::shared::rng::EffectBaseSeed>()
                .is_some(),
            "EffectBaseSeed must be initialized by RunPlugin::build — \
             add .init_resource::<EffectBaseSeed>() to the init block"
        );
    }
}
