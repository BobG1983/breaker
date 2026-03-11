//! Cleanup systems that despawn entities on state transitions.

use bevy::prelude::*;

use crate::shared::{CleanupOnNodeExit, CleanupOnRunEnd};

/// Despawns all entities marked with [`CleanupOnNodeExit`].
pub fn cleanup_on_node_exit(mut commands: Commands, query: Query<Entity, With<CleanupOnNodeExit>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Despawns all entities marked with [`CleanupOnRunEnd`].
pub fn cleanup_on_run_end(mut commands: Commands, query: Query<Entity, With<CleanupOnRunEnd>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_on_node_exit_despawns_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, cleanup_on_node_exit);

        let marked = app.world_mut().spawn(CleanupOnNodeExit).id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }

    #[test]
    fn cleanup_on_run_end_despawns_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, cleanup_on_run_end);

        let marked = app.world_mut().spawn(CleanupOnRunEnd).id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }
}
