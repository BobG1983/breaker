use bevy::{platform::collections::HashMap, prelude::*};
use breaker::shared::PlayingState;

use crate::{invariants::*, types::InvariantKind};

/// Per-entity previous position map for pause-freeze checking.
///
/// Stored in a [`Local`] to track bolt positions between fixed-update ticks.
type PreviousBoltPositions = HashMap<Entity, Vec3>;

/// Checks that physics entities do not move while the game is paused.
///
/// Stores the previous `Transform` for each tagged bolt each tick. When
/// [`PlayingState`] is [`PlayingState::Paused`] and a bolt has moved since
/// last tick, appends a [`ViolationEntry`] with
/// [`InvariantKind::PhysicsFrozenDuringPause`].
///
/// Clears local state when [`PlayingState`] is absent (game is not in `Playing`).
pub fn check_physics_frozen_during_pause(
    bolts: Query<(Entity, &Transform), With<ScenarioTagBolt>>,
    playing_state: Option<Res<State<PlayingState>>>,
    mut previous_positions: Local<PreviousBoltPositions>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(state) = playing_state else {
        previous_positions.clear();
        return;
    };

    let is_paused = **state == PlayingState::Paused;

    for (entity, transform) in &bolts {
        let current_pos = transform.translation;
        if is_paused
            && let Some(&prev_pos) = previous_positions.get(&entity)
            && current_pos != prev_pos
        {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::PhysicsFrozenDuringPause,
                entity: Some(entity),
                message: format!(
                    "PhysicsFrozenDuringPause FAIL frame={} entity={entity:?} moved from {prev_pos:?} to {current_pos:?}",
                    frame.0,
                ),
            });
        }
        previous_positions.insert(entity, current_pos);
    }
}

#[cfg(test)]
mod tests {
    use breaker::shared::GameState;

    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_physics_frozen() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_physics_frozen_during_pause);
        app
    }

    /// When [`PlayingState`] is `Paused` and a tagged bolt moves between ticks,
    /// a [`ViolationEntry`] with [`InvariantKind::PhysicsFrozenDuringPause`] fires.
    ///
    /// Tick 1 (Active): seeds Local with position (100.0, 200.0, 0.0).
    /// Then transition to Paused.
    /// Tick 2 (Paused): bolt moved to (105.0, 200.0, 0.0) → violation.
    #[test]
    fn physics_frozen_during_pause_fires_when_bolt_moves_during_pause() {
        let mut app = test_app_physics_frozen();

        // Enter Playing (needed for PlayingState to be active)
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // process state transition

        let entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
            ))
            .id();

        // Tick 1 in Active: system stores (100.0, 200.0, 0.0) in Local
        tick(&mut app);

        // Transition to Paused
        app.world_mut()
            .resource_mut::<NextState<PlayingState>>()
            .set(PlayingState::Paused);
        app.update(); // process sub-state transition

        // Move the bolt while paused
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation = Vec3::new(105.0, 200.0, 0.0);

        // Tick 2: game is paused and bolt moved → violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one PhysicsFrozenDuringPause violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::PhysicsFrozenDuringPause);
    }

    /// When [`PlayingState`] is `Active`, bolt movement is expected. No violation should fire.
    #[test]
    fn physics_frozen_during_pause_does_not_fire_when_active() {
        let mut app = test_app_physics_frozen();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
            ))
            .id();

        // Tick 1: seeds Local with position
        tick(&mut app);

        // Move bolt (game is Active — movement is legal)
        app.world_mut()
            .entity_mut(entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation = Vec3::new(200.0, 200.0, 0.0);

        // Tick 2: Active state → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when bolt moves during Active state"
        );
    }

    /// When [`PlayingState`] is absent (game not in `Playing`), the system must
    /// do nothing and not panic.
    #[test]
    fn physics_frozen_during_pause_clears_when_playing_state_absent() {
        let mut app = test_app_physics_frozen();

        // Do NOT enter Playing — PlayingState is absent

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
        ));

        // Tick with no PlayingState in world → should not panic, no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when PlayingState is absent"
        );
    }
}
