pub(super) use bevy::state::app::StatesPlugin;
pub(super) use breaker::{
    bolt::components::Bolt,
    breaker::{
        BreakerRegistry,
        components::{Breaker, BreakerState, BreakerWidth},
        messages::BumpGrade,
        resources::ForceBumpGrade,
    },
    effect::{Effect, EffectChains, EffectNode, RootEffect, Target},
    input::resources::InputActions,
    run::{NodeLayoutRegistry, node::resources::NodeTimer},
    shared::{GameState, PlayfieldConfig, PlayingState},
    ui::messages::ChipSelected,
};
pub(super) use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

pub(super) use super::*;
pub(super) use crate::{
    input::{InputDriver, PerfectDriver},
    invariants::{
        PreviousGameState, ScenarioFrame, ScenarioPhysicsFrozen, ScenarioStats, ScenarioTagBolt,
        ScenarioTagBreaker, ViolationLog,
    },
    types::{
        BumpMode, ChaosParams, DebugSetup, ForcedGameState, FrameMutation, InputStrategy,
        InvariantKind, MutationKind, ScenarioBreakerState, ScenarioDefinition, ScriptedParams,
    },
};

pub(super) fn make_scenario(max_frames: u32) -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "aegis".to_owned(),
        layout: "corridor".to_owned(),
        input: InputStrategy::Chaos(ChaosParams { action_prob: 0.3 }),
        max_frames,
        invariants: vec![InvariantKind::BoltInBounds],
        ..Default::default()
    }
}

/// Scenario for lifecycle plugin integration tests — uses `Scripted` input
/// so no randomisation is involved.
pub(super) fn make_lifecycle_test_scenario() -> ScenarioDefinition {
    ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        ..Default::default()
    }
}

/// Builds a test app that uses [`ScenarioLifecycle`] as a plugin, with the
/// minimal state wiring needed to exercise invariant registration.
pub(super) fn lifecycle_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .insert_resource(ScenarioConfig {
            definition: make_lifecycle_test_scenario(),
        })
        .insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
            zone_fraction: 0.667,
        });
    // Resources required by bypass_menu_to_playing
    app.insert_resource(breaker::breaker::SelectedBreaker("Aegis".to_owned()))
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>()
        .init_resource::<BreakerRegistry>()
        .init_resource::<NodeLayoutRegistry>()
        .init_resource::<ChipSelectionIndex>()
        .init_resource::<ForceBumpGrade>();
    // Resources required by inject_scenario_input
    app.init_resource::<InputActions>()
        .add_plugins(ScenarioLifecycle);
    app
}

/// Build a minimal app for testing `apply_debug_setup` in isolation.
pub(super) fn debug_setup_app(definition: ScenarioDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition });
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Build a minimal app for testing `bypass_menu_to_playing` in isolation
/// with all newly required resources.
pub(super) fn bypass_app(definition: ScenarioDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(breaker::breaker::SelectedBreaker::default())
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>()
        .init_resource::<BreakerRegistry>()
        .init_resource::<NodeLayoutRegistry>()
        .init_resource::<ChipSelectionIndex>()
        .add_message::<ChipSelected>()
        .add_systems(Update, bypass_menu_to_playing);
    app
}

/// Build a minimal app for testing `auto_skip_chip_select` in isolation.
pub(super) fn chip_select_app(definition: ScenarioDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .insert_resource(ScenarioConfig { definition })
        .init_resource::<ChipSelectionIndex>()
        .add_message::<ChipSelected>()
        .add_systems(Update, auto_skip_chip_select);
    app
}

/// Resource to capture `ChipSelected` messages for assertion.
#[derive(Resource, Default)]
pub(super) struct CapturedChipSelected(pub Vec<ChipSelected>);

/// System that drains `ChipSelected` messages into [`CapturedChipSelected`].
pub(super) fn collect_chip_selected(
    mut reader: MessageReader<ChipSelected>,
    mut captured: ResMut<CapturedChipSelected>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

/// Build a minimal app for testing `apply_perfect_tracking` and
/// `update_force_bump_grade` with a [`PerfectDriver`].
pub(super) fn perfect_tracking_app(seed: u64, mode: BumpMode) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .insert_resource(ForceBumpGrade::default());
    let driver = InputDriver::Perfect(PerfectDriver::new(seed, mode));
    app.insert_resource(ScenarioInputDriver(driver));
    app
}
