use bevy::prelude::*;
use breaker::{
    screen::chip_select::ChipOffers,
    shared::GameState,
};

use crate::{invariants::*, lifecycle::ScenarioConfig, types::InvariantKind};

/// Checks that all expected chip names appear in [`ChipOffers`] during chip select.
///
/// Only fires when `GameState::ChipSelect` is active, `ChipOffers` exists, and the
/// scenario defines `expected_offerings`. Uses `Local<u32>` to track the last
/// checked node index so the check fires at most once per chip-select visit.
pub fn check_chip_offer_expected(
    offers: Option<Res<ChipOffers>>,
    config: Res<ScenarioConfig>,
    state: Res<State<GameState>>,
    run_state: Option<Res<breaker::run::resources::RunState>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut last_checked_node: Local<u32>,
) {
    if *state.get() != GameState::ChipSelect {
        return;
    }

    let Some(offers) = offers else { return };
    let Some(ref expected) = config.definition.expected_offerings else {
        return;
    };

    // Avoid re-checking the same chip-select visit
    let current_node = run_state.as_ref().map_or(0, |rs| rs.node_index);
    if *last_checked_node == current_node + 1 {
        return;
    }
    *last_checked_node = current_node + 1;

    for expected_name in expected {
        let found = offers.0.iter().any(|o| o.name() == expected_name);
        if !found {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::ChipOfferExpected,
                entity: None,
                message: format!(
                    "ChipOfferExpected FAIL frame={} expected='{}' not found in offerings: [{}]",
                    frame.0,
                    expected_name,
                    offers
                        .0
                        .iter()
                        .map(|o| o.name())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::{
        chips::definition::{ChipDefinition, EvolutionIngredient, Rarity},
        effect::{Effect, EffectNode, RootEffect, Target},
        run::resources::RunState,
        screen::chip_select::{ChipOffering, ChipOffers},
        shared::GameState,
    };
    use bevy::{prelude::*, state::app::StatesPlugin};

    use super::check_chip_offer_expected;
    use crate::{
        invariants::{ScenarioFrame, ViolationLog},
        lifecycle::ScenarioConfig,
        types::{InvariantKind, ScenarioDefinition},
    };

    fn make_config(expected: Option<Vec<&str>>) -> ScenarioConfig {
        ScenarioConfig {
            definition: ScenarioDefinition {
                expected_offerings: expected
                    .map(|v| v.into_iter().map(str::to_owned).collect()),
                ..Default::default()
            },
        }
    }

    fn test_app(config: ScenarioConfig, offers: ChipOffers) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_state(GameState::ChipSelect);
        app.insert_resource(config);
        app.insert_resource(offers);
        app.insert_resource(ScenarioFrame(10));
        app.insert_resource(ViolationLog::default());
        app.insert_resource(RunState::default());
        app.add_systems(Update, check_chip_offer_expected);
        app
    }

    fn make_chip_def(name: &str) -> ChipDefinition {
        ChipDefinition {
            name: name.to_owned(),
            description: String::new(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::Piercing(1))],
            }],
            ingredients: None,
            template_name: None,
        }
    }

    #[test]
    fn fires_when_expected_chip_not_in_offers() {
        let config = make_config(Some(vec!["Railgun"]));
        let offers = ChipOffers(vec![
            ChipOffering::Normal(make_chip_def("Piercing Shot")),
            ChipOffering::Normal(make_chip_def("Damage Boost")),
        ]);
        let mut app = test_app(config, offers);
        app.update();

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1, "should fire once for missing Railgun");
        assert_eq!(log.0[0].invariant, InvariantKind::ChipOfferExpected);
        assert!(log.0[0].message.contains("Railgun"));
    }

    #[test]
    fn does_not_fire_when_expected_chip_present() {
        let config = make_config(Some(vec!["Railgun"]));
        let offers = ChipOffers(vec![ChipOffering::Evolution {
            ingredients: vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 3,
            }],
            result: ChipDefinition {
                name: "Railgun".to_owned(),
                description: String::new(),
                rarity: Rarity::Evolution,
                max_stacks: 1,
                effects: vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::Do(Effect::Piercing(5))],
                }],
                ingredients: None,
                template_name: None,
            },
        }]);
        let mut app = test_app(config, offers);
        app.update();

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "should not fire when Railgun is offered");
    }

    #[test]
    fn does_not_fire_when_no_expected_offerings_configured() {
        let config = make_config(None);
        let offers = ChipOffers(vec![
            ChipOffering::Normal(make_chip_def("Piercing Shot")),
        ]);
        let mut app = test_app(config, offers);
        app.update();

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "should not fire when expected_offerings is None"
        );
    }

    #[test]
    fn does_not_fire_outside_chip_select_state() {
        let config = make_config(Some(vec!["Railgun"]));
        let offers = ChipOffers(vec![
            ChipOffering::Normal(make_chip_def("Piercing Shot")),
        ]);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_state(GameState::Playing);
        app.insert_resource(config);
        app.insert_resource(offers);
        app.insert_resource(ScenarioFrame(10));
        app.insert_resource(ViolationLog::default());
        app.insert_resource(RunState::default());
        app.add_systems(Update, check_chip_offer_expected);
        app.update();

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "should not fire when not in ChipSelect state"
        );
    }
}
