//! Bridge system for the `node_start` trigger.
//!
//! Runs on `OnEnter(PlayingState::Active)` and fires `Trigger::NodeStart` globally
//! on all entities with `BoundEffects`.
use bevy::prelude::*;

use crate::{
    effect::{
        core::*,
        triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects},
    },
    shared::playing_state::PlayingState,
};

fn bridge_node_start(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(
            &Trigger::NodeStart,
            entity,
            bound,
            &mut staged,
            &mut commands,
        );
        evaluate_staged_effects(&Trigger::NodeStart, entity, &mut staged, &mut commands);
    }
}

/// Register trigger bridge systems.
pub(crate) fn register(app: &mut App) {
    app.add_systems(OnEnter(PlayingState::Active), bridge_node_start);
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{effect::effects::speed_boost::ActiveSpeedBoosts, shared::game_state::GameState};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .add_systems(OnEnter(PlayingState::Active), bridge_node_start);
        app
    }

    fn node_start_bound_effects() -> BoundEffects {
        BoundEffects(vec![(
            "test".into(),
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        )])
    }

    #[test]
    fn bridge_node_start_fires_globally_on_enter_playing_active() {
        let mut app = test_app();

        // Spawn entity with NodeStart chain before transitioning
        app.world_mut().spawn((
            node_start_bound_effects(),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ));

        // Transition to GameState::Playing, which enables PlayingState::Active (default)
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        // OnEnter(PlayingState::Active) should have fired bridge_node_start
        let active = app
            .world_mut()
            .query::<&ActiveSpeedBoosts>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            active.0.len(),
            1,
            "bridge_node_start should fire NodeStart globally on OnEnter(PlayingState::Active)"
        );
        assert!(
            (active.0[0] - 1.5).abs() < f32::EPSILON,
            "SpeedBoost multiplier should be 1.5"
        );
    }
}
