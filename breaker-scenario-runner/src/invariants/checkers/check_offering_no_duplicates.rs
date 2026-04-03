use std::collections::HashSet;

use bevy::prelude::*;
use breaker::state::run::chip_select::ChipOffers;

use crate::{invariants::*, types::InvariantKind};

/// Checks that no chip name appears more than once in the current offering.
///
/// Iterates all chips in [`ChipOffers`] and records a [`ViolationEntry`] for
/// each duplicate name encountered. Skips gracefully when the resource is absent.
pub fn check_offering_no_duplicates(
    offers: Option<Res<ChipOffers>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(offers) = offers else { return };

    let mut seen = HashSet::new();
    for offering in &offers.0 {
        let name = offering.name();
        if !seen.insert(name.to_owned()) {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::OfferingNoDuplicates,
                entity: None,
                message: format!(
                    "OfferingNoDuplicates FAIL frame={} duplicate chip name: {name}",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::{
        chips::definition::{ChipDefinition, Rarity},
        effect::{EffectKind, EffectNode, RootEffect, Target},
    };

    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Construct a minimal `ChipDefinition` for testing.
    fn test_chip(name: &str, max_stacks: u32) -> ChipDefinition {
        ChipDefinition {
            name: name.to_owned(),
            description: format!("{name} test description"),
            rarity: Rarity::Common,
            max_stacks,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::Piercing(1))],
            }],
            ingredients: None,
            template_name: None,
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_offering_no_duplicates);
        app
    }

    #[test]
    fn duplicates_in_offers_detected() {
        let mut app = test_app();

        // Insert ChipOffers with two chips sharing the same name "A"
        let chip_a1 = test_chip("A", 3);
        let chip_a2 = test_chip("A", 3);
        app.insert_resource(ChipOffers(vec![
            breaker::state::run::chip_select::ChipOffering::Normal(chip_a1),
            breaker::state::run::chip_select::ChipOffering::Normal(chip_a2),
        ]));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for duplicate chip names in offering"
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::OfferingNoDuplicates,
            "violation must be OfferingNoDuplicates"
        );
    }

    #[test]
    fn no_duplicates_no_violation() {
        use breaker::state::run::chip_select::ChipOffering;
        let mut app = test_app();

        // Insert ChipOffers with three distinct chip names
        let chip_a = test_chip("A", 3);
        let chip_b = test_chip("B", 3);
        let chip_c = test_chip("C", 3);
        app.insert_resource(ChipOffers(vec![
            ChipOffering::Normal(chip_a),
            ChipOffering::Normal(chip_b),
            ChipOffering::Normal(chip_c),
        ]));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when all chip names are distinct"
        );
    }

    #[test]
    fn skips_when_no_offers_resource() {
        let mut app = test_app();
        // ChipOffers not inserted — system should skip gracefully

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when ChipOffers resource is absent"
        );
    }
}
