//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(GameState::MainMenu)` → immediately enters `Playing`
//! - Auto-skips chip selection: `OnEnter(GameState::ChipSelect)` → `NodeTransition`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached or the run ends naturally

use bevy::prelude::*;
use breaker::shared::{GameState, ScenarioLayoutOverride, SelectedArchetype};

use crate::{
    invariants::{ScenarioFrame, ViolationLog},
    types::ScenarioDefinition,
};

/// Loaded scenario configuration, inserted before the app runs.
#[derive(Resource)]
pub struct ScenarioConfig {
    /// The full scenario definition loaded from RON.
    pub definition: ScenarioDefinition,
}

/// Plugin that drives the scenario lifecycle.
pub struct ScenarioLifecycle;

impl Plugin for ScenarioLifecycle {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScenarioFrame>()
            .init_resource::<ViolationLog>()
            .add_systems(OnEnter(GameState::MainMenu), bypass_menu_to_playing)
            .add_systems(OnEnter(GameState::ChipSelect), auto_skip_chip_select)
            .add_systems(
                FixedUpdate,
                (tick_scenario_frame, check_frame_limit).chain(),
            )
            .add_systems(OnEnter(GameState::RunEnd), exit_on_run_end);
    }
}

/// Sets the archetype and layout override, then immediately enters `Playing`.
///
/// This bypasses `RunSetup` entirely — the scenario controls which archetype
/// and layout are used without any user interaction.
fn bypass_menu_to_playing(
    config: Res<ScenarioConfig>,
    mut selected: ResMut<SelectedArchetype>,
    mut layout_override: ResMut<ScenarioLayoutOverride>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    selected.0.clone_from(&config.definition.breaker);
    layout_override.0 = Some(config.definition.layout.clone());
    next_state.set(GameState::Playing);
}

/// Transitions immediately to `NodeTransition`, skipping chip selection UI.
///
/// No chip is applied — the scenario runner does not model chip effects.
fn auto_skip_chip_select(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::NodeTransition);
}

/// Increments [`ScenarioFrame`] by 1 each fixed-update tick.
fn tick_scenario_frame(mut frame: ResMut<ScenarioFrame>) {
    frame.0 += 1;
}

/// Sends [`AppExit::Success`] when [`ScenarioFrame`] reaches `max_frames`.
fn check_frame_limit(
    frame: Res<ScenarioFrame>,
    config: Res<ScenarioConfig>,
    mut exits: MessageWriter<AppExit>,
) {
    if frame.0 >= config.definition.max_frames {
        exits.write(AppExit::Success);
    }
}

/// Sends [`AppExit::Success`] when the run ends naturally.
fn exit_on_run_end(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChaosParams, InputStrategy, InvariantKind, ScenarioDefinition};

    fn make_scenario(max_frames: u32) -> ScenarioDefinition {
        ScenarioDefinition {
            breaker: "aegis".to_owned(),
            layout: "corridor".to_owned(),
            input: InputStrategy::Chaos(ChaosParams {
                seed: 0,
                action_prob: 0.3,
            }),
            max_frames,
            invariants: vec![InvariantKind::BoltInBounds],
            expected_violations: None,
            debug_setup: None,
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // -------------------------------------------------------------------------
    // tick_scenario_frame
    // -------------------------------------------------------------------------

    /// Each fixed-update tick increments [`ScenarioFrame`] by 1.
    #[test]
    fn tick_scenario_frame_increments_by_one_per_tick() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ScenarioFrame(0));
        app.add_systems(FixedUpdate, tick_scenario_frame);

        tick(&mut app);
        assert_eq!(app.world().resource::<ScenarioFrame>().0, 1);

        tick(&mut app);
        assert_eq!(app.world().resource::<ScenarioFrame>().0, 2);
    }

    // -------------------------------------------------------------------------
    // check_frame_limit
    // -------------------------------------------------------------------------

    #[derive(Resource, Default)]
    struct ExitReceived(bool);

    fn capture_exit(mut reader: MessageReader<AppExit>, mut received: ResMut<ExitReceived>) {
        for _ in reader.read() {
            received.0 = true;
        }
    }

    fn exit_test_app(current_frame: u32, max_frames: u32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.insert_resource(ScenarioFrame(current_frame));
        app.insert_resource(ScenarioConfig {
            definition: make_scenario(max_frames),
        });
        app.init_resource::<ExitReceived>();
        app.add_systems(FixedUpdate, (check_frame_limit, capture_exit).chain());
        app
    }

    /// When frame equals `max_frames`, `AppExit` is sent.
    #[test]
    fn check_frame_limit_sends_exit_at_max_frames() {
        let mut app = exit_test_app(100, 100);
        tick(&mut app);
        assert!(
            app.world().resource::<ExitReceived>().0,
            "expected AppExit when frame == max_frames"
        );
    }

    /// When frame exceeds `max_frames`, `AppExit` is still sent.
    #[test]
    fn check_frame_limit_sends_exit_when_frame_exceeds_max() {
        let mut app = exit_test_app(150, 100);
        tick(&mut app);
        assert!(
            app.world().resource::<ExitReceived>().0,
            "expected AppExit when frame > max_frames"
        );
    }

    /// When frame is below `max_frames`, no `AppExit` is sent.
    #[test]
    fn check_frame_limit_does_not_exit_before_max_frames() {
        let mut app = exit_test_app(99, 100);
        tick(&mut app);
        assert!(
            !app.world().resource::<ExitReceived>().0,
            "expected no AppExit when frame < max_frames"
        );
    }
}
