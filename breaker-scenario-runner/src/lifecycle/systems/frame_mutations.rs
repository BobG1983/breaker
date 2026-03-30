//! Per-frame debug mutations: breaker state, timer, entity spawning, pause, run stats, chip injection.

use std::collections::HashSet;

use bevy::{ecs::system::SystemParam, prelude::*};
use breaker::{
    breaker::components::BreakerState,
    chips::inventory::ChipInventory,
    effect::{
        EffectiveSpeedMultiplier,
        effects::{
            chain_lightning::{ChainLightningArc, ChainLightningChain, ChainState},
            pulse::PulseRing,
            second_wind::SecondWindWall,
            shield::ShieldActive,
            speed_boost::ActiveSpeedBoosts,
        },
    },
    run::{RunStats, node::resources::NodeTimer},
    screen::chip_select::{ChipOffering, ChipOffers},
    shared::{CleanupOnNodeExit, PlayingState},
};
use rantzsoft_spatial2d::components::Position2D;

use super::{entity_tagging::map_scenario_breaker_state, types::ScenarioConfig};
use crate::{
    invariants::{ScenarioFrame, ScenarioTagBolt, ScenarioTagBreaker},
    types::{MutationKind, RunStatCounter},
};

/// Grouped system parameters for pause toggle control.
///
/// Extracted to keep [`apply_debug_frame_mutations`] under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct PauseControl<'w> {
    /// Current [`PlayingState`] -- only present when [`GameState::Playing`] is active.
    state: Option<Res<'w, State<PlayingState>>>,
    /// [`NextState`] writer for toggling pause.
    next: Option<ResMut<'w, NextState<PlayingState>>>,
}

/// Grouped system parameters for mutation targets that need additional game state.
///
/// Extracted to keep [`apply_debug_frame_mutations`] under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct MutationTargets<'w, 's> {
    /// [`RunStats`] resource -- absent before a run starts.
    run_stats: Option<ResMut<'w, RunStats>>,
    /// [`ChipInventory`] resource -- absent before a run starts.
    chip_inventory: Option<ResMut<'w, ChipInventory>>,
    /// [`ChipOffers`] resource -- present only during [`GameState::ChipSelect`].
    chip_offers: Option<ResMut<'w, ChipOffers>>,
    /// [`Commands`] for inserting resources when the optional resource is absent.
    commands: Commands<'w, 's>,
}

/// Applies per-frame mutations from [`ScenarioConfig`] at matching frames.
///
/// Reads `frame_mutations` from the scenario definition. For each mutation
/// whose frame matches [`ScenarioFrame`], applies the corresponding state
/// change (breaker state override, timer override, entity spawn, bolt
/// teleport, pause toggle, run stat decrement, or chip inventory injection).
pub fn apply_debug_frame_mutations(
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut breakers: Query<&mut BreakerState, With<ScenarioTagBreaker>>,
    mut bolts: Query<&mut Position2D, With<ScenarioTagBolt>>,
    mut node_timer: Option<ResMut<NodeTimer>>,
    mut pause: PauseControl,
    mut targets: MutationTargets,
) {
    let Some(ref mutations) = config.definition.frame_mutations else {
        return;
    };

    for mutation in mutations {
        if mutation.frame != frame.0 {
            continue;
        }
        match &mutation.mutation {
            MutationKind::SetBreakerState(scenario_state) => {
                let target = map_scenario_breaker_state(*scenario_state);
                for mut state in &mut breakers {
                    *state = target;
                }
            }
            MutationKind::SetTimerRemaining(remaining) => {
                if let Some(ref mut timer) = node_timer {
                    timer.remaining = *remaining;
                }
            }
            MutationKind::SpawnExtraEntities(count) => {
                for _ in 0..*count {
                    targets.commands.spawn(Transform::default());
                }
            }
            MutationKind::MoveBolt(x, y) => {
                for mut position in &mut bolts {
                    position.0.x = *x;
                    position.0.y = *y;
                }
            }
            MutationKind::TogglePause => {
                if let Some(ref state) = pause.state
                    && let Some(ref mut next) = pause.next
                {
                    match ***state {
                        PlayingState::Active => next.set(PlayingState::Paused),
                        PlayingState::Paused => next.set(PlayingState::Active),
                    }
                }
            }
            MutationKind::SetRunStat(counter, value) => {
                if let Some(ref mut stats) = targets.run_stats {
                    apply_set_run_stat(stats, *counter, *value);
                }
            }
            MutationKind::DecrementRunStat(counter) => {
                if let Some(ref mut stats) = targets.run_stats {
                    apply_decrement_run_stat(stats, *counter);
                }
            }
            MutationKind::InjectOverStackedChip {
                chip_name,
                stacks,
                max_stacks,
            } => {
                if let Some(ref mut inventory) = targets.chip_inventory {
                    inventory.force_insert_entry(chip_name, *stacks, *max_stacks, None);
                }
            }
            MutationKind::InjectDuplicateOffers { chip_name } => {
                apply_inject_duplicate_offers(
                    chip_name,
                    &mut targets.chip_offers,
                    &mut targets.commands,
                );
            }
            MutationKind::InjectMaxedChipOffer { chip_name } => {
                apply_inject_maxed_chip_offer(
                    chip_name,
                    &mut targets.chip_inventory,
                    &mut targets.chip_offers,
                    &mut targets.commands,
                );
            }
            MutationKind::SpawnExtraSecondWindWalls(count) => {
                for _ in 0..*count {
                    targets.commands.spawn(SecondWindWall);
                }
            }
            MutationKind::InjectZeroChargeShield => {
                // Spawn a free-floating entity with a zero-charge shield so the
                // invariant fires without requiring a live breaker entity.
                targets.commands.spawn(ShieldActive { charges: 0 });
            }
            MutationKind::SpawnExtraPulseRings(count) => {
                for _ in 0..*count {
                    targets.commands.spawn(PulseRing);
                }
            }
            MutationKind::InjectWrongEffectiveSpeed { wrong_value } => {
                apply_inject_wrong_effective_speed(*wrong_value, &mut targets.commands);
            }
            MutationKind::SpawnExtraChainArcs(count) => {
                apply_spawn_extra_chain_arcs(*count, &mut targets.commands);
            }
        }
    }
}

