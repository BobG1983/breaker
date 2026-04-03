//! Cleanup systems that despawn entities on state transitions.

use std::{fmt, marker::PhantomData};

use bevy::prelude::*;

/// Generic cleanup system — despawns all entities with the given marker component.
pub(crate) fn cleanup_entities<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Generic cleanup marker — entities with `CleanupOnExit<S>` are despawned
/// when the state machine exits any variant of `S`.
#[derive(Component)]
pub(crate) struct CleanupOnExit<S: States> {
    _marker: PhantomData<S>,
}

impl<S: States> fmt::Debug for CleanupOnExit<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CleanupOnExit<{}>", std::any::type_name::<S>())
    }
}

impl<S: States> Clone for CleanupOnExit<S> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S: States> Default for CleanupOnExit<S> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Despawns all entities with `CleanupOnExit<S>`.
///
/// Register one instance per state exit:
/// `app.add_systems(OnExit(GameState::Playing), cleanup_on_exit::<GameState>);`
pub(crate) fn cleanup_on_exit<S: States>(
    mut commands: Commands,
    query: Query<Entity, With<CleanupOnExit<S>>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        shared::{CleanupOnNodeExit, CleanupOnRunEnd, GameState},
        state::types::NodeState,
    };

    #[test]
    fn cleanup_on_node_exit_despawns_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_entities::<CleanupOnNodeExit>);

        let marked = app.world_mut().spawn(CleanupOnNodeExit).id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }

    #[test]
    fn cleanup_on_run_end_despawns_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_entities::<CleanupOnRunEnd>);

        let marked = app.world_mut().spawn(CleanupOnRunEnd).id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }

    // --- Behavior 1: CleanupOnExit<S> despawns all matching entities ---

    #[test]
    fn cleanup_on_exit_despawns_all_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>);

        let marked_a = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let marked_b = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let marked_c = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let unmarked_a = app.world_mut().spawn_empty().id();
        let unmarked_b = app.world_mut().spawn_empty().id();

        app.update();

        assert!(
            app.world().get_entity(marked_a).is_err(),
            "marked entity A should be despawned"
        );
        assert!(
            app.world().get_entity(marked_b).is_err(),
            "marked entity B should be despawned"
        );
        assert!(
            app.world().get_entity(marked_c).is_err(),
            "marked entity C should be despawned"
        );
        assert!(
            app.world().get_entity(unmarked_a).is_ok(),
            "unmarked entity A should survive"
        );
        assert!(
            app.world().get_entity(unmarked_b).is_ok(),
            "unmarked entity B should survive"
        );
    }

    #[test]
    fn cleanup_on_exit_handles_zero_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>);

        let unmarked_a = app.world_mut().spawn_empty().id();
        let unmarked_b = app.world_mut().spawn_empty().id();

        app.update();

        assert!(
            app.world().get_entity(unmarked_a).is_ok(),
            "unmarked entity A should survive when no marked entities exist"
        );
        assert!(
            app.world().get_entity(unmarked_b).is_ok(),
            "unmarked entity B should survive when no marked entities exist"
        );
    }

    // --- Behavior 2: Type parameter isolation ---

    #[test]
    fn cleanup_on_exit_only_despawns_matching_type_parameter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>)
            .add_systems(Update, cleanup_on_exit::<NodeState>);

        let entity_a = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let entity_b = app
            .world_mut()
            .spawn(CleanupOnExit::<NodeState>::default())
            .id();
        let entity_c = app
            .world_mut()
            .spawn((
                CleanupOnExit::<GameState>::default(),
                CleanupOnExit::<NodeState>::default(),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity_a).is_err(),
            "entity A (GameState marker) should be despawned"
        );
        assert!(
            app.world().get_entity(entity_b).is_err(),
            "entity B (NodeState marker) should be despawned"
        );
        assert!(
            app.world().get_entity(entity_c).is_err(),
            "entity C (both markers) should be despawned"
        );
    }

    #[test]
    fn cleanup_on_exit_query_isolation_only_game_state_registered() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>);

        let entity_a = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let entity_b = app
            .world_mut()
            .spawn(CleanupOnExit::<NodeState>::default())
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity_a).is_err(),
            "entity A (GameState marker) should be despawned by cleanup_on_exit::<GameState>"
        );
        assert!(
            app.world().get_entity(entity_b).is_ok(),
            "entity B (NodeState marker) should survive — no cleanup_on_exit::<NodeState> registered"
        );
    }

    // --- Behavior 3: Coexistence with CleanupOnNodeExit ---

    #[test]
    fn cleanup_on_exit_coexists_with_cleanup_on_node_exit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>)
            .add_systems(Update, cleanup_entities::<CleanupOnNodeExit>);

        let entity_a = app.world_mut().spawn(CleanupOnNodeExit).id();
        let entity_b = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let entity_c = app
            .world_mut()
            .spawn((CleanupOnNodeExit, CleanupOnExit::<GameState>::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity_a).is_err(),
            "entity A (CleanupOnNodeExit only) should be despawned by cleanup_entities"
        );
        assert!(
            app.world().get_entity(entity_b).is_err(),
            "entity B (CleanupOnExit only) should be despawned by cleanup_on_exit"
        );
        assert!(
            app.world().get_entity(entity_c).is_err(),
            "entity C (both markers) should be despawned"
        );
    }

    #[test]
    fn cleanup_on_exit_only_registered_node_exit_entity_survives() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>);

        let entity_a = app.world_mut().spawn(CleanupOnNodeExit).id();
        let entity_b = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let entity_c = app
            .world_mut()
            .spawn((CleanupOnNodeExit, CleanupOnExit::<GameState>::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity_a).is_ok(),
            "entity A (CleanupOnNodeExit only) should survive — no cleanup_entities::<CleanupOnNodeExit> registered"
        );
        assert!(
            app.world().get_entity(entity_b).is_err(),
            "entity B (CleanupOnExit only) should be despawned"
        );
        assert!(
            app.world().get_entity(entity_c).is_err(),
            "entity C (both markers) should be despawned — has CleanupOnExit marker"
        );
    }

    // --- Behavior 4: Coexistence with CleanupOnRunEnd ---

    #[test]
    fn cleanup_on_exit_coexists_with_cleanup_on_run_end() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<GameState>)
            .add_systems(Update, cleanup_entities::<CleanupOnRunEnd>);

        let entity_a = app.world_mut().spawn(CleanupOnRunEnd).id();
        let entity_b = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let entity_c = app
            .world_mut()
            .spawn((CleanupOnRunEnd, CleanupOnExit::<GameState>::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity_a).is_err(),
            "entity A (CleanupOnRunEnd only) should be despawned by cleanup_entities"
        );
        assert!(
            app.world().get_entity(entity_b).is_err(),
            "entity B (CleanupOnExit only) should be despawned by cleanup_on_exit"
        );
        assert!(
            app.world().get_entity(entity_c).is_err(),
            "entity C (both markers) should be despawned"
        );
    }

    #[test]
    fn cleanup_on_run_end_only_registered_exit_entity_survives() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_entities::<CleanupOnRunEnd>);

        let entity_a = app.world_mut().spawn(CleanupOnRunEnd).id();
        let entity_b = app
            .world_mut()
            .spawn(CleanupOnExit::<GameState>::default())
            .id();
        let entity_c = app
            .world_mut()
            .spawn((CleanupOnRunEnd, CleanupOnExit::<GameState>::default()))
            .id();

        app.update();

        assert!(
            app.world().get_entity(entity_a).is_err(),
            "entity A (CleanupOnRunEnd only) should be despawned"
        );
        assert!(
            app.world().get_entity(entity_b).is_ok(),
            "entity B (CleanupOnExit only) should survive — no cleanup_on_exit registered"
        );
        assert!(
            app.world().get_entity(entity_c).is_err(),
            "entity C (both markers) should be despawned — has CleanupOnRunEnd marker"
        );
    }

    // --- Behavior 5: Works with SubStates (NodeState) ---

    #[test]
    fn cleanup_on_exit_works_with_playing_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<NodeState>);

        let marked_a = app
            .world_mut()
            .spawn(CleanupOnExit::<NodeState>::default())
            .id();
        let marked_b = app
            .world_mut()
            .spawn(CleanupOnExit::<NodeState>::default())
            .id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(
            app.world().get_entity(marked_a).is_err(),
            "marked entity A should be despawned"
        );
        assert!(
            app.world().get_entity(marked_b).is_err(),
            "marked entity B should be despawned"
        );
        assert!(
            app.world().get_entity(unmarked).is_ok(),
            "unmarked entity should survive"
        );
    }

    #[test]
    fn cleanup_on_exit_despawns_entire_entity_not_just_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<NodeState>);

        let marked = app
            .world_mut()
            .spawn((
                CleanupOnExit::<NodeState>::default(),
                Transform::default(),
                Name::new("test_entity"),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(marked).is_err(),
            "entire entity should be despawned, not just the marker removed"
        );
    }
}
