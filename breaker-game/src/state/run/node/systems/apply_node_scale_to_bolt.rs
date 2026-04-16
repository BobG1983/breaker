//! System to insert [`NodeScalingFactor`] on bolt entities from [`ActiveNodeLayout`].

use bevy::prelude::*;

use crate::{prelude::*, state::run::node::ActiveNodeLayout};

/// Inserts [`NodeScalingFactor`] on all bolt entities from the active node layout.
///
/// Runs `OnEnter(NodeState::Loading)`. Overwrites any existing `NodeScalingFactor`.
pub(crate) fn apply_node_scale_to_bolt(
    layout: Option<Res<ActiveNodeLayout>>,
    query: Query<Entity, With<Bolt>>,
    mut commands: Commands,
) {
    let Some(layout) = layout else { return };
    for entity in &query {
        commands
            .entity(entity)
            .insert(NodeScalingFactor(layout.0.entity_scale));
    }
}

/// Catches bolts spawned mid-gameplay (e.g. by `spawn_bolts` effect) that
/// missed the `OnEnter(NodeState::Loading)` pass.
///
/// Runs in `FixedUpdate` during `NodeState::Playing`. Only targets bolts
/// that don't already have [`NodeScalingFactor`].
pub(crate) fn apply_node_scale_to_late_bolts(
    layout: Option<Res<ActiveNodeLayout>>,
    query: Query<Entity, (With<Bolt>, Without<NodeScalingFactor>)>,
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
    use bevy::ecs::world::CommandQueue;

    use super::*;
    use crate::{bolt::definition::BoltDefinition, state::run::node::definition::NodePool};

    fn test_bolt_definition() -> BoltDefinition {
        BoltDefinition {
            name:                 "Bolt".to_string(),
            base_speed:           400.0,
            min_speed:            200.0,
            max_speed:            800.0,
            radius:               8.0,
            base_damage:          10.0,
            effects:              vec![],
            color_rgb:            [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical:   5.0,
            min_radius:           None,
            max_radius:           None,
        }
    }

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_system(Update, apply_node_scale_to_bolt)
            .build()
    }

    fn make_layout(entity_scale: f32) -> ActiveNodeLayout {
        ActiveNodeLayout(NodeLayout {
            name: "test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!["S".to_owned(), "S".to_owned()]],
            pool: NodePool::default(),
            entity_scale,
            locks: None,
            sequences: None,
        })
    }

    fn spawn_bolt(app: &mut App) -> Entity {
        let world = app.world_mut();
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, world);
            Bolt::builder()
                .at_position(Vec2::ZERO)
                .definition(&test_bolt_definition())
                .with_velocity(Velocity2D(Vec2::ZERO))
                .primary()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(world);
        entity
    }

    #[test]
    fn inserts_entity_scale_from_active_node_layout() {
        // Given: Bolt entity, ActiveNodeLayout with entity_scale = 0.7
        // When: apply_node_scale_to_bolt runs
        // Then: Bolt has NodeScalingFactor(0.7)
        let mut app = test_app();
        app.insert_resource(make_layout(0.7));

        let entity = spawn_bolt(&mut app);

        app.update();

        let scale = app.world().get::<NodeScalingFactor>(entity).unwrap();
        assert!(
            (scale.0 - 0.7).abs() < f32::EPSILON,
            "expected NodeScalingFactor(0.7), got NodeScalingFactor({})",
            scale.0,
        );
    }

    #[test]
    fn overwrites_existing_entity_scale_on_node_transition() {
        // Given: Bolt with NodeScalingFactor(0.7), ActiveNodeLayout with entity_scale = 1.0
        // When: apply_node_scale_to_bolt runs
        // Then: Bolt has NodeScalingFactor(1.0)
        let mut app = test_app();
        app.insert_resource(make_layout(1.0));

        let entity = spawn_bolt(&mut app);
        app.world_mut()
            .entity_mut(entity)
            .insert(NodeScalingFactor(0.7));

        app.update();

        let scale = app.world().get::<NodeScalingFactor>(entity).unwrap();
        assert!(
            (scale.0 - 1.0).abs() < f32::EPSILON,
            "expected NodeScalingFactor(1.0) after overwrite, got NodeScalingFactor({})",
            scale.0,
        );
    }

    #[test]
    fn no_panic_without_active_node_layout() {
        // Given: Bolt entity, NO ActiveNodeLayout resource
        // When: apply_node_scale_to_bolt runs
        // Then: no panic, no NodeScalingFactor inserted
        let mut app = test_app();
        // Do NOT insert ActiveNodeLayout

        let entity = spawn_bolt(&mut app);

        app.update();

        // Builder does not insert NodeScalingFactor, so it should still be absent.
        assert!(
            app.world().get::<NodeScalingFactor>(entity).is_none(),
            "NodeScalingFactor should not be inserted without ActiveNodeLayout",
        );
    }

    #[test]
    fn late_bolt_receives_node_scaling_factor() {
        // Given: ActiveNodeLayout with entity_scale = 0.6
        // When: a bolt WITHOUT NodeScalingFactor exists and apply_node_scale_to_late_bolts runs
        // Then: the bolt gets NodeScalingFactor(0.6)
        let mut app = TestAppBuilder::new()
            .with_system(Update, super::apply_node_scale_to_late_bolts)
            .insert_resource(make_layout(0.6))
            .build();

        let entity = spawn_bolt(&mut app);
        // Bolt has no NodeScalingFactor (simulates effect-spawned bolt)
        assert!(app.world().get::<NodeScalingFactor>(entity).is_none());

        app.update();

        let scale = app.world().get::<NodeScalingFactor>(entity).unwrap();
        assert!(
            (scale.0 - 0.6).abs() < f32::EPSILON,
            "late bolt should get NodeScalingFactor(0.6), got NodeScalingFactor({})",
            scale.0,
        );
    }

    #[test]
    fn late_bolt_skips_already_tagged() {
        // Given: Bolt WITH NodeScalingFactor(0.7), ActiveNodeLayout with entity_scale = 0.6
        // When: apply_node_scale_to_late_bolts runs
        // Then: NodeScalingFactor stays 0.7 (Without filter excludes it)
        let mut app = TestAppBuilder::new()
            .with_system(Update, super::apply_node_scale_to_late_bolts)
            .insert_resource(make_layout(0.6))
            .build();

        let entity = spawn_bolt(&mut app);
        app.world_mut()
            .entity_mut(entity)
            .insert(NodeScalingFactor(0.7));

        app.update();

        let scale = app.world().get::<NodeScalingFactor>(entity).unwrap();
        assert!(
            (scale.0 - 0.7).abs() < f32::EPSILON,
            "already-tagged bolt should keep NodeScalingFactor(0.7), got NodeScalingFactor({})",
            scale.0,
        );
    }

    #[test]
    fn applies_to_both_primary_and_extra_bolt() {
        // Given: Two Bolt entities (one primary, one ExtraBolt), ActiveNodeLayout(entity_scale=0.7)
        // When: apply_node_scale_to_bolt runs
        // Then: BOTH have NodeScalingFactor(0.7)
        let mut app = test_app();
        app.insert_resource(make_layout(0.7));

        let primary = spawn_bolt(&mut app);
        let extra = {
            let world = app.world_mut();
            let mut queue = CommandQueue::default();
            let entity = {
                let mut commands = Commands::new(&mut queue, world);
                Bolt::builder()
                    .at_position(Vec2::ZERO)
                    .definition(&test_bolt_definition())
                    .with_velocity(Velocity2D(Vec2::ZERO))
                    .extra()
                    .headless()
                    .spawn(&mut commands)
            };
            queue.apply(world);
            entity
        };

        app.update();

        let primary_scale = app.world().get::<NodeScalingFactor>(primary).unwrap();
        assert!(
            (primary_scale.0 - 0.7).abs() < f32::EPSILON,
            "primary bolt should have NodeScalingFactor(0.7), got NodeScalingFactor({})",
            primary_scale.0,
        );

        let extra_scale = app.world().get::<NodeScalingFactor>(extra).unwrap();
        assert!(
            (extra_scale.0 - 0.7).abs() < f32::EPSILON,
            "extra bolt should have NodeScalingFactor(0.7), got NodeScalingFactor({})",
            extra_scale.0,
        );
    }
}
