use bevy::prelude::*;
use breaker::shared::GameState;

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`GameState`] transitions follow valid paths.
///
/// Forbidden transitions:
/// - `Loading → Playing` (must go through `MainMenu`)
/// - `Loading → RunEnd`
/// - `Playing → Loading`
/// - `RunEnd → Playing` (must go through `MainMenu`)
pub fn check_valid_state_transitions(
    state: Res<State<GameState>>,
    mut previous: ResMut<PreviousGameState>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let current = **state;
    if let Some(prev) = previous.0
        && prev != current
    {
        let forbidden = matches!(
            (prev, current),
            (GameState::Loading | GameState::RunEnd, GameState::Playing)
                | (GameState::Loading, GameState::RunEnd)
                | (GameState::Playing, GameState::Loading)
        );
        if forbidden {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::ValidStateTransitions,
                entity: None,
                message: format!(
                    "ValidStateTransitions FAIL frame={} {prev:?} → {current:?}",
                    frame.0,
                ),
            });
        }
    }
    previous.0 = Some(current);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_valid_transitions() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .init_resource::<PreviousGameState>()
            .add_systems(FixedUpdate, check_valid_state_transitions);
        app
    }

    #[test]
    fn valid_state_transitions_fires_on_loading_to_playing() {
        let mut app = test_app_valid_transitions();
        // Set previous to Loading (the default initial state)
        app.world_mut()
            .insert_resource(PreviousGameState(Some(GameState::Loading)));
        // Transition to Playing (forbidden: skips MainMenu)
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // process state transition
        tick(&mut app); // run checker in FixedUpdate

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::ValidStateTransitions),
            "expected ValidStateTransitions violation for Loading→Playing"
        );
    }

    #[test]
    fn valid_state_transitions_does_not_fire_on_loading_to_main_menu() {
        let mut app = test_app_valid_transitions();
        app.world_mut()
            .insert_resource(PreviousGameState(Some(GameState::Loading)));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::MainMenu);
        app.update();
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        let violations: Vec<_> = log
            .0
            .iter()
            .filter(|v| v.invariant == InvariantKind::ValidStateTransitions)
            .collect();
        assert!(
            violations.is_empty(),
            "Loading→MainMenu should be valid, got: {:?}",
            violations.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }
}
