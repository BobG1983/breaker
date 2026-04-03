//! Entity tagging and breaker state mapping.

use bevy::prelude::*;
use breaker::{
    bolt::components::Bolt,
    breaker::components::{Breaker, DashState},
    cells::components::Cell,
    walls::components::Wall,
};

use crate::{
    invariants::{
        ScenarioStats, ScenarioTagBolt, ScenarioTagBreaker, ScenarioTagCell, ScenarioTagWall,
    },
    types::ScenarioDashState,
};

/// Tags game entities with scenario marker components for invariant checking.
///
/// Finds all untagged [`Bolt`] entities and inserts [`ScenarioTagBolt`].
/// Finds all untagged [`Breaker`] entities and inserts [`ScenarioTagBreaker`].
/// Finds all untagged [`Cell`] entities and inserts [`ScenarioTagCell`].
/// Finds all untagged [`Wall`] entities and inserts [`ScenarioTagWall`].
/// Runs in `OnEnter(NodeState::Loading)` before [`super::debug_setup::apply_debug_setup`].
pub fn tag_game_entities(
    bolt_query: Query<Entity, (With<Bolt>, Without<ScenarioTagBolt>)>,
    breaker_query: Query<Entity, (With<Breaker>, Without<ScenarioTagBreaker>)>,
    cell_query: Query<Entity, (With<Cell>, Without<ScenarioTagCell>)>,
    wall_query: Query<Entity, (With<Wall>, Without<ScenarioTagWall>)>,
    mut commands: Commands,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let mut bolts_tagged = 0u32;
    let mut breakers_tagged = 0u32;
    let mut cells_tagged = 0u32;
    let mut walls_tagged = 0u32;

    for entity in &bolt_query {
        commands.entity(entity).insert(ScenarioTagBolt);
        bolts_tagged += 1;
    }
    for entity in &breaker_query {
        commands.entity(entity).insert(ScenarioTagBreaker);
        breakers_tagged += 1;
    }
    for entity in &cell_query {
        commands.entity(entity).insert(ScenarioTagCell);
        cells_tagged += 1;
    }
    for entity in &wall_query {
        commands.entity(entity).insert(ScenarioTagWall);
        walls_tagged += 1;
    }

    if let Some(ref mut s) = stats {
        s.bolts_tagged += bolts_tagged;
        s.breakers_tagged += breakers_tagged;
        s.cells_tagged += cells_tagged;
        s.walls_tagged += walls_tagged;
    }
}

/// Maps a [`ScenarioDashState`] to the game crate's [`DashState`].
///
/// Used by [`super::frame_mutations::apply_debug_frame_mutations`] to translate the RON-serializable
/// enum into the Bevy component enum.
#[must_use]
pub const fn map_scenario_dash_state(state: ScenarioDashState) -> DashState {
    match state {
        ScenarioDashState::Idle => DashState::Idle,
        ScenarioDashState::Dashing => DashState::Dashing,
        ScenarioDashState::Braking => DashState::Braking,
        ScenarioDashState::Settling => DashState::Settling,
    }
}
