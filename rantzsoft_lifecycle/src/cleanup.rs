//! Generic cleanup component and system for state-scoped entity despawning.

use std::{fmt, marker::PhantomData};

use bevy::prelude::*;

/// Marks entities for despawn when state `S` enters teardown.
///
/// The crate provides the component; the game decides when cleanup runs
/// by wiring `cleanup_on_exit::<S>` to the appropriate schedule
/// (e.g. `OnEnter(NodeState::Teardown)`).
#[derive(Component)]
pub struct CleanupOnExit<S: States> {
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

/// Despawns all entities with [`CleanupOnExit<S>`].
///
/// Register on the appropriate schedule for your state:
/// ```text
/// app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);
/// ```
pub fn cleanup_on_exit<S: States>(
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

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
    }

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum OtherState {
        #[default]
        X,
    }

    #[test]
    fn cleanup_despawns_all_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<TestState>);

        let marked_a = app
            .world_mut()
            .spawn(CleanupOnExit::<TestState>::default())
            .id();
        let marked_b = app
            .world_mut()
            .spawn(CleanupOnExit::<TestState>::default())
            .id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked_a).is_err());
        assert!(app.world().get_entity(marked_b).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }

    #[test]
    fn cleanup_only_affects_matching_type_parameter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<TestState>);

        let test_marked = app
            .world_mut()
            .spawn(CleanupOnExit::<TestState>::default())
            .id();
        let other_marked = app
            .world_mut()
            .spawn(CleanupOnExit::<OtherState>::default())
            .id();

        app.update();

        assert!(
            app.world().get_entity(test_marked).is_err(),
            "TestState marker should be despawned"
        );
        assert!(
            app.world().get_entity(other_marked).is_ok(),
            "OtherState marker should survive — no cleanup_on_exit::<OtherState> registered"
        );
    }

    #[test]
    fn cleanup_handles_zero_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, cleanup_on_exit::<TestState>);

        let unmarked = app.world_mut().spawn_empty().id();
        app.update();

        assert!(app.world().get_entity(unmarked).is_ok());
    }

    #[test]
    fn cleanup_debug_includes_type_name() {
        let marker = CleanupOnExit::<TestState>::default();
        let debug = format!("{marker:?}");
        assert!(debug.contains("TestState"));
    }
}
