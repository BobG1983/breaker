//! Menu bypass, chip selection auto-skip, and initial chip seeding.

use bevy::{ecs::system::SystemParam, prelude::*};
use breaker::{
    breaker::{BreakerDefinition, BreakerRegistry, SelectedBreaker, components::BoltLossBehavior},
    chips::{ChipCatalog, inventory::ChipInventory},
    effect_v3::types::{RootNode, StampTarget, Tree},
    shared::RunSeed,
    state::{
        run::{
            NodeLayoutRegistry,
            chip_select::messages::ChipSelected,
            node::{ScenarioLayoutOverride, definition::NodePool},
        },
        types::{ChipSelectState, MenuState},
    },
};

use super::types::{
    ChipSelectionIndex, GODMODE_BREAKER_SENTINEL, PendingBoltEffects, PendingBreakerEffects,
    PendingCellEffects, PendingWallEffects, QUICK_BOSS_LAYOUT_SENTINEL,
    QUICK_CLEAR_LAYOUT_SENTINEL, ScenarioConfig, quick_clear_layout,
};

/// Grouped system parameters for [`bypass_menu_to_playing`].
///
/// Extracted to keep the function under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct BypassExtras<'w, 's> {
    /// Breaker registry for godmode sentinel.
    breaker_registry: ResMut<'w, BreakerRegistry>,
    /// Layout registry for quick-clear sentinel.
    layout_registry:  ResMut<'w, NodeLayoutRegistry>,
    /// Commands for inserting resources (e.g. `PendingBoltEffects`).
    commands:         Commands<'w, 's>,
    /// Chip selection index -- reset on each run.
    chip_index:       ResMut<'w, ChipSelectionIndex>,
}

/// Sets the breaker and layout override, then immediately transitions
/// through the state hierarchy toward `NodeState::Playing`.
///
/// This bypasses `RunSetup` entirely -- the scenario controls which breaker
/// and layout are used without any user interaction.
///
/// Navigates: `MenuState::Main` → `MenuState::Teardown` (the game's
/// teardown routing then advances to `GameState::Run` → `RunState::Node`).
pub fn bypass_menu_to_playing(
    config: Res<ScenarioConfig>,
    mut selected: ResMut<SelectedBreaker>,
    mut layout_override: ResMut<ScenarioLayoutOverride>,
    mut next_menu: ResMut<NextState<MenuState>>,
    mut run_seed: ResMut<RunSeed>,
    mut extras: BypassExtras,
) {
    if config.definition.breaker == GODMODE_BREAKER_SENTINEL {
        extras.breaker_registry.insert(
            "Godmode".to_owned(),
            BreakerDefinition {
                name: "Godmode".to_owned(),
                bolt: "Bolt".to_owned(),
                life_pool: None,
                effects: vec![],
                bolt_loss_behavior: BoltLossBehavior::None,
                ..BreakerDefinition::default()
            },
        );
        "Godmode".clone_into(&mut selected.0);
    } else {
        selected.0.clone_from(&config.definition.breaker);
    }

    if config.definition.layout == QUICK_CLEAR_LAYOUT_SENTINEL {
        extras
            .layout_registry
            .insert(quick_clear_layout(NodePool::default()));
        layout_override.0 = Some("quick_clear".to_owned());
    } else if config.definition.layout == QUICK_BOSS_LAYOUT_SENTINEL {
        extras
            .layout_registry
            .insert(quick_clear_layout(NodePool::Boss));
        layout_override.0 = Some("quick_boss".to_owned());
    } else {
        layout_override.0 = Some(config.definition.layout.clone());
    }

    // Reset chip selection index for each run
    extras.chip_index.0 = 0;

    // Scenarios always use deterministic seed (default 0 when not specified)
    run_seed.0 = Some(config.definition.seed.unwrap_or(0));
    // initial_chips are seeded by `seed_initial_chips` on OnEnter(NodeState::Loading),
    // AFTER reset_run_state clears the inventory on OnExit(MenuState).

    // Dispatch initial_effects to the correct target's pending resource.
    // All targets use deferred pending resources because no game entities
    // exist when this system runs (OnEnter(MenuState::Main)).
    if let Some(ref effects) = config.definition.initial_effects {
        let mut bolt_entries: Vec<(String, Tree)> = Vec::new();
        let mut breaker_entries: Vec<(String, Tree)> = Vec::new();
        let mut cell_entries: Vec<(String, Tree)> = Vec::new();
        let mut wall_entries: Vec<(String, Tree)> = Vec::new();
        for root in effects {
            match root {
                RootNode::Stamp(target, tree) => match target {
                    StampTarget::Bolt | StampTarget::ActiveBolts | StampTarget::EveryBolt => {
                        bolt_entries.push((String::new(), tree.clone()));
                    }
                    StampTarget::Breaker
                    | StampTarget::ActiveBreakers
                    | StampTarget::EveryBreaker => {
                        breaker_entries.push((String::new(), tree.clone()));
                    }
                    StampTarget::ActiveCells | StampTarget::EveryCell => {
                        cell_entries.push((String::new(), tree.clone()));
                    }
                    StampTarget::ActiveWalls | StampTarget::EveryWall => {
                        wall_entries.push((String::new(), tree.clone()));
                    }
                },
                RootNode::Spawn(..) => {
                    // Spawn-based roots not used in scenarios yet
                }
            }
        }
        if !bolt_entries.is_empty() {
            extras
                .commands
                .insert_resource(PendingBoltEffects(bolt_entries));
        }
        if !breaker_entries.is_empty() {
            extras
                .commands
                .insert_resource(PendingBreakerEffects(breaker_entries));
        }
        if !cell_entries.is_empty() {
            extras
                .commands
                .insert_resource(PendingCellEffects(cell_entries));
        }
        if !wall_entries.is_empty() {
            extras
                .commands
                .insert_resource(PendingWallEffects(wall_entries));
        }
    }

    // Transition through the hierarchy: MenuState::Teardown triggers
    // the game's menu_teardown_router → GameState::Run → RunState::Node
    next_menu.set(MenuState::Teardown);
}

