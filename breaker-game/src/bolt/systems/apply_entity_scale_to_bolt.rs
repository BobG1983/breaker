//! System to insert [`EntityScale`] on bolt entities from [`ActiveNodeLayout`].

use bevy::prelude::*;

use crate::{bolt::components::Bolt, run::node::ActiveNodeLayout, shared::EntityScale};

/// Inserts [`EntityScale`] on all bolt entities from the active node layout.
///
/// Runs `OnEnter(GameState::Playing)`. Overwrites any existing `EntityScale`.
pub(crate) fn apply_entity_scale_to_bolt(
    layout: Option<Res<ActiveNodeLayout>>,
    query: Query<Entity, With<Bolt>>,
    mut commands: Commands,
) {
    let Some(layout) = layout else { return };
    for entity in &query {
        commands
            .entity(entity)
            .insert(EntityScale(layout.0.entity_scale));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::components::ExtraBolt,
        run::node::{NodeLayout, definition::NodePool},
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, apply_entity_scale_to_bolt);
        app
    }

    fn make_layout(entity_scale: f32) -> ActiveNodeLayout {
        ActiveNodeLayout(NodeLayout {
            name: "test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale,
        })
    }

    #[test]
    fn inserts_entity_scale_from_active_node_layout() {
        // Given: Bolt entity, ActiveNodeLayout with entity_scale = 0.7
        // When: apply_entity_scale_to_bolt runs
        // Then: Bolt has EntityScale(0.7)
        let mut app = test_app();
        app.insert_resource(make_layout(0.7));

        let entity = app.world_mut().spawn(Bolt).id();

        app.update();

        let scale = app.world().get::<EntityScale>(entity).unwrap();
        assert!(
            (scale.0 - 0.7).abs() < f32::EPSILON,
            "expected EntityScale(0.7), got EntityScale({})",
            scale.0,
        );
    }

    #[test]
    fn overwrites_existing_entity_scale_on_node_transition() {
        // Given: Bolt with EntityScale(0.7), ActiveNodeLayout with entity_scale = 1.0
        // When: apply_entity_scale_to_bolt runs
        // Then: Bolt has EntityScale(1.0)
        let mut app = test_app();
        app.insert_resource(make_layout(1.0));

        let entity = app.world_mut().spawn((Bolt, EntityScale(0.7))).id();

        app.update();

        let scale = app.world().get::<EntityScale>(entity).unwrap();
        assert!(
            (scale.0 - 1.0).abs() < f32::EPSILON,
            "expected EntityScale(1.0) after overwrite, got EntityScale({})",
            scale.0,
        );
    }

    #[test]
    fn no_panic_without_active_node_layout() {
        // Given: Bolt entity, NO ActiveNodeLayout resource
        // When: apply_entity_scale_to_bolt runs
        // Then: no panic, no EntityScale inserted
        let mut app = test_app();
        // Do NOT insert ActiveNodeLayout

        let entity = app.world_mut().spawn(Bolt).id();

        app.update();

        assert!(
            app.world().get::<EntityScale>(entity).is_none(),
            "EntityScale should not be inserted without ActiveNodeLayout",
        );
    }

    #[test]
    fn applies_to_both_primary_and_extra_bolt() {
        // Given: Two Bolt entities (one primary, one ExtraBolt), ActiveNodeLayout(entity_scale=0.7)
        // When: apply_entity_scale_to_bolt runs
        // Then: BOTH have EntityScale(0.7)
        let mut app = test_app();
        app.insert_resource(make_layout(0.7));

        let primary = app.world_mut().spawn(Bolt).id();
        let extra = app.world_mut().spawn((Bolt, ExtraBolt)).id();

        app.update();

        let primary_scale = app.world().get::<EntityScale>(primary).unwrap();
        assert!(
            (primary_scale.0 - 0.7).abs() < f32::EPSILON,
            "primary bolt should have EntityScale(0.7), got EntityScale({})",
            primary_scale.0,
        );

        let extra_scale = app.world().get::<EntityScale>(extra).unwrap();
        assert!(
            (extra_scale.0 - 0.7).abs() < f32::EPSILON,
            "extra bolt should have EntityScale(0.7), got EntityScale({})",
            extra_scale.0,
        );
    }
}
