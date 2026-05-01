//! Shared test helpers for `dispatch_protocol_selection` tests.

use bevy::{ecs::message::Messages, prelude::*};

use super::super::dispatch_protocol_selection;
use crate::{
    effect_v3::{
        effects::{DamageBoostConfig, PiercingConfig},
        types::{EffectType, RootNode, StampTarget, Tree},
    },
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        messages::ProtocolSelected,
        resources::{ActiveProtocols, ProtocolRegistry},
    },
    prelude::*,
};

/// Local `def_for` helper — mirrors `protocol/resources.rs::tests::def_for`.
pub(super) fn def_for(kind: ProtocolKind, name: &str) -> ProtocolDefinition {
    let tuning = match kind {
        ProtocolKind::Deadline => ProtocolTuning::Deadline { effects: vec![] },
        ProtocolKind::Ricochet => ProtocolTuning::Ricochet { effects: vec![] },
        ProtocolKind::Anchor => ProtocolTuning::Anchor { effects: vec![] },
        ProtocolKind::Kickstart => ProtocolTuning::Kickstart { effects: vec![] },
        ProtocolKind::DebtCollector => ProtocolTuning::DebtCollector {
            stack_per_bump: 0.1,
        },
        ProtocolKind::IronCurtain => ProtocolTuning::IronCurtain {
            damage_fraction: 0.25,
            falloff_start:   0.5,
        },
        ProtocolKind::EchoStrike => ProtocolTuning::EchoStrike {
            max_echoes:      3,
            newest_fraction: 0.5,
            middle_fraction: 0.25,
            oldest_fraction: 0.125,
        },
        ProtocolKind::Siphon => ProtocolTuning::Siphon {
            streak_window: 2.0,
            time_per_kill: 0.25,
        },
        ProtocolKind::Greed => ProtocolTuning::Greed {
            rarity_boost_per_skip: 0.05,
        },
        ProtocolKind::RecklessDash => ProtocolTuning::RecklessDash {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    true,
        },
        ProtocolKind::Burnout => ProtocolTuning::Burnout {
            fill_duration:               4.0,
            drain_duration:              2.0,
            still_threshold:             1.5,
            full_heat_damage_multiplier: 4.0,
            speed_boost_duration:        2.0,
            shockwave_base_range:        0.0,
            shockwave_range_per_level:   0.0,
            shockwave_stacks:            0,
            shockwave_speed:             0.0,
        },
        ProtocolKind::Conductor => ProtocolTuning::Conductor,
        ProtocolKind::Afterimage => ProtocolTuning::Afterimage {
            phantom_duration:      1.5,
            phantom_bolt_duration: 0.75,
        },
        ProtocolKind::Fission => ProtocolTuning::Fission {
            kills_per_split:      10,
            divergence_angle_rad: 0.0,
        },
        ProtocolKind::TierRegression => ProtocolTuning::TierRegression { tiers_back: 1 },
    };
    ProtocolDefinition {
        name: name.to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning,
    }
}

/// Build a `ProtocolDefinition` whose `Deadline` tuning carries the given
/// effect-tree roots.
pub(super) fn deadline_with_effects(effects: Vec<RootNode>) -> ProtocolDefinition {
    ProtocolDefinition {
        name:        "Deadline".to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      ProtocolTuning::Deadline { effects },
    }
}

/// Build a `Kickstart` definition with the given effect roots.
pub(super) fn kickstart_with_effects(effects: Vec<RootNode>) -> ProtocolDefinition {
    ProtocolDefinition {
        name:        "Kickstart".to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      ProtocolTuning::Kickstart { effects },
    }
}

/// Build an `Anchor` definition with the given effect roots.
pub(super) fn anchor_with_effects(effects: Vec<RootNode>) -> ProtocolDefinition {
    ProtocolDefinition {
        name:        "Anchor".to_string(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      ProtocolTuning::Anchor { effects },
    }
}

/// A simple effect-tree root that stamps a Piercing charge onto the Breaker.
pub(super) fn stub_piercing_root() -> RootNode {
    RootNode::Stamp(
        StampTarget::Breaker,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
    )
}

/// A second effect-tree root for multi-entry tests.
pub(super) fn stub_damage_boost_root() -> RootNode {
    use ordered_float::OrderedFloat;
    RootNode::Stamp(
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
    )
}

/// Build a registry seeded with every `ProtocolKind::ALL` variant.
pub(super) fn fully_seeded_registry() -> ProtocolRegistry {
    let mut registry = ProtocolRegistry::default();
    for kind in ProtocolKind::ALL {
        registry.insert(def_for(*kind, &format!("{kind:?}")));
    }
    registry
}

/// Build a test app at `ChipSelectState::Selecting` with the dispatch
/// system wired in `Update` behind the canonical `run_if` gate. Caller
/// inserts the registry.
///
/// The `.run_if(in_state(ChipSelectState::Selecting))` gate mirrors what
/// `ProtocolPlugin::build` is specified to install. Registering the system
/// with the same gate here lets Behavior 6 exercise the state gate without
/// needing the full plugin.
pub(super) fn test_app_selecting(registry: ProtocolRegistry) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .insert_resource(registry)
        .with_resource::<ActiveProtocols>()
        .with_message::<ProtocolSelected>()
        .with_system(
            Update,
            dispatch_protocol_selection.run_if(in_state(ChipSelectState::Selecting)),
        )
        .build()
}

/// Build a test app at `NodeState::Playing` (NOT `ChipSelectState::Selecting`)
/// for gate-check tests. Same `run_if` gate applied — the state is the
/// independent variable.
pub(super) fn test_app_node_playing(registry: ProtocolRegistry) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .insert_resource(registry)
        .with_resource::<ActiveProtocols>()
        .with_message::<ProtocolSelected>()
        .with_system(
            Update,
            dispatch_protocol_selection.run_if(in_state(ChipSelectState::Selecting)),
        )
        .build()
}

/// Write a `ProtocolSelected` message.
pub(super) fn send_selection(app: &mut App, kind: ProtocolKind) {
    app.world_mut()
        .resource_mut::<Messages<ProtocolSelected>>()
        .write(ProtocolSelected { kind });
}