/// Transitions immediately to `ChipSelectState::Teardown`, skipping chip selection UI.
///
/// When `chip_selections` is configured, writes the appropriate [`ChipSelected`]
/// message before transitioning.
pub fn auto_skip_chip_select(
    mut next_chip: ResMut<NextState<ChipSelectState>>,
    config: Res<ScenarioConfig>,
    mut index: ResMut<ChipSelectionIndex>,
    mut chip_writer: MessageWriter<ChipSelected>,
) {
    info!("auto_skip_chip_select: transitioning ChipSelect -> Teardown");
    if let Some(ref selections) = config.definition.chip_selections
        && index.0 < selections.len()
    {
        chip_writer.write(ChipSelected {
            name: selections[index.0].clone(),
        });
        index.0 += 1;
    }
    // ChipSelectState::Teardown triggers the game's chip_select_teardown_router
    // → RunState::Node → next node.
    next_chip.set(ChipSelectState::Teardown);
}

/// Seeds `initial_chips` into [`ChipInventory`] from the [`ChipCatalog`].
///
/// Runs on `OnEnter(NodeState::Loading)` -- after `reset_run_state` has cleared the
/// inventory on `OnExit(MenuState)`. This ensures the chips survive the
/// reset and are present when `generate_chip_offerings` checks eligibility.
pub fn seed_initial_chips(
    config: Res<ScenarioConfig>,
    catalog: Option<Res<ChipCatalog>>,
    mut inventory: Option<ResMut<ChipInventory>>,
    mut seeded: Local<bool>,
) {
    if *seeded {
        return;
    }
    let Some(ref chips) = config.definition.initial_chips else {
        return;
    };
    let (Some(catalog), Some(inventory)) = (catalog, &mut inventory) else {
        return;
    };
    *seeded = true;
    for chip_name in chips {
        if let Some(def) = catalog.get(chip_name) {
            let _ = inventory.add_chip(chip_name, def);
        } else {
            warn!("initial_chips: chip '{}' not found in catalog", chip_name);
        }
    }
}

#[cfg(test)]
mod tests {
    // ── Wave 3 Behavior 19: Godmode BreakerDefinition literal sets
    //                        the bolt-loss-behavior field to None explicitly ──
    //
    // The `bypass_menu_to_playing` system has 6+ params and a `BypassExtras`
    // SystemParam wrapper; building a full harness to drive the system is
    // heavy. Use a code-grep test against the source text instead — it
    // catches drift in the production literal regardless of formatting
    // changes around it (substring search).
    //
    // The needle is built at runtime from disjoint pieces so the literal
    // substring does NOT appear anywhere in this file's source text. Without
    // this dance the test would trivially pass at RED because the assert
    // message itself would supply the substring `include_str!` reads back.

    /// Source text of `menu_bypass.rs` — `include_str!` resolves at compile
    /// time, so this assertion fires as soon as the file changes.
    const MENU_BYPASS_SOURCE: &str = include_str!("menu_bypass.rs");

    /// Builds the expected production-literal field declaration without
    /// writing it as a single string anywhere in this file.
    fn expected_field_declaration() -> String {
        // Pieces are joined at runtime; no single source-string contains the
        // full substring we're searching for. Concatenations are runtime so
        // `include_str!` cannot read back the full needle.
        let field = ["bolt", "loss", "behavior"].join("_");
        let ty_prefix = ["BoltLoss", "Behavior"].concat();
        let ty = ty_prefix + "::None";
        format!("{field}: {ty}")
    }

    #[test]
    fn godmode_breaker_definition_literal_sets_bolt_loss_behavior_none() {
        // The production Godmode literal must contain the explicit
        // bolt-loss-behavior field set to None — relying on
        // `..BreakerDefinition::default()` would fall through to
        // `LifeLoss(1)`, which is wrong for Godmode.
        let needle = expected_field_declaration();
        assert!(
            MENU_BYPASS_SOURCE.contains(&needle),
            "menu_bypass.rs Godmode literal must contain `{needle}` so \
             Godmode opts out of life-loss explicitly. Currently the literal \
             relies on `..BreakerDefinition::default()` which yields \
             `LifeLoss(1)`.",
        );
    }

    #[test]
    fn godmode_breaker_literal_still_names_godmode() {
        // Regression guard: confirm we're matching against the right block.
        // Build the needle at runtime via Debug-format so the searched
        // substring isn't present anywhere else in this file's source text.
        let needle = format!("{}: {:?}", "name", "Godmode");
        assert!(
            MENU_BYPASS_SOURCE.contains(&needle),
            "menu_bypass.rs should still declare the Godmode breaker name field",
        );
    }
}
