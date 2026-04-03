//! Shared test helpers for `dispatch_chip_effects` tests.

use bevy::{ecs::world::CommandQueue, prelude::*};

use crate::{
    chips::{definition::ChipDefinition, inventory::ChipInventory, resources::ChipCatalog},
    effect::{BoundEffects, StagedEffects},
    state::{
        run::chip_select::messages::ChipSelected,
        types::{AppState, ChipSelectState, GameState, RunPhase},
    },
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
/// - New state hierarchy registered (`AppState` -> `GameState` -> `RunPhase` -> `ChipSelectState`)
/// - Navigated to `ChipSelectState::Selecting`
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
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .add_sub_state::<RunPhase>()
        .add_sub_state::<ChipSelectState>()
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

    // Navigate to ChipSelectState::Selecting
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunPhase>>()
        .set(RunPhase::ChipSelect);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<ChipSelectState>>()
        .set(ChipSelectState::Selecting);
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
    use rantzsoft_spatial2d::components::Velocity2D;

    use crate::{
        bolt::{components::Bolt, definition::BoltDefinition},
        effect::effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    };

    let def = BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 400.0,
        min_speed: 200.0,
        max_speed: 800.0,
        radius: 8.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    let entity = {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&def)
                .with_velocity(Velocity2D(Vec2::ZERO))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    };

    // Test-specific effect components not handled by builder
    app.world_mut().entity_mut(entity).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts::default(),
        ActiveSpeedBoosts::default(),
    ));

    entity
}

/// Spawn a Breaker entity with effect components.
pub(super) fn spawn_breaker(app: &mut App) -> Entity {
    use crate::{
        breaker::{components::Breaker, definition::BreakerDefinition},
        effect::effects::{
            bump_force::ActiveBumpForces, damage_boost::ActiveDamageBoosts,
            size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts,
        },
    };

    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveBumpForces::default(),
        ActiveSizeBoosts::default(),
        ActiveDamageBoosts::default(),
        ActiveSpeedBoosts::default(),
    ));
    entity
}

/// Spawn a Breaker entity without `BoundEffects` or `StagedEffects`.
pub(super) fn spawn_breaker_bare(app: &mut App) -> Entity {
    use crate::{
        breaker::{components::Breaker, definition::BreakerDefinition},
        effect::effects::{
            bump_force::ActiveBumpForces, damage_boost::ActiveDamageBoosts,
            size_boost::ActiveSizeBoosts, speed_boost::ActiveSpeedBoosts,
        },
    };

    let def = BreakerDefinition::default();
    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert((
        ActiveBumpForces::default(),
        ActiveSizeBoosts::default(),
        ActiveDamageBoosts::default(),
        ActiveSpeedBoosts::default(),
    ));
    entity
}

/// Spawn a Cell entity with effect components.
pub(super) fn spawn_cell(app: &mut App) -> Entity {
    use crate::cells::components::Cell;

    app.world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id()
}

/// Spawn a Wall entity with effect components.
pub(super) fn spawn_wall(app: &mut App) -> Entity {
    use crate::walls::components::Wall;

    app.world_mut()
        .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
        .id()
}
