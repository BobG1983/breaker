//! System to reset run state at the start of a new run.

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

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
    prelude::*,
    shared::{
        RunSeed,
        rng::{ChipSelectCount, EffectBaseSeed, derive_seed_named},
    },
    state::run::resources::{HighlightTracker, NodeOutcome, RunProgress, TierConfig},
};

/// Per-run inventories cleared on each new run. Grouped by **lifecycle**
/// (cleared on every new run), not by domain — callers outside
/// [`reset_run_state`] should read the individual resources directly.
/// Bundled here so [`reset_run_state`] stays within the system-parameter
/// limit.
#[derive(SystemParam)]
pub(crate) struct RunInventories<'w> {
    chips:                   ResMut<'w, ChipInventory>,
    protocols:               ResMut<'w, ActiveProtocols>,
    hazards:                 ResMut<'w, ActiveHazards>,
    offer:                   ResMut<'w, ProtocolOffer>,
    greed_stacks:            ResMut<'w, GreedStacks>,
    siphon_streak:           ResMut<'w, SiphonStreak>,
    protocol_offering_count: ResMut<'w, ProtocolOfferingCount>,
    chip_select_count:       ResMut<'w, ChipSelectCount>,
}

impl RunInventories<'_> {
    fn clear_all(&mut self) {
        self.chips.clear();
        self.protocols.clear();
        self.hazards.clear();
        *self.offer = ProtocolOffer::default();
        *self.greed_stacks = GreedStacks::default();
        *self.siphon_streak = SiphonStreak::default();
        *self.protocol_offering_count = ProtocolOfferingCount::default();
        *self.chip_select_count = ChipSelectCount::default();
    }
}

/// [`SystemParam`] bundle of per-run state resources cleared on every new
/// run: [`HighlightTracker`], `ResMut<RunProgress>`, and `ResMut<TierConfig>`.
/// Used by [`reset_run_state`] to reset all three with a single param, keeping
/// the system within clippy's argument-count threshold.
#[derive(SystemParam)]
pub(crate) struct TierProgress<'w> {
    highlight_tracker: ResMut<'w, HighlightTracker>,
    run_progress:      ResMut<'w, RunProgress>,
    tier_config:       ResMut<'w, TierConfig>,
}

impl TierProgress<'_> {
    fn reset(&mut self) {
        *self.highlight_tracker = HighlightTracker::default();
        *self.run_progress = RunProgress::default();
        *self.tier_config = TierConfig::default();
    }
}

/// Resets [`NodeOutcome`] to defaults, reseeds [`GameRng`], and derives
/// [`EffectBaseSeed`] from the captured run seed when leaving the main menu
/// (starting a run). The effect base seed is set once per run; ephemeral
/// per-effect-fire RNG is derived from it in Wave 2D.
pub(crate) fn reset_run_state(
    mut run_state: ResMut<NodeOutcome>,
    mut rng: ResMut<GameRng>,
    seed: Res<RunSeed>,
    mut stats: ResMut<RunStats>,
    mut effect_base_seed: ResMut<EffectBaseSeed>,
    mut tier_progress: TierProgress,
    mut inventories: RunInventories,
) {
    *run_state = NodeOutcome::default();
    *stats = RunStats::default();
    tier_progress.reset();
    inventories.clear_all();

    let run_seed_value: u64 = if let Some(s) = seed.0 {
        *rng = GameRng::from_seed(s);
        s
    } else {
        let mut fresh = ChaCha8Rng::from_os_rng();
        let drawn: u64 = fresh.random();
        rng.0 = ChaCha8Rng::seed_from_u64(drawn);
        drawn
    };

    *effect_base_seed = EffectBaseSeed(derive_seed_named(run_seed_value, "effect"));
}
