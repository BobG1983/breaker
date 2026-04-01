//! Menu bypass, chip selection auto-skip, and initial chip seeding.

use bevy::{ecs::system::SystemParam, prelude::*};
use breaker::{
    breaker::{
        BreakerDefinition, BreakerRegistry, SelectedBreaker, definition::BreakerStatOverrides,
    },
    chips::{ChipCatalog, inventory::ChipInventory},
    effect::{EffectNode, RootEffect, Target},
    run::{
        NodeLayoutRegistry,
        node::{ScenarioLayoutOverride, definition::NodePool},
    },
    shared::{GameState, RunSeed},
    ui::messages::ChipSelected,
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
    layout_registry: ResMut<'w, NodeLayoutRegistry>,
    /// Commands for inserting resources (e.g. `PendingBoltEffects`).
    commands: Commands<'w, 's>,
    /// Chip selection index -- reset on each run.
    chip_index: ResMut<'w, ChipSelectionIndex>,
}

/// Sets the breaker and layout override, then immediately enters `Playing`.
///
/// This bypasses `RunSetup` entirely -- the scenario controls which breaker
/// and layout are used without any user interaction.
pub fn bypass_menu_to_playing(
    config: Res<ScenarioConfig>,
    mut selected: ResMut<SelectedBreaker>,
    mut layout_override: ResMut<ScenarioLayoutOverride>,
    mut next_state: ResMut<NextState<GameState>>,
    mut run_seed: ResMut<RunSeed>,
    mut extras: BypassExtras,
) {
    if config.definition.breaker == GODMODE_BREAKER_SENTINEL {
        extras.breaker_registry.insert(
            "Godmode".to_owned(),
            BreakerDefinition {
                name: "Godmode".to_owned(),
                bolt: "Bolt".to_owned(),
                stat_overrides: BreakerStatOverrides::default(),
                life_pool: None,
                effects: vec![],
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
    // initial_chips are seeded by `seed_initial_chips` on OnEnter(Playing),
    // AFTER reset_run_state clears the inventory on OnExit(MainMenu).

    // Dispatch initial_effects to the correct target's pending resource.
    // All targets use deferred pending resources because no game entities
    // exist when this system runs (OnEnter(MainMenu)).
    if let Some(ref effects) = config.definition.initial_effects {
        let mut bolt_entries: Vec<(String, EffectNode)> = Vec::new();
        let mut breaker_entries: Vec<(String, EffectNode)> = Vec::new();
        let mut cell_entries: Vec<(String, EffectNode)> = Vec::new();
        let mut wall_entries: Vec<(String, EffectNode)> = Vec::new();
        for root_effect in effects {
            let RootEffect::On { target, then } = root_effect;
            match target {
                Target::Bolt | Target::AllBolts => {
                    bolt_entries.extend(then.iter().cloned().map(|node| (String::new(), node)));
                }
                Target::Breaker => {
                    breaker_entries.extend(then.iter().cloned().map(|node| (String::new(), node)));
                }
                Target::Cell | Target::AllCells => {
                    cell_entries.extend(then.iter().cloned().map(|node| (String::new(), node)));
                }
                Target::Wall | Target::AllWalls => {
                    wall_entries.extend(then.iter().cloned().map(|node| (String::new(), node)));
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

    next_state.set(GameState::Playing);
}

/// Transitions immediately to `TransitionIn`, skipping chip selection UI.
///
/// When `chip_selections` is configured, writes the appropriate [`ChipSelected`]
/// message before transitioning.
pub fn auto_skip_chip_select(
    mut next_state: ResMut<NextState<GameState>>,
    config: Res<ScenarioConfig>,
    mut index: ResMut<ChipSelectionIndex>,
    mut chip_writer: MessageWriter<ChipSelected>,
) {
    info!("auto_skip_chip_select: transitioning ChipSelect -> TransitionIn");
    if let Some(ref selections) = config.definition.chip_selections
        && index.0 < selections.len()
    {
        chip_writer.write(ChipSelected {
            name: selections[index.0].clone(),
        });
        index.0 += 1;
    }
    next_state.set(GameState::TransitionIn);
}

/// Seeds `initial_chips` into [`ChipInventory`] from the [`ChipCatalog`].
///
/// Runs on `OnEnter(Playing)` -- after `reset_run_state` has cleared the
/// inventory on `OnExit(MainMenu)`. This ensures the chips survive the
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
