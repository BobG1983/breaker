//! Cleanup systems that despawn entities on state transitions.

use bevy::prelude::*;

/// Generic cleanup system — despawns all entities with the given marker component.
pub fn cleanup_entities<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{CleanupOnNodeExit, CleanupOnRunEnd};

    #[test]
    fn cleanup_on_node_exit_despawns_marked_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, cleanup_entities::<CleanupOnNodeExit>);

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
        app.add_systems(Update, cleanup_entities::<CleanupOnRunEnd>);

        let marked = app.world_mut().spawn(CleanupOnRunEnd).id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }
}
