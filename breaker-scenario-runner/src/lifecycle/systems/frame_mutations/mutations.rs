//! Per-frame debug mutations: breaker state, timer, entity spawning, pause, run stats, chip injection.

use std::collections::HashSet;

use bevy::{ecs::system::SystemParam, prelude::*};
use breaker::{
    bolt::components::{Bolt, PrimaryBolt},
    breaker::components::{DashState, PrimaryBreaker},
    chips::inventory::ChipInventory,
    effect_v3::effects::{
        chain_lightning::{ChainLightningArc, ChainLightningChain, ChainState},
        gravity_well::GravityWellSource,
        pulse::PulseRing,
        second_wind::SecondWindWall,
        shield::ShieldWall,
    },
    hazard::{
        self,
        definition::HazardKind,
        resources::{ActiveHazards, HazardRegistry},
    },
    protocol::{
        self,
        definition::ProtocolKind,
        resources::{ActiveProtocols, ProtocolRegistry},
    },
    shared::birthing::Birthing,
    state::{
        run::{
            RunStats,
            chip_select::{ChipOffering, ChipOffers},
            node::resources::NodeTimer,
        },
        types::NodeState,
    },
};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    invariants::{ScenarioFrame, ScenarioTagBolt, ScenarioTagBreaker},
    lifecycle::systems::{entity_tagging::map_scenario_dash_state, types::ScenarioConfig},
    types::{MutationKind, RunStatCounter},
};

/// Grouped system parameters for pause toggle control.
///
/// Extracted to keep [`apply_debug_frame_mutations`] under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct PauseControl<'w> {
    /// Virtual time resource for pausing/unpausing.
    time_virtual: ResMut<'w, Time<Virtual>>,
}

/// Grouped system parameters for mutation targets that need additional game state.
///
/// Extracted to keep [`apply_debug_frame_mutations`] under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct MutationTargets<'w, 's> {
    /// [`RunStats`] resource -- absent before a run starts.
    run_stats:            Option<ResMut<'w, RunStats>>,
    /// [`ChipInventory`] resource -- absent before a run starts.
    chip_inventory:       Option<ResMut<'w, ChipInventory>>,
    /// [`ChipOffers`] resource -- present only during chip select.
    chip_offers:          Option<ResMut<'w, ChipOffers>>,
    /// [`ActiveHazards`] resource -- always present; used by
    /// [`MutationKind::InjectZeroStackHazard`] and [`MutationKind::InjectHazardStack`].
    active_hazards:       Option<ResMut<'w, ActiveHazards>>,
    /// [`HazardRegistry`] resource -- used by
    /// [`MutationKind::InjectHazardStack`] to look up tuning.
    hazard_registry:      Option<Res<'w, HazardRegistry>>,
    /// [`ActiveProtocols`] resource -- used by
    /// [`MutationKind::InjectProtocol`] to record activation.
    active_protocols:     Option<ResMut<'w, ActiveProtocols>>,
    /// [`ProtocolRegistry`] resource -- used by
    /// [`MutationKind::InjectProtocol`] to look up tuning + effect trees.
    protocol_registry:    Option<Res<'w, ProtocolRegistry>>,
    /// Tagged breaker entities -- used by
    /// [`MutationKind::InjectProtocol`] as the stamp target for
    /// effect-tree protocols.
    breaker_entities:     Query<'w, 's, Entity, With<ScenarioTagBreaker>>,
    /// [`Commands`] for inserting resources when the optional resource is absent.
    commands:             Commands<'w, 's>,
    /// Bolt entities with `Aabb2D` -- for [`MutationKind::InjectMismatchedBoltAabb`].
    bolt_aabbs:           Query<'w, 's, &'static mut Aabb2D, With<ScenarioTagBolt>>,
    /// Birthing bolt entities -- for [`MutationKind::InjectNonZeroBirthingLayers`].
    birthing_bolt_layers:
        Query<'w, 's, &'static mut CollisionLayers, (With<ScenarioTagBolt>, With<Birthing>)>,
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
    mut breakers: Query<&mut DashState, With<ScenarioTagBreaker>>,
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
            MutationKind::SetDashState(scenario_state) => {
                let target = map_scenario_dash_state(*scenario_state);
                for mut state in &mut breakers {
                    *state = target;
                }
            }
            MutationKind::SetTimerRemaining(remaining) => {
                apply_set_timer_remaining(*remaining, &mut node_timer);
            }
            MutationKind::SpawnExtraEntities(count) => {
                apply_spawn_extra_entities(*count, &mut targets.commands);
            }
            MutationKind::MoveBolt(x, y) => {
                apply_move_bolt(*x, *y, &mut bolts);
            }
            MutationKind::TogglePause => {
                apply_toggle_pause(&mut pause);
            }
            MutationKind::SetRunStat(counter, value) => {
                apply_set_run_stat_optional(*counter, *value, &mut targets.run_stats);
            }
            MutationKind::DecrementRunStat(counter) => {
                apply_decrement_run_stat_optional(*counter, &mut targets.run_stats);
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
            MutationKind::SpawnExtraShieldWalls(count) => {
                for _ in 0..*count {
                    targets.commands.spawn(ShieldWall);
                }
            }
            MutationKind::SpawnExtraPulseRings(count) => {
                for _ in 0..*count {
                    targets.commands.spawn(PulseRing);
                }
            }
            MutationKind::SpawnExtraChainArcs(count) => {
                apply_spawn_extra_chain_arcs(*count, &mut targets.commands);
            }
            MutationKind::InjectMismatchedBoltAabb => {
                apply_inject_mismatched_bolt_aabb(&mut targets.bolt_aabbs);
            }
            MutationKind::SpawnExtraGravityWells(count) => {
                apply_spawn_extra_gravity_wells(*count, &mut targets.commands);
            }
            MutationKind::SpawnExtraPrimaryBreakers(count) => {
                apply_spawn_extra_primary_breakers(*count, &mut targets.commands);
            }
            MutationKind::SpawnExtraPrimaryBolts { count } => {
                apply_spawn_extra_primary_bolts(*count, &mut targets.commands);
            }
            MutationKind::InjectNonZeroBirthingLayers => {
                apply_inject_non_zero_birthing_layers(&mut targets.birthing_bolt_layers);
            }
            MutationKind::InjectZeroStackHazard { kind_name } => {
                apply_inject_zero_stack_hazard(kind_name, &mut targets.active_hazards);
            }
            MutationKind::InjectHazardStack { kind_name, stacks } => {
                apply_hazard_injection(kind_name, *stacks, &mut targets);
            }
            MutationKind::InjectProtocol { kind_name } => {
                apply_protocol_injection(kind_name, &mut targets);
            }
        }
    }
}

