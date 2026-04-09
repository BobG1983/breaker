//! Cleanup systems that despawn entities on state transitions.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct TestMarker;

    #[test]
    fn cleanup_entities_despawns_marked_entities() {
        use crate::shared::test_utils::TestAppBuilder;
        let mut app = TestAppBuilder::new()
            .with_system(Update, cleanup_entities::<TestMarker>)
            .build();

        let marked = app.world_mut().spawn(TestMarker).id();
        let unmarked = app.world_mut().spawn_empty().id();

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
    }
}
