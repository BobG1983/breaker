use bevy::{platform::collections::HashMap, prelude::*};
use breaker::{breaker::components::DashState, state::types::GameState};

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`DashState`] transitions on the tagged breaker follow the legal path.
///
/// Legal transitions: `Idle -> Dashing`, `Settling -> Dashing` (re-dash),
/// `Dashing -> Braking`, `Dashing -> Settling` (dash cancel),
/// `Braking -> Settling`, `Settling -> Idle`, and any state → `Idle`
/// (`reset_breaker` can force Idle from any state on `OnEnter(NodeState::Loading)`).
///
/// Clears tracking on [`GameState`] transitions so that cross-state resets
/// are not flagged.
///
/// Skips the first frame per entity (no previous state stored yet for that entity).
pub fn check_valid_breaker_state(
    breakers: Query<(Entity, &DashState), With<ScenarioTagBreaker>>,
    mut previous: Local<HashMap<Entity, DashState>>,
    game_state: Option<Res<State<GameState>>>,
    mut prev_game_state: Local<Option<GameState>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let current_game = game_state.map(|s| **s);
    // On game-state transition, clear tracking.
    if let (Some(prev_gs), Some(cur_gs)) = (*prev_game_state, current_game)
        && prev_gs != cur_gs
    {
        previous.clear();
    }
    *prev_game_state = current_game;

    for (entity, &current) in &breakers {
        if let Some(&prev) = previous.get(&entity)
            && prev != current
        {
            // Any transition to Idle is legal — `reset_breaker` can force
            // Idle from any state on `OnEnter(NodeState::Loading)`.
            let legal = current == DashState::Idle
                || matches!(
                    (prev, current),
                    (DashState::Idle | DashState::Settling, DashState::Dashing)
                        | (DashState::Dashing, DashState::Braking | DashState::Settling)
                        | (DashState::Braking, DashState::Settling)
                );
            if !legal {
                log.0.push(ViolationEntry {
                    frame:     frame.0,
                    invariant: InvariantKind::ValidDashState,
                    entity:    None,
                    message:   format!(
                        "ValidDashState FAIL frame={} {prev:?} -> {current:?}",
                        frame.0,
                    ),
                });
            }
        }
        previous.insert(entity, current);
    }
    previous.retain(|e, _| breakers.contains(*e));
}
