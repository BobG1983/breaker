use bevy::prelude::*;

use crate::invariants::*;

/// Validates game state transitions.
///
/// The old monolithic `GameState` had forbidden transitions that this checker
/// validated. The new hierarchical state machine (`AppState` / `GameState` /
/// `RunState` / `NodeState` / etc.) enforces valid transitions structurally
/// via Bevy's sub-state system and is validated by unit + integration tests
/// in the game crate.
///
/// This checker is now a no-op. The [`InvariantKind::ValidStateTransitions`]
/// variant is retained so existing scenario RON files that reference it
/// continue to parse.
pub fn check_valid_state_transitions(
    mut _previous: ResMut<PreviousGameState>,
    frame: Res<ScenarioFrame>,
    mut _log: ResMut<ViolationLog>,
) {
    // No-op — hierarchical state machine transitions are validated by the
    // game crate's unit and integration tests, not the scenario runner.
    let _ = &*frame;
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
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .init_resource::<PreviousGameState>()
            .add_systems(FixedUpdate, check_valid_state_transitions);
        app
    }

    #[test]
    fn valid_state_transitions_checker_is_noop() {
        let mut app = test_app_valid_transitions();

        // Tick several times — no violations should ever be produced
        // since the checker is a no-op.
        for _ in 0..5 {
            tick(&mut app);
        }

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations from no-op checker, got: {:?}",
            log.0.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }
}
