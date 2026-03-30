//! Shared test helpers for `dispatch_chip_effects` tests.

use bevy::prelude::*;

use crate::{
    chips::{definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog},
    effect::{BoundEffects, StagedEffects},
    shared::GameState,
    ui::messages::ChipSelected,
};

/// Resource holding messages to be sent before the dispatch system runs.
#[derive(Resource, Default)]
pub(super) struct PendingChipSelections(pub Vec<ChipSelected>);

/// System that writes pending `ChipSelected` messages before dispatch runs.
pub(super) fn send_chip_selections(
    pending: Res<PendingChipSelections>,
    mut writer: MessageWriter<ChipSelected>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

/// Build a minimal test app wired for `dispatch_chip_effects`.
///
/// - `MinimalPlugins` + `StatesPlugin`
/// - `GameState` initialised and set to `ChipSelect`
/// - `ChipSelected` message registered
/// - `ChipInventory` default resource
/// - `ChipCatalog` default resource (caller inserts chips after)
/// - `PendingChipSelections` resource
/// - `send_chip_selections` runs before `dispatch_chip_effects` in `Update`
pub(super) fn test_app() -> App {
    use crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .init_state::<GameState>()
        .add_message::<ChipSelected>()
        .init_resource::<ChipInventory>()
        .init_resource::<ChipCatalog>()
        .init_resource::<PendingChipSelections>();

    // Add the system without run_if guard for direct testing.
    // The plugin_builds test in plugin.rs covers the state guard.
    app.add_systems(
        Update,
        (
            send_chip_selections.before(dispatch_chip_effects),
            dispatch_chip_effects,
        ),
    );

    // Force GameState to ChipSelect
    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::ChipSelect);
    // One update to apply the state transition
    app.update();

    app
}

/// Insert a chip definition into the app's `ChipCatalog`.
pub(super) fn insert_chip(app: &mut App, def: ChipDefinition) {
    app.world_mut().resource_mut::<ChipCatalog>().insert(def);
}

/// Queue a `ChipSelected` message to be sent on the next update.
pub(super) fn select_chip(app: &mut App, name: &str) {
    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .push(ChipSelected {
            name: name.to_owned(),
        });
}

/// Spawn a Bolt entity with effect components.
pub(super) fn spawn_bolt(app: &mut App) -> Entity {
    use crate::{
        bolt::components::Bolt,
        effect::effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    };

    app.world_mut()
        .spawn((
            Bolt,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveDamageBoosts::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id()
}

/// Spawn a Bolt entity without `BoundEffects` or `StagedEffects`.
pub(super) fn spawn_bolt_bare(app: &mut App) -> Entity {
    use crate::{
        bolt::components::Bolt,
        effect::effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    };

    app.world_mut()
        .spawn((
            Bolt,
            ActiveDamageBoosts::default(),
            ActiveSpeedBoosts::default(),
        ))
        .id()
}

/// Spawn a Breaker entity with effect components.
pub(super) fn spawn_breaker(app: &mut App) -> Entity {
    use crate::{
        breaker::components::Breaker,
        effect::effects::{bump_force::ActiveBumpForces, size_boost::ActiveSizeBoosts},
    };

    app.world_mut()
        .spawn((
            Breaker,
            BoundEffects::default(),
            StagedEffects::default(),
            ActiveBumpForces::default(),
            ActiveSizeBoosts::default(),
        ))
        .id()
}

/// Spawn a Breaker entity without `BoundEffects` or `StagedEffects`.
pub(super) fn spawn_breaker_bare(app: &mut App) -> Entity {
    use crate::{
        breaker::components::Breaker,
        effect::effects::{bump_force::ActiveBumpForces, size_boost::ActiveSizeBoosts},
    };

    app.world_mut()
        .spawn((
            Breaker,
            ActiveBumpForces::default(),
            ActiveSizeBoosts::default(),
        ))
        .id()
}

/// Spawn a Cell entity with effect components.
pub(super) fn spawn_cell(app: &mut App) -> Entity {
    use crate::cells::components::Cell;

    app.world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id()
}

/// Spawn a Cell entity without `BoundEffects` or `StagedEffects`.
pub(super) fn spawn_cell_bare(app: &mut App) -> Entity {
    use crate::cells::components::Cell;

    app.world_mut().spawn(Cell).id()
}

/// Spawn a Wall entity with effect components.
pub(super) fn spawn_wall(app: &mut App) -> Entity {
    use crate::wall::components::Wall;

    app.world_mut()
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id()
}

/// Spawn a Wall entity without `BoundEffects` or `StagedEffects`.
pub(super) fn spawn_wall_bare(app: &mut App) -> Entity {
    use crate::wall::components::Wall;

    app.world_mut().spawn(Wall).id()
}
