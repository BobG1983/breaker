use bevy::prelude::*;
use breaker::chips::inventory::ChipInventory;

use crate::{invariants::*, types::InvariantKind};

/// Checks that every held chip in [`ChipInventory`] has stacks <= `max_stacks`.
///
/// Iterates all held chips via [`ChipInventory::iter_held_stacks`] and records
/// a [`ViolationEntry`] for each entry where `stacks > max_stacks`. Skips
/// gracefully when the resource is absent.
///
/// A violation here indicates a bug in `add_chip` or `remove_chip` that allowed
/// stacks to exceed the declared maximum — which should never happen in correct
/// code. The invariant is intentionally triggered in the self-test scenario via
/// a `MutationKind::InjectOverStackedChip` frame mutation.
pub fn check_chip_stacks_consistent(
    inventory: Option<Res<ChipInventory>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(inventory) = inventory else { return };

    for (name, stacks, max_stacks) in inventory.iter_held_stacks() {
        if stacks > max_stacks {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::ChipStacksConsistent,
                entity: None,
                message: format!(
                    "ChipStacksConsistent FAIL frame={} chip={name} stacks={stacks} max_stacks={max_stacks}",
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

    fn test_chip(name: &str, max_stacks: u32) -> ChipDefinition {
        ChipDefinition {
            name: name.to_owned(),
            description: format!("{name} test description"),
            rarity: Rarity::Common,
            max_stacks,
            effects: vec![TriggerChain::Piercing(1)],
            ingredients: None,
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_chip_stacks_consistent);
        app
    }

    #[test]
    fn chip_stacks_at_max_no_violation() {
        let mut app = test_app();

        let def = test_chip("A", 2);
        let mut inventory = ChipInventory::default();
        let _ = inventory.add_chip("A", &def);
        let _ = inventory.add_chip("A", &def); // stacks=2, max=2 — at cap, legal
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when stacks == max_stacks (2/2)"
        );
    }

    #[test]
    fn chip_stacks_below_max_no_violation() {
        let mut app = test_app();

        let def = test_chip("A", 3);
        let mut inventory = ChipInventory::default();
        let _ = inventory.add_chip("A", &def); // stacks=1, max=3
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when stacks (1) is below max_stacks (3)"
        );
    }

    #[test]
    fn skips_when_no_inventory_resource() {
        let mut app = test_app();
        // ChipInventory not inserted — system should skip gracefully

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when ChipInventory resource is absent"
        );
    }

    #[test]
    fn empty_inventory_no_violation() {
        let mut app = test_app();
        app.insert_resource(ChipInventory::default());

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations on default empty ChipInventory"
        );
    }

    #[test]
    fn multiple_chips_all_valid_no_violation() {
        let mut app = test_app();

        let def_a = test_chip("A", 3);
        let def_b = test_chip("B", 2);
        let def_c = test_chip("C", 1);
        let mut inventory = ChipInventory::default();
        let _ = inventory.add_chip("A", &def_a); // 1/3
        let _ = inventory.add_chip("A", &def_a); // 2/3
        let _ = inventory.add_chip("B", &def_b); // 1/2
        let _ = inventory.add_chip("C", &def_c); // 1/1 — maxed but valid
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when all chip stacks are at or below max_stacks"
        );
    }

    /// Verifies the `stacks > max_stacks` boundary:
    /// `stacks == max_stacks` must NOT fire — the invariant is `>`, not `>=`.
    #[test]
    fn stacks_equal_to_max_is_boundary_not_a_violation() {
        let mut app = test_app();

        // Single-stack chip at exactly 1/1 — the tightest legal boundary.
        let def = test_chip("SingleStack", 1);
        let mut inventory = ChipInventory::default();
        let _ = inventory.add_chip("SingleStack", &def); // stacks=1, max=1
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "stacks==max_stacks (1/1) must NOT fire ChipStacksConsistent — only stacks>max fires"
        );
    }

    /// Verifies the violation fires when stacks exceed `max_stacks`.
    ///
    /// Uses [`ChipInventory::force_insert_entry`] to bypass normal cap enforcement
    /// and inject an over-stacked entry (stacks=3, max=2).
    #[test]
    fn over_stacked_chip_fires_violation() {
        let mut app = test_app();

        let mut inventory = ChipInventory::default();
        inventory.force_insert_entry("OverStacked", 3, 2); // stacks=3 > max=2
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for over-stacked chip (3 > 2)"
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::ChipStacksConsistent,
            "violation must be ChipStacksConsistent"
        );
        assert!(
            log.0[0].message.contains("OverStacked"),
            "violation message must include chip name 'OverStacked', got: {}",
            log.0[0].message
        );
    }

    /// Multiple over-stacked chips each produce a violation.
    #[test]
    fn multiple_over_stacked_chips_each_fire_violation() {
        let mut app = test_app();

        let mut inventory = ChipInventory::default();
        inventory.force_insert_entry("ChipA", 5, 3); // 5 > 3
        inventory.force_insert_entry("ChipB", 2, 1); // 2 > 1
        inventory.force_insert_entry("ChipC", 1, 2); // 1 <= 2 — valid
        app.insert_resource(inventory);

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::ChipStacksConsistent)
                .count(),
            2,
            "expected exactly 2 violations (ChipA and ChipB), got: {:?}",
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::ChipStacksConsistent)
                .map(|v| &v.message)
                .collect::<Vec<_>>()
        );
    }
}
