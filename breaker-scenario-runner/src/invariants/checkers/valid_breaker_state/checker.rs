use bevy::{platform::collections::HashMap, prelude::*};
use breaker::{breaker::components::DashState, shared::GameState};

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`DashState`] transitions on the tagged breaker follow the legal path.
///
/// Legal transitions: `Idle → Dashing`, `Settling → Dashing` (re-dash),
/// `Dashing → Braking`, `Dashing → Settling` (dash cancel),
/// `Braking → Settling`, `Settling → Idle`. Any other change fires a [`ViolationEntry`] with
/// [`InvariantKind::ValidDashState`].
///
/// Clears tracking on [`GameState`] transitions (e.g., entering `Playing` after a
/// node change) so that forced `reset_breaker` resets to `Idle` are not flagged.
///
/// Skips the first frame per entity (no previous state stored yet for that entity).
pub fn check_valid_breaker_state(
    breakers: Query<(Entity, &DashState), With<ScenarioTagBreaker>>,
    mut previous: Local<HashMap<Entity, DashState>>,
    game_state: Res<State<GameState>>,
    mut prev_game_state: Local<Option<GameState>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let current_game = **game_state;
    // On game-state transition (e.g., entering Playing after a node change),
    // clear tracking — `reset_breaker` may have forcibly set any breaker to
    // `Idle`, which is not a state-machine violation.
    if let Some(prev_gs) = *prev_game_state
        && prev_gs != current_game
    {
        previous.clear();
    }
    *prev_game_state = Some(current_game);

    for (entity, &current) in &breakers {
        if let Some(&prev) = previous.get(&entity)
            && prev != current
        {
            let legal = matches!(
                (prev, current),
                (DashState::Idle | DashState::Settling, DashState::Dashing)
                    | (DashState::Dashing, DashState::Braking | DashState::Settling)
                    | (DashState::Braking, DashState::Settling)
                    | (DashState::Settling, DashState::Idle)
            );
            if !legal {
                log.0.push(ViolationEntry {
                    frame: frame.0,
                    invariant: InvariantKind::ValidDashState,
                    entity: None,
                    message: format!(
                        "ValidDashState FAIL frame={} {prev:?} → {current:?}",
                        frame.0,
                    ),
                });
            }
        }
        previous.insert(entity, current);
    }
    previous.retain(|e, _| breakers.contains(*e));
}
