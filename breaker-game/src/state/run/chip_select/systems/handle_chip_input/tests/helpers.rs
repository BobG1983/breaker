//! Shared test helpers for `handle_chip_input` tests.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use super::super::*;
use crate::{
    chips::ChipDefinition,
    effect_v3::{
        effects::PiercingConfig,
        types::{EffectType, Tree},
    },
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        messages::ProtocolSelected,
        resources::{ActiveProtocols, ProtocolOffer},
    },
    state::run::chip_select::{
        messages::ChipOfferSkipped,
        resources::{ChipOffering, SelectionRow},
    },
};

#[derive(Resource, Default)]
pub(super) struct ReceivedChips(pub(super) Vec<ChipSelected>);

pub(super) fn collect_chips(
    mut reader: MessageReader<ChipSelected>,
    mut received: ResMut<ReceivedChips>,
) {
    for msg in reader.read() {
        received.0.push(msg.clone());
    }
}

pub(super) fn make_offers(count: usize) -> ChipOffers {
    let all = vec![
        ChipOffering::Normal(ChipDefinition::test(
            "Piercing Shot",
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            3,
        )),
        ChipOffering::Normal(ChipDefinition::test_simple("Wide Breaker")),
        ChipOffering::Normal(ChipDefinition::test_simple("Surge")),
    ];
    ChipOffers(all.into_iter().take(count).collect())
}

pub(super) fn test_app() -> App {
    test_app_with_offers(make_offers(3))
}

pub(super) fn test_app_with_offers(offers: ChipOffers) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ButtonInput<KeyCode>>()
        .insert_resource(InputConfig::default())
        .insert_resource(ChipSelectSelection {
            row:        SelectionRow::Chip,
            chip_index: 0,
        })
        .insert_resource(offers)
        .with_resource::<ReceivedChips>()
        .with_resource::<ChipInventory>()
        .insert_resource(ChipSelectConfig::default())
        .with_resource::<ProtocolOffer>()
        .with_message::<ChipSelected>()
        .with_message::<ProtocolSelected>()
        .with_message::<ChangeState<ChipSelectState>>()
        .with_system(Update, (handle_chip_input, collect_chips).chain())
        .build();
    app.insert_resource(ActiveProtocols::default());
    // `ChipInputActions` now has a `MessageWriter<ChipOfferSkipped>` — every
    // test-app builder must register the message so Bevy can validate the
    // SystemParam. Greed-specific tests still overwrite this with their own
    // builder (no double-registration concern; `add_message` is idempotent).
    app.add_message::<ChipOfferSkipped>();
    app
}

/// Build a `ProtocolDefinition` for a given `kind` with a human-readable name.
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

/// Build a test app with three chip offers, an empty protocol offer by
/// default, and the `ProtocolSelected` message registered.
pub(super) fn test_app_with_offers_and_protocol_offer(
    offers: ChipOffers,
    protocol: ProtocolOffer,
) -> App {
    let mut app = test_app_with_offers(offers);
    app.insert_resource(protocol);
    app
}

/// Collect `ProtocolSelected` messages for assertion.
#[derive(Resource, Default)]
pub(super) struct ReceivedProtocols(pub(super) Vec<ProtocolSelected>);

pub(super) fn collect_protocols(
    mut reader: MessageReader<ProtocolSelected>,
    mut received: ResMut<ReceivedProtocols>,
) {
    for msg in reader.read() {
        received.0.push(msg.clone());
    }
}

/// Build a test app with an active `ProtocolOffer(Some(greed))` and the
/// `ReceivedProtocols` collector wired in.
pub(super) fn test_app_with_protocol_offer(
    offers: ChipOffers,
    protocol: ProtocolOffer,
    selection: ChipSelectSelection,
) -> App {
    let mut app = test_app_with_offers_and_protocol_offer(offers, protocol);
    app.insert_resource(selection);
    app.init_resource::<ReceivedProtocols>();
    app.add_systems(Update, collect_protocols.after(handle_chip_input));
    app
}

/// Collect `ChipOfferSkipped` messages for assertion. The message is a
/// unit-payload struct — the count is the assertion target.
#[derive(Resource, Default)]
pub(super) struct ReceivedSkips(pub(super) usize);

pub(super) fn collect_skips(
    mut reader: MessageReader<ChipOfferSkipped>,
    mut received: ResMut<ReceivedSkips>,
) {
    for _ in reader.read() {
        received.0 += 1;
    }
}

/// Builder helper that wraps the existing `test_app_with_protocol_offer`
/// and additionally installs `ActiveProtocols`, the `ChipOfferSkipped`
/// message + `ReceivedSkips` collector, and the `collect_skips` system
/// after `handle_chip_input`.
pub(super) fn test_app_greed_input(
    offers: ChipOffers,
    protocol_offer: ProtocolOffer,
    selection: ChipSelectSelection,
    active_protocols: ActiveProtocols,
) -> App {
    let mut app = test_app_with_protocol_offer(offers, protocol_offer, selection);
    app.insert_resource(active_protocols);
    app.add_message::<ChipOfferSkipped>();
    app.init_resource::<ReceivedSkips>();
    app.add_systems(Update, collect_skips.after(handle_chip_input));
    app
}