fn apply_spawn_extra_chain_arcs(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn((
            ChainLightningChain {
                source: Vec2::ZERO,
                remaining_jumps: 0,
                damage: 0.0,
                hit_set: HashSet::new(),
                state: ChainState::Idle,
                range: 0.0,
                arc_speed: 0.0,
            },
            CleanupOnNodeExit,
        ));
        commands.spawn((ChainLightningArc, CleanupOnNodeExit));
    }
}

/// Sets the named [`RunStats`] counter to `value`.
pub const fn apply_set_run_stat(stats: &mut RunStats, counter: RunStatCounter, value: u32) {
    match counter {
        RunStatCounter::NodesCleared => stats.nodes_cleared = value,
        RunStatCounter::CellsDestroyed => stats.cells_destroyed = value,
        RunStatCounter::BumpsPerformed => stats.bumps_performed = value,
        RunStatCounter::PerfectBumps => stats.perfect_bumps = value,
        RunStatCounter::BoltsLost => stats.bolts_lost = value,
    }
}

/// Decrements the named [`RunStats`] counter by 1 (saturating at 0).
pub const fn apply_decrement_run_stat(stats: &mut RunStats, counter: RunStatCounter) {
    match counter {
        RunStatCounter::NodesCleared => {
            stats.nodes_cleared = stats.nodes_cleared.saturating_sub(1);
        }
        RunStatCounter::CellsDestroyed => {
            stats.cells_destroyed = stats.cells_destroyed.saturating_sub(1);
        }
        RunStatCounter::BumpsPerformed => {
            stats.bumps_performed = stats.bumps_performed.saturating_sub(1);
        }
        RunStatCounter::PerfectBumps => {
            stats.perfect_bumps = stats.perfect_bumps.saturating_sub(1);
        }
        RunStatCounter::BoltsLost => {
            stats.bolts_lost = stats.bolts_lost.saturating_sub(1);
        }
    }
}

/// Injects a [`ChipOffers`] resource containing two identical chips (triggers
/// [`InvariantKind::OfferingNoDuplicates`]).
pub fn apply_inject_duplicate_offers(
    chip_name: &str,
    chip_offers: &mut Option<ResMut<ChipOffers>>,
    commands: &mut Commands,
) {
    use breaker::{
        chips::definition::{ChipDefinition, Rarity},
        effect::{EffectKind, EffectNode, RootEffect, Target},
    };
    let def = ChipDefinition {
        name: chip_name.to_owned(),
        description: String::new(),
        rarity: Rarity::Common,
        max_stacks: 3,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: None,
        template_name: None,
    };
    let offers = ChipOffers(vec![
        ChipOffering::Normal(def.clone()),
        ChipOffering::Normal(def),
    ]);
    if let Some(existing) = chip_offers {
        **existing = offers;
    } else {
        commands.insert_resource(offers);
    }
}

/// Injects a [`ChipOffers`] resource containing a chip already maxed in
/// [`ChipInventory`] (triggers [`InvariantKind::MaxedChipNeverOffered`]).
pub fn apply_inject_maxed_chip_offer(
    chip_name: &str,
    chip_inventory: &mut Option<ResMut<ChipInventory>>,
    chip_offers: &mut Option<ResMut<ChipOffers>>,
    commands: &mut Commands,
) {
    use breaker::{
        chips::definition::{ChipDefinition, Rarity},
        effect::{EffectKind, EffectNode, RootEffect, Target},
    };
    let def = ChipDefinition {
        name: chip_name.to_owned(),
        description: String::new(),
        rarity: Rarity::Common,
        max_stacks: 1,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: None,
        template_name: None,
    };
    if let Some(inventory) = chip_inventory {
        inventory.force_insert_entry(chip_name, 1, 1, None);
    }
    let offers = ChipOffers(vec![ChipOffering::Normal(def)]);
    if let Some(existing) = chip_offers {
        **existing = offers;
    } else {
        commands.insert_resource(offers);
    }
}

/// Spawns an entity with `ActiveSpeedBoosts([1.5])` and `EffectiveSpeedMultiplier(wrong_value)`.
///
/// Since `1.5 != wrong_value`, `check_effective_speed_consistent` will detect the
/// divergence and fire a [`InvariantKind::EffectiveSpeedConsistent`] violation.
///
/// Used exclusively by the `effective_speed_consistent` self-test scenario.
pub fn apply_inject_wrong_effective_speed(wrong_value: f32, commands: &mut Commands) {
    commands.spawn((
        ActiveSpeedBoosts(vec![1.5]),
        EffectiveSpeedMultiplier(wrong_value),
    ));
}
