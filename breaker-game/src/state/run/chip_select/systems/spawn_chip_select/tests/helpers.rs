use bevy::prelude::*;

use super::super::system::*;
use crate::{
    chips::ChipDefinition,
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        greed::{GreedConfig, GreedStacks},
        resources::{ActiveProtocols, ProtocolOffer},
    },
    prelude::*,
    state::run::chip_select::{ChipOffering, ChipSelectConfig, resources::ChipOffers},
};

pub(super) fn make_offers(count: usize) -> ChipOffers {
    let all = vec![
        ChipOffering::Normal(ChipDefinition::test_simple("Piercing Shot")),
        ChipOffering::Normal(ChipDefinition::test_simple("Wide Breaker")),
        ChipOffering::Normal(ChipDefinition::test_simple("Surge")),
        ChipOffering::Normal(ChipDefinition::test_simple("Ricochet")),
        ChipOffering::Normal(ChipDefinition::test_simple("Quick Dash")),
    ];
    ChipOffers(all.into_iter().take(count).collect())
}

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
        },
        ProtocolKind::Conductor => ProtocolTuning::Conductor,
        ProtocolKind::Afterimage => ProtocolTuning::Afterimage {
            phantom_duration:      1.5,
            phantom_bolt_duration: 0.75,
        },
        ProtocolKind::Fission => ProtocolTuning::Fission {
            kills_per_split: 10,
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

pub(super) fn test_app_with_offers(offers: ChipOffers) -> App {
    let mut app = TestAppBuilder::new()
        .insert_resource(ChipSelectConfig::default())
        .insert_resource(offers)
        .with_resource::<ProtocolOffer>()
        .with_system(Update, spawn_chip_select)
        .build();
    app.insert_resource(ActiveProtocols::default());
    app
}

/// Like `test_app_with_offers` but also inserts a `ProtocolOffer` value.
pub(super) fn test_app_with_offers_and_protocol_offer(
    offers: ChipOffers,
    protocol_offer: ProtocolOffer,
) -> App {
    let mut app = test_app_with_offers(offers);
    app.insert_resource(protocol_offer);
    app
}

/// Build a test app with offers, a protocol offer, and an optional set of
/// (Greed-related) resources installed. Greed is "inactive" when
/// `active_protocols` is left at default; the helper inserts `GreedStacks`
/// and `GreedConfig` only when the caller passes them.
pub(super) fn test_app_greed(
    offers: ChipOffers,
    protocol_offer: ProtocolOffer,
    active_protocols: ActiveProtocols,
    greed_stacks: Option<GreedStacks>,
    greed_config: Option<GreedConfig>,
) -> App {
    let mut app = test_app_with_offers_and_protocol_offer(offers, protocol_offer);
    app.insert_resource(active_protocols);
    if let Some(stacks) = greed_stacks {
        app.insert_resource(stacks);
    }
    if let Some(cfg) = greed_config {
        app.insert_resource(cfg);
    }
    app
}
