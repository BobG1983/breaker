//! Individual mutation handler implementations extracted from the dispatch chain.

use std::collections::HashSet;

use bevy::prelude::*;
use breaker::{
    bolt::components::{Bolt, PrimaryBolt},
    breaker::components::PrimaryBreaker,
    chips::inventory::ChipInventory,
    effect_v3::effects::{
        chain_lightning::{ChainLightningArc, ChainLightningChain, ChainState},
        gravity_well::GravityWellSource,
    },
    mutators::{
        hazards::{
            self,
            definition::HazardKind,
            resources::{ActiveHazards, HazardRegistry},
        },
        protocols::burnout::BurnoutHeat,
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
use rantzsoft_spatial2d::components::{BaseSpeed, Position2D};
use rantzsoft_stateflow::CleanupOnExit;

use crate::{
    invariants::{ScenarioTagBolt, ScenarioTagBreaker},
    types::RunStatCounter,
};

pub(super) fn apply_inject_hazard_stack(
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
    if !hazards::activate_from_registry(registry, kind, commands) {
        warn!("InjectHazardStack: no definition for {kind_name}");
        return;
    }
    for _ in 0..stacks {
        active.add_stack(kind);
    }
}

/// Sets [`NodeTimer::remaining`] to the requested value when the timer
/// resource exists, no-op otherwise.
pub(super) fn apply_set_timer_remaining(
    remaining: f32,
    node_timer: &mut Option<ResMut<NodeTimer>>,
) {
    if let Some(timer) = node_timer {
        timer.remaining = remaining;
    }
}

/// Optional-aware wrapper around [`apply_set_run_stat`].
pub(super) fn apply_set_run_stat_optional(
    counter: RunStatCounter,
    value: u32,
    run_stats: &mut Option<ResMut<RunStats>>,
) {
    if let Some(stats) = run_stats {
        apply_set_run_stat(stats, counter, value);
    }
}

/// Optional-aware wrapper around [`apply_decrement_run_stat`].
pub(super) fn apply_decrement_run_stat_optional(
    counter: RunStatCounter,
    run_stats: &mut Option<ResMut<RunStats>>,
) {
    if let Some(stats) = run_stats {
        apply_decrement_run_stat(stats, counter);
    }
}

/// Spawns `count` entities with only a default [`Transform`].
pub(super) fn apply_spawn_extra_entities(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn(Transform::default());
    }
}

/// Moves every tagged bolt to world-space `(x, y)`, preserving z.
pub(super) fn apply_move_bolt(
    x: f32,
    y: f32,
    bolts: &mut Query<&mut Position2D, With<ScenarioTagBolt>>,
) {
    for mut position in bolts {
        position.0.x = x;
        position.0.y = y;
    }
}

/// Parses `kind_name` as a [`HazardKind`] and injects a 0-stack entry via
/// `force_insert_entry`. No-op if [`ActiveHazards`] is absent or the name is
/// unrecognised.
pub(super) fn apply_inject_zero_stack_hazard(
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

pub(super) fn apply_spawn_extra_chain_arcs(count: usize, commands: &mut Commands) {
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
                tick:            0,
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

pub(super) fn apply_spawn_extra_primary_breakers(count: usize, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn(PrimaryBreaker);
    }
}

/// Spawns `count` extra entities each carrying `(Bolt, PrimaryBolt,
/// Position2D(Vec2::ZERO), BaseSpeed(400.0))`.
pub(super) fn apply_spawn_extra_primary_bolts(count: u32, commands: &mut Commands) {
    for _ in 0..count {
        commands.spawn((Bolt, PrimaryBolt, Position2D(Vec2::ZERO), BaseSpeed(400.0)));
    }
}

/// Sets `BurnoutHeat.heat` and `BurnoutHeat.still_timer` on every tagged
/// breaker that already carries the component.
pub(super) fn apply_set_burnout_heat(
    heat: f32,
    still_timer: f32,
    burnout_heats: &mut Query<&mut BurnoutHeat, With<ScenarioTagBreaker>>,
) {
    for mut bh in burnout_heats {
        bh.heat = heat;
        bh.still_timer = still_timer;
    }
}

pub(super) type BirthingBoltLayersQuery<'w, 's> =
    Query<'w, 's, &'static mut CollisionLayers, (With<ScenarioTagBolt>, With<Birthing>)>;

pub(super) fn apply_inject_non_zero_birthing_layers(bolts: &mut BirthingBoltLayersQuery) {
    for mut layers in bolts {
        layers.membership = 0xFF;
        layers.mask = 0xFF;
    }
}
