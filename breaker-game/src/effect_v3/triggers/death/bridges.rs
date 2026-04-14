//! Death trigger bridge systems.
//!
//! Each bridge reads `Destroyed<T>` messages and dispatches `Died`, `Killed`, and
//! `DeathOccurred` triggers to entities with bound effects.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::Cell,
    effect_v3::{
        storage::{BoundEffects, StagedEffects},
        types::{EntityKind, Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
    shared::death_pipeline::{Destroyed, GameEntity},
    walls::components::Wall,
};

/// Generic death bridge — reads `Destroyed<T>` and dispatches death triggers.
///
/// For each destroyed entity:
/// - `Died` on the victim entity (local)
/// - `Killed(kind)` on the killer entity (local, if killer exists)
/// - `DeathOccurred(kind)` on all entities (global)
fn on_destroyed_inner<T: GameEntity>(
    kind: EntityKind,
    reader: &mut MessageReader<Destroyed<T>>,
    bound_query: &Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: &Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    commands: &mut Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext::Death {
            victim: msg.victim,
            killer: msg.killer,
        };

        // Local: Died on victim — staged first, then bound.
        if let Ok((bound, staged)) = bound_query.get(msg.victim) {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(
                msg.victim,
                &Trigger::Died,
                &context,
                &staged_trees,
                commands,
            );
            walk_bound_effects(msg.victim, &Trigger::Died, &context, &bound_trees, commands);
        }

        // Local: Killed(kind) and Killed(Any) on killer — staged first
        // for both trigger variants against a single snapshot.
        if let Some(killer) = msg.killer
            && let Ok((bound, staged)) = bound_query.get(killer)
        {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();

            walk_staged_effects(
                killer,
                &Trigger::Killed(kind),
                &context,
                &staged_trees,
                commands,
            );
            walk_staged_effects(
                killer,
                &Trigger::Killed(EntityKind::Any),
                &context,
                &staged_trees,
                commands,
            );

            walk_bound_effects(
                killer,
                &Trigger::Killed(kind),
                &context,
                &bound_trees,
                commands,
            );
            walk_bound_effects(
                killer,
                &Trigger::Killed(EntityKind::Any),
                &context,
                &bound_trees,
                commands,
            );
        }

        // Global: DeathOccurred(kind) on all entities.
        let trigger = Trigger::DeathOccurred(kind);
        for (entity, bound, staged) in global_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, &context, &staged_trees, commands);
            walk_bound_effects(entity, &trigger, &context, &bound_trees, commands);
        }

        // Global: DeathOccurred(Any) on all entities.
        let trigger_any = Trigger::DeathOccurred(EntityKind::Any);
        for (entity, bound, staged) in global_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger_any, &context, &staged_trees, commands);
            walk_bound_effects(entity, &trigger_any, &context, &bound_trees, commands);
        }
    }
}