fn apply_hazard_injection(kind_name: &str, stacks: u32, targets: &mut MutationTargets) {
    apply_inject_hazard_stack(
        kind_name,
        stacks,
        &mut targets.active_hazards,
        targets.hazard_registry.as_deref(),
        &mut targets.commands,
    );
}

fn apply_protocol_injection(kind_name: &str, targets: &mut MutationTargets) {
    let Some(kind) = parse_protocol_kind(kind_name) else {
        warn!("InjectProtocol: unknown kind {kind_name:?}");
        return;
    };
    let Some(active) = targets.active_protocols.as_deref_mut() else {
        return;
    };
    let Some(registry) = targets.protocol_registry.as_deref() else {
        warn!("InjectProtocol: ProtocolRegistry absent — cannot activate {kind_name}");
        return;
    };
    let breakers: Vec<Entity> = targets.breaker_entities.iter().collect();
    if !protocol::activate_from_registry(registry, kind, &breakers, &mut targets.commands, active) {
        warn!("InjectProtocol: no definition for {kind_name}");
    }
}

fn parse_protocol_kind(name: &str) -> Option<ProtocolKind> {
    ProtocolKind::ALL
        .iter()
        .copied()
        .find(|k| format!("{k:?}") == name)
}

/// Installs `stacks` of the named hazard by calling the normal
/// [`hazards::activate`] path once and then bumping the stack count to the
/// requested total. No-op if [`ActiveHazards`] or [`HazardRegistry`] is
/// absent or the name is unrecognised.
fn apply_inject_hazard_stack(
    kind_name: &str,
    stacks: u32,
    active_hazards: &mut Option<ResMut<ActiveHazards>>,
    hazard_registry: Option<&HazardRegistry>,
    commands: &mut Commands,
) {
    if stacks == 0 {
        return;
    }
    let Some(kind) = parse_hazard_kind(kind_name) else {
        warn!("InjectHazardStack: unknown kind {kind_name:?}");
        return;
    };
    let Some(active) = active_hazards else {
        return;
    };
    let Some(registry) = hazard_registry else {
        warn!("InjectHazardStack: HazardRegistry absent — cannot activate {kind_name}");
        return;
    };
    if !hazard::activate_from_registry(registry, kind, commands) {
        warn!("InjectHazardStack: no definition for {kind_name}");
        return;
    }
    for _ in 0..stacks {
        active.add_stack(kind);
    }
}

/// Sets [`NodeTimer::remaining`] to the requested value when the timer
/// resource exists, no-op otherwise.
fn apply_set_timer_remaining(remaining: f32, node_timer: &mut Option<ResMut<NodeTimer>>) {
    if let Some(timer) = node_timer {
        timer.remaining = remaining;
    }
}

/// Optional-aware wrapper around [`apply_set_run_stat`] for the
/// dispatch-loop site, keeping that function under the line cap.
fn apply_set_run_stat_optional(
    counter: RunStatCounter,
    value: u32,
    run_stats: &mut Option<ResMut<RunStats>>,
) {
    if let Some(stats) = run_stats {
        apply_set_run_stat(stats, counter, value);
    }
}

/// Optional-aware wrapper around [`apply_decrement_run_stat`].
fn apply_decrement_run_stat_optional(
    counter: RunStatCounter,
    run_stats: &mut Option<ResMut<RunStats>>,
) {
    if let Some(stats) = run_stats {
        apply_decrement_run_stat(stats, counter);
    }
}

