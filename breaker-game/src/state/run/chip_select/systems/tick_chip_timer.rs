//! System to tick the chip selection countdown timer.

use bevy::prelude::*;

use crate::{
    chips::inventory::ChipInventory,
    state::{
        run::chip_select::{
            ChipSelectConfig,
            resources::{ChipOffering, ChipOffers, ChipSelectTimer},
        },
        types::ChipSelectState,
    },
};

/// Ticks the chip selection timer and auto-advances on expiry.
///
/// Timer expiry transitions to [`ChipSelectState::AnimateOut`] (skip, no chip).
pub(crate) fn tick_chip_timer(
    time: Res<Time>,
    mut timer: ResMut<ChipSelectTimer>,
    mut next_state: ResMut<NextState<ChipSelectState>>,
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

        next_state.set(ChipSelectState::AnimateOut);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::state::types::{AppState, GamePhase, RunPhase};

    fn test_app(remaining: f32) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GamePhase>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<ChipSelectState>()
            .insert_resource(ChipSelectTimer { remaining })
            .add_systems(Update, tick_chip_timer);
        app
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

        let next = app.world().resource::<NextState<ChipSelectState>>();
        assert!(
            format!("{next:?}").contains("AnimateOut"),
            "expected AnimateOut, got: {next:?}"
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

        let next = app.world().resource::<NextState<ChipSelectState>>();
        assert!(
            !format!("{next:?}").contains("AnimateOut"),
            "expected no transition, got: {next:?}"
        );
    }

    // --- Decay-on-expiry tests ---

    use crate::{
        chips::{ChipDefinition, definition::EvolutionIngredient, inventory::ChipInventory},
        effect::{EffectKind, EffectNode},
        state::run::chip_select::{
            ChipSelectConfig,
            resources::{ChipOffering, ChipOffers},
        },
    };

    fn make_offers_3() -> ChipOffers {
        ChipOffers(vec![
            ChipOffering::Normal(ChipDefinition::test(
                "A",
                EffectNode::Do(EffectKind::Piercing(1)),
                3,
            )),
            ChipOffering::Normal(ChipDefinition::test(
                "B",
                EffectNode::Do(EffectKind::Piercing(1)),
                3,
            )),
            ChipOffering::Normal(ChipDefinition::test(
                "C",
                EffectNode::Do(EffectKind::Piercing(1)),
                3,
            )),
        ])
    }

    fn test_app_with_offers(remaining: f32, offers: ChipOffers) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GamePhase>()
            .add_sub_state::<RunPhase>()
            .add_sub_state::<ChipSelectState>()
            .insert_resource(ChipSelectTimer { remaining })
            .insert_resource(offers)
            .init_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .add_systems(Update, tick_chip_timer);
        app
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
                EffectNode::Do(EffectKind::Piercing(1)),
                3,
            )),
            ChipOffering::Evolution {
                ingredients: vec![EvolutionIngredient {
                    chip_name: "X".to_owned(),
                    stacks_required: 2,
                }],
                result: ChipDefinition::test("B+", EffectNode::Do(EffectKind::Piercing(5)), 1),
            },
            ChipOffering::Normal(ChipDefinition::test(
                "C",
                EffectNode::Do(EffectKind::Piercing(1)),
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

    // --- Missing-resources path tests ---

    #[test]
    fn timer_expiry_transitions_without_chip_offers_resource() {
        // When the timer expires but ChipOffers, ChipInventory, and
        // ChipSelectConfig are all absent (Option<Res<...>> = None), the
        // system should still transition to TransitionIn without panicking.
        // This exercises the defensive `if let (Some(...), Some(...), Some(...))` guard.
        let mut app = test_app(0.0);
        // test_app does NOT insert ChipOffers, ChipInventory, or ChipSelectConfig.
        // The system receives None for all three Option parameters.
        app.update();

        let next = app.world().resource::<NextState<ChipSelectState>>();
        assert!(
            format!("{next:?}").contains("AnimateOut"),
            "expected AnimateOut even without ChipOffers resource, got: {next:?}"
        );

        let timer = app.world().resource::<ChipSelectTimer>();
        assert!(
            timer.remaining.abs() < f32::EPSILON,
            "timer should be clamped to 0.0 on expiry, got: {}",
            timer.remaining
        );
    }
}