/// Bridge for cell deaths.
pub(crate) fn on_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Cell,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for bolt deaths.
pub(crate) fn on_bolt_destroyed(
    mut reader: MessageReader<Destroyed<Bolt>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Bolt,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for wall deaths.
pub(crate) fn on_wall_destroyed(
    mut reader: MessageReader<Destroyed<Wall>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Wall,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

/// Bridge for breaker deaths.
pub(crate) fn on_breaker_destroyed(
    mut reader: MessageReader<Destroyed<Breaker>>,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    global_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    on_destroyed_inner(
        EntityKind::Breaker,
        &mut reader,
        &bound_query,
        &global_query,
        &mut commands,
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::{on_bolt_destroyed, on_breaker_destroyed, on_cell_destroyed, on_wall_destroyed};
    use crate::{
        bolt::components::Bolt,
        breaker::components::Breaker,
        cells::components::Cell,
        effect_v3::{
            effects::SpeedBoostConfig,
            stacking::EffectStack,
            storage::{BoundEffects, StagedEffects},
            types::{EffectType, EntityKind, Tree, Trigger},
        },
        shared::{death_pipeline::Destroyed, test_utils::TestAppBuilder},
        walls::components::Wall,
    };

    // -- Test message resources -----------------------------------------------

    #[derive(Resource, Default)]
    struct TestCellDestroyedMessages(Vec<Destroyed<Cell>>);

    #[derive(Resource, Default)]
    struct TestBoltDestroyedMessages(Vec<Destroyed<Bolt>>);

    #[derive(Resource, Default)]
    struct TestBreakerDestroyedMessages(Vec<Destroyed<Breaker>>);

    #[derive(Resource, Default)]
    struct TestWallDestroyedMessages(Vec<Destroyed<Wall>>);

    fn inject_cell_destroyed(
        messages: Res<TestCellDestroyedMessages>,
        mut writer: MessageWriter<Destroyed<Cell>>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn inject_bolt_destroyed(
        messages: Res<TestBoltDestroyedMessages>,
        mut writer: MessageWriter<Destroyed<Bolt>>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn inject_breaker_destroyed(
        messages: Res<TestBreakerDestroyedMessages>,
        mut writer: MessageWriter<Destroyed<Breaker>>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn inject_wall_destroyed(
        messages: Res<TestWallDestroyedMessages>,
        mut writer: MessageWriter<Destroyed<Wall>>,
    ) {
        for msg in &messages.0 {
            writer.write(msg.clone());
        }
    }

    fn cell_death_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<Destroyed<Cell>>()
            .with_resource::<TestCellDestroyedMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_cell_destroyed.before(on_cell_destroyed),
                    on_cell_destroyed,
                ),
            )
            .build()
    }

    fn bolt_death_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<Destroyed<Bolt>>()
            .with_resource::<TestBoltDestroyedMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_bolt_destroyed.before(on_bolt_destroyed),
                    on_bolt_destroyed,
                ),
            )
            .build()
    }

    fn breaker_death_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<Destroyed<Breaker>>()
            .with_resource::<TestBreakerDestroyedMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_breaker_destroyed.before(on_breaker_destroyed),
                    on_breaker_destroyed,
                ),
            )
            .build()
    }

    fn wall_death_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<Destroyed<Wall>>()
            .with_resource::<TestWallDestroyedMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_wall_destroyed.before(on_wall_destroyed),
                    on_wall_destroyed,
                ),
            )
            .build()
    }

    fn tick(app: &mut App) {
        crate::shared::test_utils::tick(app);
    }

    /// Helper to build a When(trigger, Fire(SpeedBoost)) tree.
    fn death_speed_tree(name: &str, trigger: Trigger, multiplier: f32) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                trigger,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                }))),
            ),
        )
    }

    fn destroyed_cell(victim: Entity, killer: Option<Entity>) -> Destroyed<Cell> {
        Destroyed {
            victim,
            killer,
            victim_pos: Vec2::ZERO,
            killer_pos: killer.map(|_| Vec2::ZERO),
            _marker: PhantomData,
        }
    }

    fn destroyed_bolt(victim: Entity, killer: Option<Entity>) -> Destroyed<Bolt> {
        Destroyed {
            victim,
            killer,
            victim_pos: Vec2::ZERO,
            killer_pos: killer.map(|_| Vec2::ZERO),
            _marker: PhantomData,
        }
    }

    fn destroyed_breaker(victim: Entity, killer: Option<Entity>) -> Destroyed<Breaker> {
        Destroyed {
            victim,
            killer,
            victim_pos: Vec2::ZERO,
            killer_pos: killer.map(|_| Vec2::ZERO),
            _marker: PhantomData,
        }
    }

    fn destroyed_wall(victim: Entity, killer: Option<Entity>) -> Destroyed<Wall> {
        Destroyed {
            victim,
            killer,
            victim_pos: Vec2::ZERO,
            killer_pos: killer.map(|_| Vec2::ZERO),
            _marker: PhantomData,
        }
    }

    // -- Behavior 10: DeathOccurred(Any) fires on all entities when Cell dies --

    #[test]
    fn death_occurred_any_fires_on_all_entities_when_cell_dies() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();
        let bolt_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::DeathOccurred(EntityKind::Any),
                1.5,
            )]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_b",
                Trigger::DeathOccurred(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("entity_a should have EffectStack after DeathOccurred(Any)");
        assert_eq!(stack_a.len(), 1);

        let stack_b = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("entity_b should have EffectStack after DeathOccurred(Any)");
        assert_eq!(stack_b.len(), 1);
    }

    // -- Behavior 10 edge case: specific kind still works alongside Any --

    #[test]
    fn death_occurred_specific_fires_alongside_any_when_cell_dies() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();
        let bolt_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                death_speed_tree("chip_any", Trigger::DeathOccurred(EntityKind::Any), 1.5),
                death_speed_tree("chip_cell", Trigger::DeathOccurred(EntityKind::Cell), 2.0),
            ]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack");
        assert_eq!(
            stack.len(),
            2,
            "Both Any and Cell DeathOccurred gates should fire"
        );
    }

    // -- Behavior 11: DeathOccurred(Any) fires when Bolt dies --

    #[test]
    fn death_occurred_any_fires_when_bolt_dies() {
        let mut app = bolt_death_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::DeathOccurred(EntityKind::Any),
                2.0,
            )]))
            .id();

        app.insert_resource(TestBoltDestroyedMessages(vec![destroyed_bolt(
            bolt_entity,
            None,
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after bolt DeathOccurred(Any)");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 11 edge case: killer is None, Killed(Any) does NOT fire --

    #[test]
    fn killed_any_does_not_fire_when_killer_is_none_for_bolt_death() {
        let mut app = bolt_death_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();

        // This entity has Killed(Any) — should NOT fire because killer is None
        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::Killed(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestBoltDestroyedMessages(vec![destroyed_bolt(
            bolt_entity,
            None,
        )]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "Killed(Any) should not fire when killer is None (environmental death)"
        );
    }

    // -- Behavior 12: DeathOccurred(Any) fires when Breaker dies --

    #[test]
    fn death_occurred_any_fires_when_breaker_dies() {
        let mut app = breaker_death_test_app();

        let breaker_entity = app.world_mut().spawn_empty().id();
        let bolt_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::DeathOccurred(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestBreakerDestroyedMessages(vec![destroyed_breaker(
            breaker_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after breaker DeathOccurred(Any)");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 12 edge case: Killed(Any) fires on killer for breaker death --

    #[test]
    fn killed_any_fires_on_killer_when_breaker_dies() {
        let mut app = breaker_death_test_app();

        let breaker_entity = app.world_mut().spawn_empty().id();

        // bolt_entity is the killer, and has Killed(Any) tree
        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::Killed(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestBreakerDestroyedMessages(vec![destroyed_breaker(
            breaker_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("killer bolt_entity should have EffectStack from Killed(Any)");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 13: Killed(Any) fires on killer when Cell dies --

    #[test]
    fn killed_any_fires_on_killer_when_cell_dies() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::Killed(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt_entity (killer) should have EffectStack from Killed(Any)");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 13 edge case: Killed(Any) and Killed(Cell) both fire --

    #[test]
    fn killed_any_and_killed_specific_both_fire_when_cell_dies() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                death_speed_tree("chip_any", Trigger::Killed(EntityKind::Any), 1.5),
                death_speed_tree("chip_cell", Trigger::Killed(EntityKind::Cell), 2.0),
            ]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt_entity should have EffectStack");
        assert_eq!(
            stack.len(),
            2,
            "Both Killed(Any) and Killed(Cell) should fire on the killer"
        );
    }

    // -- Behavior 14: Killed(Any) fires on killer when Wall dies --

    #[test]
    fn killed_any_fires_on_killer_when_wall_dies() {
        let mut app = wall_death_test_app();

        let wall_entity = app.world_mut().spawn_empty().id();

        let bolt_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::Killed(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestWallDestroyedMessages(vec![destroyed_wall(
            wall_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
            .expect("bolt_entity (killer) should have EffectStack from Killed(Any) on wall death");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 14 edge case: killer has no BoundEffects — no panic --

    #[test]
    fn killed_any_no_panic_when_killer_has_no_bound_effects() {
        let mut app = wall_death_test_app();

        let wall_entity = app.world_mut().spawn_empty().id();
        // killer has no BoundEffects
        let bolt_entity = app.world_mut().spawn_empty().id();

        app.insert_resource(TestWallDestroyedMessages(vec![destroyed_wall(
            wall_entity,
            Some(bolt_entity),
        )]));

        // Should not panic
        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
        assert!(
            stack.is_none(),
            "no BoundEffects should mean no EffectStack"
        );
    }

    // -- Behavior 15: Killed(Any) does NOT fire when killer is None --

    #[test]
    fn killed_any_does_not_fire_when_killer_is_none() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::Killed(EntityKind::Any),
                1.5,
            )]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            None,
        )]));

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "Killed(Any) should not fire when killer is None"
        );
    }

    // -- Behavior 15 edge case: both Killed(Any) and DeathOccurred(Any) —
    //    only DeathOccurred fires when killer is None --

    #[test]
    fn only_death_occurred_any_fires_when_killer_is_none() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                death_speed_tree("chip_killed", Trigger::Killed(EntityKind::Any), 1.5),
                death_speed_tree("chip_death", Trigger::DeathOccurred(EntityKind::Any), 2.0),
            ]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            None,
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack from DeathOccurred(Any)");
        assert_eq!(
            stack.len(),
            1,
            "Only DeathOccurred(Any) should fire; Killed(Any) should not (killer is None)"
        );
    }

    // -- Behavior 16: DeathOccurred(Any) and DeathOccurred(specific) both fire --

    #[test]
    fn death_occurred_any_and_specific_both_fire_for_cell_death() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();
        let bolt_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                death_speed_tree("chip_cell", Trigger::DeathOccurred(EntityKind::Cell), 1.5),
                death_speed_tree("chip_any", Trigger::DeathOccurred(EntityKind::Any), 2.0),
            ]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack");
        assert_eq!(
            stack.len(),
            2,
            "Both DeathOccurred(Cell) and DeathOccurred(Any) should fire"
        );
    }

    // -- Behavior 17: Killed(Any) and Killed(specific) both fire --

    #[test]
    fn killed_any_and_killed_specific_both_fire_for_cell_death() {
        let mut app = cell_death_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();

        let killer_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                death_speed_tree("chip_cell", Trigger::Killed(EntityKind::Cell), 1.5),
                death_speed_tree("chip_any", Trigger::Killed(EntityKind::Any), 2.0),
            ]))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(killer_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(killer_entity)
            .expect("killer_entity should have EffectStack");
        assert_eq!(
            stack.len(),
            2,
            "Both Killed(Cell) and Killed(Any) should fire on the killer"
        );
    }

    // -- Behavior 18: DeathOccurred(Any) no-op when no Destroyed messages --

    #[test]
    fn death_occurred_any_no_op_without_destroyed_messages() {
        let mut app = cell_death_test_app();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::DeathOccurred(EntityKind::Any),
                1.5,
            )]))
            .id();

        // No messages injected
        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "no EffectStack should exist when no Destroyed messages are sent"
        );
    }

    // -- Behavior 19: Died trigger unaffected by EntityKind::Any changes --

    #[test]
    fn died_trigger_fires_on_victim_regardless_of_any_changes() {
        let mut app = cell_death_test_app();

        let victim_entity = app
            .world_mut()
            .spawn(BoundEffects(vec![death_speed_tree(
                "chip_a",
                Trigger::Died,
                1.5,
            )]))
            .id();

        let bolt_entity = app.world_mut().spawn_empty().id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            victim_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(victim_entity)
            .expect("victim should have EffectStack from Died trigger");
        assert_eq!(
            stack.len(),
            1,
            "Died (local, no EntityKind) must still fire on the victim"
        );
    }

    // -- Behavior 20: StagedEffects path — Died (local dispatch) walks staged first --

    #[test]
    fn died_trigger_fires_staged_entry_and_consumes_it_entry_specifically() {
        // Verifies the death bridge's local-dispatch staged path:
        // `walk_staged_effects` runs before `walk_bound_effects`, fires the
        // staged entry via `commands.remove_staged_effect` (entry-specific,
        // BoundEffects untouched). Regression guard: a rewire of the bridge
        // that skipped `walk_staged_effects` would silently drop staged
        // entries on death events.
        let mut app = cell_death_test_app();
        let bolt_entity = app.world_mut().spawn_empty().id();
        let staged_fire_tree = death_speed_tree("chip_a", Trigger::Died, 1.5).1;

        let victim_entity = app
            .world_mut()
            .spawn((
                BoundEffects(vec![]),
                StagedEffects(vec![("chip_a".to_string(), staged_fire_tree)]),
            ))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            victim_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(victim_entity)
            .expect("staged When(Died, Fire) must have fired on the victim");
        assert_eq!(stack.len(), 1);

        // Entry-specific consume removed the staged entry from StagedEffects
        // but left BoundEffects untouched.
        let staged = app.world().get::<StagedEffects>(victim_entity).unwrap();
        assert!(staged.0.is_empty(), "staged entry should be consumed");
        let bound = app.world().get::<BoundEffects>(victim_entity).unwrap();
        assert!(bound.0.is_empty(), "BoundEffects must not be touched");
    }

    // -- Behavior 21: StagedEffects path — DeathOccurred(Any) global dispatch walks staged first --

    #[test]
    fn death_occurred_any_fires_staged_entries_on_all_entities() {
        let mut app = cell_death_test_app();
        let cell_entity = app.world_mut().spawn_empty().id();
        let bolt_entity = app.world_mut().spawn_empty().id();
        let staged_tree =
            death_speed_tree("chip_a", Trigger::DeathOccurred(EntityKind::Any), 1.5).1;

        let listener = app
            .world_mut()
            .spawn((
                BoundEffects(vec![]),
                StagedEffects(vec![("chip_a".to_string(), staged_tree)]),
            ))
            .id();

        app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
            cell_entity,
            Some(bolt_entity),
        )]));

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(listener)
            .expect("global staged entry should fire on DeathOccurred(Any)");
        assert_eq!(stack.len(), 1);

        let staged = app.world().get::<StagedEffects>(listener).unwrap();
        assert!(staged.0.is_empty(), "staged entry should be consumed");
    }
}