/// Spawns `count` entities with only a default [`Transform`].
fn apply_spawn_extra_entities(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn(Transform::default());
    }
}

/// Moves every tagged bolt to world-space `(x, y)`, preserving z.
fn apply_move_bolt(x: f32, y: f32, bolts: &mut Query<&mut Position2D, With<ScenarioTagBolt>>) {
    for mut position in bolts {
        position.0.x = x;
        position.0.y = y;
    }
}

/// Parses `kind_name` as a [`HazardKind`] and injects a 0-stack entry via
/// `force_insert_entry`. No-op if [`ActiveHazards`] is absent or the name is
/// unrecognised.
fn apply_inject_zero_stack_hazard(
    kind_name: &str,
    active_hazards: &mut Option<ResMut<ActiveHazards>>,
) {
    let Some(kind) = parse_hazard_kind(kind_name) else {
        warn!("InjectZeroStackHazard: unknown kind {kind_name:?}");
        return;
    };
    if let Some(active) = active_hazards {
        active.force_insert_entry(kind, 0);
    }
}

fn parse_hazard_kind(name: &str) -> Option<HazardKind> {
    HazardKind::ALL
        .iter()
        .copied()
        .find(|k| format!("{k:?}") == name)
}

/// Toggles pause via `Time<Virtual>` -- pauses or unpauses the game clock.
///
/// No-op if the time resource is not available (should always be present).
fn apply_toggle_pause(pause: &mut PauseControl) {
    if pause.time_virtual.is_paused() {
        pause.time_virtual.unpause();
    } else {
        pause.time_virtual.pause();
    }
}

fn apply_spawn_extra_chain_arcs(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn((
            ChainLightningChain {
                source_pos:      Vec2::ZERO,
                remaining_jumps: 0,
                damage:          0.0,
                hit_set:         HashSet::new(),
                state:           ChainState::Idle,
                range:           0.0,
                arc_speed:       0.0,
            },
            CleanupOnExit::<NodeState>::default(),
        ));
        commands.spawn((ChainLightningArc, CleanupOnExit::<NodeState>::default()));
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
        effect_v3::{
            effects::PiercingConfig,
            types::{EffectType, RootNode, StampTarget, Tree},
        },
    };
    let def = ChipDefinition {
        name:          chip_name.to_owned(),
        description:   String::new(),
        rarity:        Rarity::Common,
        max_stacks:    3,
        effects:       vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        )],
        ingredients:   None,
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
        effect_v3::{
            effects::PiercingConfig,
            types::{EffectType, RootNode, StampTarget, Tree},
        },
    };
    let def = ChipDefinition {
        name:          chip_name.to_owned(),
        description:   String::new(),
        rarity:        Rarity::Common,
        max_stacks:    1,
        effects:       vec![RootNode::Stamp(
            StampTarget::Bolt,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        )],
        ingredients:   None,
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

/// Sets the first tagged bolt's `Aabb2D.half_extents` to `Vec2::splat(999.0)`.
///
/// No-op if no bolts exist. Used exclusively by the `aabb_matches_entity_dimensions`
/// self-test scenario.
pub fn apply_inject_mismatched_bolt_aabb(bolts: &mut Query<&mut Aabb2D, With<ScenarioTagBolt>>) {
    if let Some(mut aabb) = bolts.iter_mut().next() {
        aabb.half_extents = Vec2::splat(999.0);
    }
}

/// Spawns `count` extra [`GravityWellSource`] entities.
///
/// Used exclusively by the `gravity_well_count_reasonable` self-test scenario.
pub fn apply_spawn_extra_gravity_wells(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn((GravityWellSource, CleanupOnExit::<NodeState>::default()));
    }
}

fn apply_spawn_extra_primary_breakers(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn(PrimaryBreaker);
    }
}

/// Spawns `count` extra entities each carrying `(Bolt, PrimaryBolt,
/// Position2D(Vec2::ZERO), BaseSpeed(400.0))`.
///
/// Each spawned entity satisfies the `With<Bolt>` query used by
/// `check_exactly_one_primary_bolt`, so a count of 2+ pushes the
/// `PrimaryBolt` total above 1 and triggers an
/// [`InvariantKind::ExactlyOnePrimaryBolt`] violation.
fn apply_spawn_extra_primary_bolts(count: u32, commands: &mut Commands) {
    use rantzsoft_spatial2d::components::{BaseSpeed, Position2D};
    for _ in 0..count {
        commands.spawn((Bolt, PrimaryBolt, Position2D(Vec2::ZERO), BaseSpeed(400.0)));
    }
}

type BirthingBoltLayersQuery<'w, 's> =
    Query<'w, 's, &'static mut CollisionLayers, (With<ScenarioTagBolt>, With<Birthing>)>;

fn apply_inject_non_zero_birthing_layers(bolts: &mut BirthingBoltLayersQuery) {
    for mut layers in bolts {
        layers.membership = 0xFF;
        layers.mask = 0xFF;
    }
}
