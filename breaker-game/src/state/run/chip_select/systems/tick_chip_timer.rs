//! System to tick the chip selection countdown timer.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use crate::{
    chips::inventory::ChipInventory,
    prelude::*,
    state::run::chip_select::{
        ChipSelectConfig,
        resources::{ChipOffering, ChipOffers, ChipSelectTimer},
    },
};

/// Ticks the chip selection timer and auto-advances on expiry.
///
/// Timer expiry transitions to [`ChipSelectState::AnimateOut`] (skip, no chip).
pub(crate) fn tick_chip_timer(
    time: Res<Time>,
    mut timer: ResMut<ChipSelectTimer>,
    mut state_writer: MessageWriter<ChangeState<ChipSelectState>>,
    offers: Option<Res<ChipOffers>>,
    inventory: Option<ResMut<ChipInventory>>,
    config: Option<Res<ChipSelectConfig>>,
) {
    timer.remaining -= time.delta_secs();

    if timer.remaining <= 0.0 {
        timer.remaining = 0.0;

        // On timeout, all offered chips were seen but none selected — decay normal only
        if let (Some(offers), Some(mut inventory), Some(config)) = (offers, inventory, config) {
            for offer in &offers.0 {
                if let ChipOffering::Normal(_) = offer {
                    inventory.record_offered(offer.name(), config.seen_decay_factor);
                }
            }
        }

        state_writer.write(ChangeState::new());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{ecs::message::Messages, time::TimeUpdateStrategy};
    use rantzsoft_stateflow::ChangeState;

    use super::*;

    /// Frame delta the chip/hazard timer tests assume when they call
    /// `app.update()`. `TestAppBuilder::new()` pins `TimeUpdateStrategy` to
    /// `ManualDuration(Duration::ZERO)` for parallel-test determinism (so
    /// wall-clock time can't leak into `Time<Fixed>::overstep`). Tests that
    /// read `Time::delta` in `Update` must override the strategy with a
    /// concrete positive delta — 16ms (~one 60Hz frame) is enough for a
    /// meaningful "did the timer decrement?" assertion.
    const TEST_FRAME_DELTA: Duration = Duration::from_millis(16);

    fn test_app(remaining: f32) -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .with_message::<ChangeState<ChipSelectState>>()
            .insert_resource(ChipSelectTimer { remaining })
            .insert_resource(TimeUpdateStrategy::ManualDuration(TEST_FRAME_DELTA))
            .with_system(Update, tick_chip_timer)
            .build()
    }

    #[test]
    fn timer_decrements_after_update() {
        let mut app = test_app(10.0);

        // First update initializes time; second gets a real delta
        app.update();
        app.update();

        let timer = app.world().resource::<ChipSelectTimer>();
        assert!(
            timer.remaining < 10.0,
            "expected timer to decrease, got: {}",
            timer.remaining
        );
    }

    #[test]
    fn timer_expiry_transitions_to_transition_in() {
        // Start with 0 remaining — should expire immediately
        let mut app = test_app(0.0);
        app.update();

        let msgs = app
            .world()
            .resource::<Messages<ChangeState<ChipSelectState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<ChipSelectState> message"
        );
    }

    #[test]
    fn timer_clamps_to_zero_on_expiry() {
        let mut app = test_app(0.0);
        app.update();

        let timer = app.world().resource::<ChipSelectTimer>();
        assert!(
            timer.remaining.abs() < f32::EPSILON,
            "expected 0.0, got: {}",
            timer.remaining
        );
    }

    #[test]
    fn no_transition_when_time_remains() {
        let mut app = test_app(100.0);
        app.update();

        let msgs = app
            .world()
            .resource::<Messages<ChangeState<ChipSelectState>>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            0,
            "expected no ChangeState message when time remains"
        );
    }

    // --- Decay-on-expiry tests ---

    use crate::{
        chips::{ChipDefinition, definition::EvolutionIngredient, inventory::ChipInventory},
        effect_v3::{
            effects::PiercingConfig,
            types::{EffectType, Tree},
        },
        state::run::chip_select::{
            ChipSelectConfig,
            resources::{ChipOffering, ChipOffers},
        },
    };

    fn make_offers_3() -> ChipOffers {
        ChipOffers(vec![
            ChipOffering::Normal(ChipDefinition::test(
                "A",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )),
            ChipOffering::Normal(ChipDefinition::test(
                "B",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )),
            ChipOffering::Normal(ChipDefinition::test(
                "C",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )),
        ])
    }

    fn test_app_with_offers(remaining: f32, offers: ChipOffers) -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .with_message::<ChangeState<ChipSelectState>>()
            .insert_resource(ChipSelectTimer { remaining })
            .insert_resource(offers)
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .insert_resource(TimeUpdateStrategy::ManualDuration(TEST_FRAME_DELTA))
            .with_system(Update, tick_chip_timer)
            .build()
    }

    #[test]
    fn timer_expiry_applies_decay_to_all_offered_chips() {
        // Timer at 0.0 — expires immediately on first update
        let mut app = test_app_with_offers(0.0, make_offers_3());
        app.update();

        let inventory = app.world().resource::<ChipInventory>();
        let config = app.world().resource::<ChipSelectConfig>();
        let expected_decay = config.seen_decay_factor; // 0.8

        // On timeout (no chip selected), ALL offered chips should be decayed
        for name in &["A", "B", "C"] {
            let decay = inventory.weight_decay(name);
            assert!(
                (decay - expected_decay).abs() < f32::EPSILON,
                "expected chip '{name}' to have decay {expected_decay} after timer expiry, got {decay}"
            );
        }
    }

    #[test]
    fn timer_no_decay_when_time_remains() {
        // Timer at 100.0 — plenty of time remaining, should NOT expire
        let mut app = test_app_with_offers(100.0, make_offers_3());
        app.update();

        let inventory = app.world().resource::<ChipInventory>();

        // Timer has not expired — no decay should be applied
        for name in &["A", "B", "C"] {
            let decay = inventory.weight_decay(name);
            assert!(
                (decay - 1.0).abs() < f32::EPSILON,
                "expected chip '{name}' to have no decay (1.0) when time remains, got {decay}"
            );
        }
    }

    // --- Evolution decay-skip tests ---

    #[test]
    fn timer_expiry_applies_decay_only_to_normal_offerings_not_evolution() {
        // Offers: Normal("A"), Evolution(result: "B+"), Normal("C")
        // On timer expiry, decay should be applied to "A" (0.8) and "C" (0.8)
        // but NOT to "B+" (should remain 1.0)
        let offers = ChipOffers(vec![
            ChipOffering::Normal(ChipDefinition::test(
                "A",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )),
            ChipOffering::Evolution {
                ingredients: vec![EvolutionIngredient {
                    chip_name:       "X".to_owned(),
                    stacks_required: 2,
                }],
                result:      ChipDefinition::test(
                    "B+",
                    Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 5 })),
                    1,
                ),
            },
            ChipOffering::Normal(ChipDefinition::test(
                "C",
                Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
                3,
            )),
        ]);

        let mut app = test_app_with_offers(0.0, offers);
        app.update();

        let inventory = app.world().resource::<ChipInventory>();

        let decay_a = inventory.weight_decay("A");
        assert!(
            (decay_a - 0.8).abs() < f32::EPSILON,
            "Normal offering 'A' should have decay 0.8 after timer expiry, got {decay_a}"
        );

        let decay_c = inventory.weight_decay("C");
        assert!(
            (decay_c - 0.8).abs() < f32::EPSILON,
            "Normal offering 'C' should have decay 0.8 after timer expiry, got {decay_c}"
        );

        let decay_b_plus = inventory.weight_decay("B+");
        assert!(
            (decay_b_plus - 1.0).abs() < f32::EPSILON,
            "Evolution offering 'B+' should NOT have decay applied (expected 1.0), got {decay_b_plus}"
        );
    }

    // ─────────────────────────────────────────────────────────────────
    // Regression guard: timer expiry with a `ProtocolOffer::Some(_)` MUST
    // NOT emit `ProtocolSelected`. Timer-driven chip-select closure never
    // auto-picks a protocol.
    // ─────────────────────────────────────────────────────────────────

    use crate::mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        messages::ProtocolSelected,
        resources::{ActiveProtocols, ProtocolOffer},
    };

    fn def_for(kind: ProtocolKind, name: &str) -> ProtocolDefinition {
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

    fn test_app_for_protocol_regression(protocol_offer: ProtocolOffer) -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .with_message::<ChangeState<ChipSelectState>>()
            .with_message::<ProtocolSelected>()
            .insert_resource(ChipSelectTimer { remaining: 0.0 })
            .insert_resource(make_offers_3())
            .with_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .with_resource::<ActiveProtocols>()
            .insert_resource(protocol_offer)
            .insert_resource(TimeUpdateStrategy::ManualDuration(TEST_FRAME_DELTA))
            .with_system(Update, tick_chip_timer)
            .build()
    }

    #[test]
    fn timer_expiry_with_protocol_offer_some_does_not_send_protocol_selected() {
        let offer = ProtocolOffer(Some(def_for(ProtocolKind::Greed, "Greed")));
        let mut app = test_app_for_protocol_regression(offer);
        app.update();

        // Zero ProtocolSelected messages.
        let protocol_msgs = app.world().resource::<Messages<ProtocolSelected>>();
        assert_eq!(
            protocol_msgs.iter_current_update_messages().count(),
            0,
            "timer expiry must not emit ProtocolSelected"
        );

        // ActiveProtocols must stay empty.
        let active = app.world().resource::<ActiveProtocols>();
        assert!(
            active.is_empty(),
            "ActiveProtocols must stay empty on timer expiry"
        );

        // One ChangeState<ChipSelectState> message (existing behavior).
        let state_msgs = app
            .world()
            .resource::<Messages<ChangeState<ChipSelectState>>>();
        assert_eq!(
            state_msgs.iter_current_update_messages().count(),
            1,
            "timer expiry must still emit one ChangeState<ChipSelectState>"
        );
    }

    #[test]
    fn timer_expiry_with_protocol_offer_none_does_not_send_protocol_selected() {
        // Edge case of E.1.
        let mut app = test_app_for_protocol_regression(ProtocolOffer(None));
        app.update();

        let protocol_msgs = app.world().resource::<Messages<ProtocolSelected>>();
        assert_eq!(
            protocol_msgs.iter_current_update_messages().count(),
            0,
            "timer expiry with no protocol offer must also not emit ProtocolSelected"
        );

        let state_msgs = app
            .world()
            .resource::<Messages<ChangeState<ChipSelectState>>>();
        assert_eq!(state_msgs.iter_current_update_messages().count(), 1);
    }

    // --- Missing-resources path tests ---

    #[test]
    fn timer_expiry_transitions_without_chip_offers_resource() {
        // When the timer expires but ChipOffers, ChipInventory, and
        // ChipSelectConfig are all absent (Option<Res<...>> = None), the
        // system should still transition without panicking.
        // This exercises the defensive `if let (Some(...), Some(...), Some(...))` guard.
        let mut app = test_app(0.0);
        // test_app does NOT insert ChipOffers, ChipInventory, or ChipSelectConfig.
        // The system receives None for all three Option parameters.
        app.update();

        let msgs = app
            .world()
            .resource::<Messages<ChangeState<ChipSelectState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<ChipSelectState> message even without ChipOffers resource"
        );

        let timer = app.world().resource::<ChipSelectTimer>();
        assert!(
            timer.remaining.abs() < f32::EPSILON,
            "timer should be clamped to 0.0 on expiry, got: {}",
            timer.remaining
        );
    }
}
