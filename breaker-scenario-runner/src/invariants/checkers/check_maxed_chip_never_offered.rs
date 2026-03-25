use bevy::prelude::*;
use breaker::{chips::inventory::ChipInventory, screen::chip_select::ChipOffers};

use crate::{invariants::*, types::InvariantKind};

/// Checks that no chip at max stacks appears in the current offering.
///
/// Iterates all chips in [`ChipOffers`] and records a [`ViolationEntry`] for
/// each chip whose name is at max stacks in [`ChipInventory`]. Skips gracefully
/// when either resource is absent.
pub fn check_maxed_chip_never_offered(
    offers: Option<Res<ChipOffers>>,
    inventory: Option<Res<ChipInventory>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(offers) = offers else { return };
    let Some(inventory) = inventory else { return };

    for offering in &offers.0 {
        let name = offering.name();
        if inventory.is_maxed(name) {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::MaxedChipNeverOffered,
                entity: None,
                message: format!(
                    "MaxedChipNeverOffered FAIL frame={} maxed chip in offering: {name}",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::chips::definition::{ChipDefinition, Rarity, TriggerChain};

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
            effects: vec![TriggerChain::Piercing(1)],
            ingredients: None,
            template_name: None,
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_maxed_chip_never_offered);
        app
    }

    #[test]
    fn maxed_chip_in_offers_detected() {
        let mut app = test_app();

        // Chip "A" has max_stacks=1
        let chip_a = test_chip("A", 1);

        // Insert ChipOffers containing "A"
        app.insert_resource(ChipOffers(vec![
            breaker::screen::chip_select::ChipOffering::Normal(chip_a.clone()),
        ]));

        // Insert ChipInventory where "A" is at 1/1 (maxed)
        let mut inventory = ChipInventory::default();
        let _ = inventory.add_chip("A", &chip_a);
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for maxed chip in offering"
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::MaxedChipNeverOffered,
            "violation must be MaxedChipNeverOffered"
        );
    }

    #[test]
    fn non_maxed_chip_no_violation() {
        let mut app = test_app();

        // Chip "A" has max_stacks=3
        let chip_a = test_chip("A", 3);

        // Insert ChipOffers containing "A"
        app.insert_resource(ChipOffers(vec![
            breaker::screen::chip_select::ChipOffering::Normal(chip_a.clone()),
        ]));

        // Insert ChipInventory where "A" is at 1/3 (not maxed)
        let mut inventory = ChipInventory::default();
        let _ = inventory.add_chip("A", &chip_a);
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when chip is not maxed (1/3 stacks)"
        );
    }

    #[test]
    fn skips_when_no_offers_resource() {
        let mut app = test_app();
        // No ChipOffers inserted — system should skip gracefully
        app.init_resource::<ChipInventory>();

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when ChipOffers resource is absent"
        );
    }

    #[test]
    fn skips_when_no_inventory_resource() {
        let mut app = test_app();
        // No ChipInventory inserted — system should skip gracefully
        let chip_a = test_chip("A", 1);
        app.insert_resource(ChipOffers(vec![
            breaker::screen::chip_select::ChipOffering::Normal(chip_a),
        ]));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when ChipInventory resource is absent"
        );
    }
}
