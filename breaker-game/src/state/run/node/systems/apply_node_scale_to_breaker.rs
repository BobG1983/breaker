//! System to insert [`NodeScalingFactor`] on the breaker from [`ActiveNodeLayout`].

use bevy::prelude::*;

use crate::{
    breaker::components::Breaker, shared::NodeScalingFactor, state::run::node::ActiveNodeLayout,
};

/// Inserts or overwrites [`NodeScalingFactor`] on the breaker entity using the
/// scale from the current [`ActiveNodeLayout`].
///
/// Runs `OnEnter(GameState::Playing)` every node entry. Overwrites any
/// existing `NodeScalingFactor` — the breaker persists across nodes but each
/// layout may specify a different scale.
///
/// Early-returns if no `ActiveNodeLayout` resource exists.
pub(crate) fn apply_node_scale_to_breaker(
    layout: Option<Res<ActiveNodeLayout>>,
    query: Query<Entity, With<Breaker>>,
    mut commands: Commands,
) {
    let Some(layout) = layout else { return };
    for entity in &query {
        commands
            .entity(entity)
            .insert(NodeScalingFactor(layout.0.entity_scale));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        breaker::definition::BreakerDefinition,
        state::run::node::{NodeLayout, definition::NodePool},
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, apply_node_scale_to_breaker);
        app
    }

    fn make_layout(entity_scale: f32) -> NodeLayout {
        NodeLayout {
            name: "test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale,
        }
    }

    #[test]
    fn inserts_entity_scale_from_active_node_layout() {
        // Given: Breaker entity exists, ActiveNodeLayout with entity_scale = 0.7
        // When: apply_node_scale_to_breaker runs
        // Then: Breaker entity has NodeScalingFactor(0.7)
        let mut app = test_app();

        app.insert_resource(ActiveNodeLayout(make_layout(0.7)));

        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();

        app.update();

        let scale = app
            .world()
            .get::<NodeScalingFactor>(entity)
            .expect("breaker should have NodeScalingFactor after system runs");
        assert!(
            (scale.0 - 0.7).abs() < f32::EPSILON,
            "expected NodeScalingFactor(0.7), got NodeScalingFactor({})",
            scale.0,
        );
    }

    #[test]
    fn overwrites_existing_entity_scale_on_node_transition() {
        // Given: Breaker entity with existing NodeScalingFactor(0.7),
        //        ActiveNodeLayout with entity_scale = 0.9
        // When: apply_node_scale_to_breaker runs
        // Then: Breaker entity has NodeScalingFactor(0.9)
        let mut app = test_app();

        app.insert_resource(ActiveNodeLayout(make_layout(0.9)));

        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();
        app.world_mut()
            .entity_mut(entity)
            .insert(NodeScalingFactor(0.7));

        app.update();

        let scale = app
            .world()
            .get::<NodeScalingFactor>(entity)
            .expect("breaker should have NodeScalingFactor after system runs");
        assert!(
            (scale.0 - 0.9).abs() < f32::EPSILON,
            "expected NodeScalingFactor(0.9) after overwrite, got NodeScalingFactor({})",
            scale.0,
        );
    }

    #[test]
    fn does_nothing_without_active_node_layout() {
        // Given: Breaker entity exists, no ActiveNodeLayout resource
        // When: apply_node_scale_to_breaker runs
        // Then: no panic, no NodeScalingFactor inserted
        let mut app = test_app();

        let def = BreakerDefinition::default();
        let entity = app
            .world_mut()
            .spawn(
                Breaker::builder()
                    .definition(&def)
                    .headless()
                    .primary()
                    .build(),
            )
            .id();

        app.update();

        assert!(
            app.world().get::<NodeScalingFactor>(entity).is_none(),
            "breaker should NOT have NodeScalingFactor when no ActiveNodeLayout exists",
        );
    }
}
