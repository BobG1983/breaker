//! Impact trigger bridge systems.
//!
//! Each bridge reads collision messages, builds a [`TriggerContext`], and dispatches
//! the corresponding trigger to entities with bound effects.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall},
    cells::messages::CellImpactWall,
    effect_v3::{
        storage::{BoundEffects, StagedEffects},
        types::{EntityKind, Trigger, TriggerContext},
        walking::{walk_bound_effects, walk_staged_effects},
    },
};

/// Bundled message readers for all collision types — avoids `too_many_arguments`.
#[derive(SystemParam)]
pub(crate) struct ImpactReaders<'w, 's> {
    bolt_cell:    MessageReader<'w, 's, BoltImpactCell>,
    bolt_wall:    MessageReader<'w, 's, BoltImpactWall>,
    bolt_breaker: MessageReader<'w, 's, BoltImpactBreaker>,
    breaker_cell: MessageReader<'w, 's, BreakerImpactCell>,
    breaker_wall: MessageReader<'w, 's, BreakerImpactWall>,
    cell_wall:    MessageReader<'w, 's, CellImpactWall>,
}

/// Local bridge: fires `Impacted(entity_kind)` on entities involved in a collision.
///
/// For each collision message, both participants receive the trigger with the
/// `EntityKind` of the *other* entity. E.g., when a bolt hits a cell, the bolt
/// gets `Impacted(Cell)` and the cell gets `Impacted(Bolt)`.
pub(crate) fn on_impacted(
    mut readers: ImpactReaders,
    bound_query: Query<(&BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    for msg in readers.bolt_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.cell,
        };
        walk_local_impact(
            msg.bolt,
            EntityKind::Cell,
            msg.cell,
            EntityKind::Bolt,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.bolt_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.wall,
        };
        walk_local_impact(
            msg.bolt,
            EntityKind::Wall,
            msg.wall,
            EntityKind::Bolt,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.bolt_breaker.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.breaker,
        };
        walk_local_impact(
            msg.bolt,
            EntityKind::Breaker,
            msg.breaker,
            EntityKind::Bolt,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.breaker_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.cell,
        };
        walk_local_impact(
            msg.breaker,
            EntityKind::Cell,
            msg.cell,
            EntityKind::Breaker,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.breaker_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.wall,
        };
        walk_local_impact(
            msg.breaker,
            EntityKind::Wall,
            msg.wall,
            EntityKind::Breaker,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
    for msg in readers.cell_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.cell,
            impactee: msg.wall,
        };
        walk_local_impact(
            msg.cell,
            EntityKind::Wall,
            msg.wall,
            EntityKind::Cell,
            &ctx,
            &bound_query,
            &mut commands,
        );
    }
}

/// Global bridge: fires `ImpactOccurred(entity_kind)` on all entities with bound
/// effects when a collision involving the given entity kind happens.
pub(crate) fn on_impact_occurred(
    mut readers: ImpactReaders,
    bound_query: Query<(Entity, &BoundEffects, Option<&StagedEffects>)>,
    mut commands: Commands,
) {
    // Collect all entity kinds involved in collisions this frame
    let mut kinds = Vec::new();
    for msg in readers.bolt_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.cell,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.bolt_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.bolt_breaker.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.bolt,
            impactee: msg.breaker,
        };
        kinds.push((EntityKind::Bolt, ctx.clone()));
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.breaker_cell.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.cell,
        };
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.breaker_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.breaker,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Breaker, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }
    for msg in readers.cell_wall.read() {
        let ctx = TriggerContext::Impact {
            impactor: msg.cell,
            impactee: msg.wall,
        };
        kinds.push((EntityKind::Cell, ctx.clone()));
        kinds.push((EntityKind::Wall, ctx.clone()));
        kinds.push((EntityKind::Any, ctx));
    }

    for (kind, ctx) in &kinds {
        let trigger = Trigger::ImpactOccurred(*kind);
        for (entity, bound, staged) in bound_query.iter() {
            let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
            let bound_trees = bound.0.clone();
            walk_staged_effects(entity, &trigger, ctx, &staged_trees, &mut commands);
            walk_bound_effects(entity, &trigger, ctx, &bound_trees, &mut commands);
        }
    }
}

/// Walk effects on both collision participants (local dispatch).
///
/// Walks staged entries before bound entries for each entity so a
/// freshly-armed inner gate does not match the same-tick trigger.
fn walk_local_impact(
    entity_a: Entity,
    kind_b: EntityKind,
    entity_b: Entity,
    kind_a: EntityKind,
    context: &TriggerContext,
    bound_query: &Query<(&BoundEffects, Option<&StagedEffects>)>,
    commands: &mut Commands,
) {
    // Entity A gets Impacted(kind_of_B) — and also Impacted(Any).
    if let Ok((bound, staged)) = bound_query.get(entity_a) {
        let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
        let bound_trees = bound.0.clone();

        // Walk staged first for both trigger variants against the same
        // snapshot — matches the existing clone-once pattern for bound.
        walk_staged_effects(
            entity_a,
            &Trigger::Impacted(kind_b),
            context,
            &staged_trees,
            commands,
        );
        walk_staged_effects(
            entity_a,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &staged_trees,
            commands,
        );

        walk_bound_effects(
            entity_a,
            &Trigger::Impacted(kind_b),
            context,
            &bound_trees,
            commands,
        );
        walk_bound_effects(
            entity_a,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &bound_trees,
            commands,
        );
    }
    // Entity B gets Impacted(kind_of_A) — and also Impacted(Any).
    if let Ok((bound, staged)) = bound_query.get(entity_b) {
        let staged_trees = staged.map(|s| s.0.clone()).unwrap_or_default();
        let bound_trees = bound.0.clone();

        walk_staged_effects(
            entity_b,
            &Trigger::Impacted(kind_a),
            context,
            &staged_trees,
            commands,
        );
        walk_staged_effects(
            entity_b,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &staged_trees,
            commands,
        );

        walk_bound_effects(
            entity_b,
            &Trigger::Impacted(kind_a),
            context,
            &bound_trees,
            commands,
        );
        walk_bound_effects(
            entity_b,
            &Trigger::Impacted(EntityKind::Any),
            context,
            &bound_trees,
            commands,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::on_impact_occurred;
    use crate::{
        bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
        breaker::messages::{BreakerImpactCell, BreakerImpactWall},
        cells::messages::CellImpactWall,
        effect_v3::{
            effects::SpeedBoostConfig,
            stacking::EffectStack,
            storage::BoundEffects,
            types::{EffectType, EntityKind, Tree, Trigger},
        },
        shared::test_utils::TestAppBuilder,
    };

    // -- Test message resource ------------------------------------------------

    /// Resource to inject all six collision message types into the test app.
    #[derive(Resource, Default)]
    struct TestImpactMessages {
        bolt_cell:    Vec<BoltImpactCell>,
        bolt_wall:    Vec<BoltImpactWall>,
        bolt_breaker: Vec<BoltImpactBreaker>,
        breaker_cell: Vec<BreakerImpactCell>,
        breaker_wall: Vec<BreakerImpactWall>,
        cell_wall:    Vec<CellImpactWall>,
    }

    /// System that writes all queued impact messages from the test resource.
    fn inject_impacts(
        messages: Res<TestImpactMessages>,
        mut w_bolt_cell: MessageWriter<BoltImpactCell>,
        mut w_bolt_wall: MessageWriter<BoltImpactWall>,
        mut w_bolt_breaker: MessageWriter<BoltImpactBreaker>,
        mut w_breaker_cell: MessageWriter<BreakerImpactCell>,
        mut w_breaker_wall: MessageWriter<BreakerImpactWall>,
        mut w_cell_wall: MessageWriter<CellImpactWall>,
    ) {
        for msg in &messages.bolt_cell {
            w_bolt_cell.write(msg.clone());
        }
        for msg in &messages.bolt_wall {
            w_bolt_wall.write(msg.clone());
        }
        for msg in &messages.bolt_breaker {
            w_bolt_breaker.write(msg.clone());
        }
        for msg in &messages.breaker_cell {
            w_breaker_cell.write(msg.clone());
        }
        for msg in &messages.breaker_wall {
            w_breaker_wall.write(msg.clone());
        }
        for msg in &messages.cell_wall {
            w_cell_wall.write(msg.clone());
        }
    }

    fn bridge_test_app() -> App {
        TestAppBuilder::new()
            .with_message::<BoltImpactCell>()
            .with_message::<BoltImpactWall>()
            .with_message::<BoltImpactBreaker>()
            .with_message::<BreakerImpactCell>()
            .with_message::<BreakerImpactWall>()
            .with_message::<CellImpactWall>()
            .with_resource::<TestImpactMessages>()
            .with_system(
                FixedUpdate,
                (
                    inject_impacts.before(on_impact_occurred),
                    on_impact_occurred,
                ),
            )
            .build()
    }

    fn tick(app: &mut App) {
        crate::shared::test_utils::tick(app);
    }

    /// Helper to build a When(ImpactOccurred(kind), Fire(SpeedBoost)) tree.
    fn impact_occurred_speed_tree(name: &str, kind: EntityKind, multiplier: f32) -> (String, Tree) {
        (
            name.to_string(),
            Tree::When(
                Trigger::ImpactOccurred(kind),
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                }))),
            ),
        )
    }

    // -- Behavior 1: ImpactOccurred(Any) fires on all entities for BoltImpactCell --

    #[test]
    fn impact_occurred_any_fires_on_all_entities_for_bolt_impact_cell() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        let entity_b = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_b",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_cell: vec![BoltImpactCell {
                bolt: bolt_entity,
                cell: cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("entity_a should have EffectStack after ImpactOccurred(Any)");
        assert_eq!(stack_a.len(), 1, "entity_a should have 1 effect entry");

        let stack_b = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_b)
            .expect("entity_b should have EffectStack after ImpactOccurred(Any)");
        assert_eq!(stack_b.len(), 1, "entity_b should have 1 effect entry");
    }

    // -- Behavior 1 edge case: specific kind also fires alongside Any --

    #[test]
    fn impact_occurred_specific_kind_fires_alongside_any_for_bolt_impact_cell() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        // Entity with both Any and Cell gates
        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                impact_occurred_speed_tree("chip_any", EntityKind::Any, 1.5),
                impact_occurred_speed_tree("chip_cell", EntityKind::Cell, 2.0),
            ]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_cell: vec![BoltImpactCell {
                bolt: bolt_entity,
                cell: cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack");
        assert_eq!(
            stack.len(),
            2,
            "Both Any and Cell gates should fire, yielding 2 entries"
        );
    }

    // -- Behavior 2: ImpactOccurred(Any) fires for BoltImpactWall --

    #[test]
    fn impact_occurred_any_fires_for_bolt_impact_wall() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let wall_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                2.0,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_wall: vec![BoltImpactWall {
                bolt: bolt_entity,
                wall: wall_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after BoltImpactWall + Any gate");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 2 edge case: both Any and Wall gates fire --

    #[test]
    fn impact_occurred_any_and_wall_both_fire_for_bolt_impact_wall() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let wall_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                impact_occurred_speed_tree("chip_any", EntityKind::Any, 2.0),
                impact_occurred_speed_tree("chip_wall", EntityKind::Wall, 3.0),
            ]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_wall: vec![BoltImpactWall {
                bolt: bolt_entity,
                wall: wall_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack");
        assert_eq!(stack.len(), 2, "Both Any and Wall gates should fire");
    }

    // -- Behavior 3: ImpactOccurred(Any) fires for BoltImpactBreaker --

    #[test]
    fn impact_occurred_any_fires_for_bolt_impact_breaker() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_breaker: vec![BoltImpactBreaker {
                bolt:    bolt_entity,
                breaker: breaker_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after BoltImpactBreaker + Any gate");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 3 edge case: no bound effects entities — no panic --

    #[test]
    fn impact_occurred_any_no_bound_effects_entities_is_no_op() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let breaker_entity = app.world_mut().spawn_empty().id();

        app.insert_resource(TestImpactMessages {
            bolt_breaker: vec![BoltImpactBreaker {
                bolt:    bolt_entity,
                breaker: breaker_entity,
            }],
            ..default()
        });

        // No entities with BoundEffects — should not panic
        tick(&mut app);
    }

    // -- Behavior 4: ImpactOccurred(Any) fires for BreakerImpactCell --

    #[test]
    fn impact_occurred_any_fires_for_breaker_impact_cell() {
        let mut app = bridge_test_app();

        let breaker_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            breaker_cell: vec![BreakerImpactCell {
                breaker: breaker_entity,
                cell:    cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after BreakerImpactCell + Any gate");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 4 edge case: Breaker-specific also fires alongside Any --

    #[test]
    fn impact_occurred_breaker_specific_fires_alongside_any_for_breaker_impact_cell() {
        let mut app = bridge_test_app();

        let breaker_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![
                impact_occurred_speed_tree("chip_any", EntityKind::Any, 1.5),
                impact_occurred_speed_tree("chip_breaker", EntityKind::Breaker, 2.0),
            ]))
            .id();

        app.insert_resource(TestImpactMessages {
            breaker_cell: vec![BreakerImpactCell {
                breaker: breaker_entity,
                cell:    cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack");
        assert_eq!(stack.len(), 2, "Both Any and Breaker gates should fire");
    }

    // -- Behavior 5: ImpactOccurred(Any) fires once per collision, not per participant --

    #[test]
    fn impact_occurred_any_fires_once_per_collision_event() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_cell: vec![BoltImpactCell {
                bolt: bolt_entity,
                cell: cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after ImpactOccurred(Any)");
        assert_eq!(
            stack.len(),
            1,
            "EntityKind::Any should fire once per collision, not once per participant"
        );
    }

    // -- Behavior 5 edge case: two collisions in one frame fires twice --

    #[test]
    fn impact_occurred_any_fires_once_per_collision_two_events() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();
        let wall_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            bolt_cell: vec![BoltImpactCell {
                bolt: bolt_entity,
                cell: cell_entity,
            }],
            bolt_wall: vec![BoltImpactWall {
                bolt: bolt_entity,
                wall: wall_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after two collision events");
        assert_eq!(
            stack.len(),
            2,
            "Two collision events should fire ImpactOccurred(Any) twice"
        );
    }

    // -- Behavior 6: does not fire on entities without BoundEffects --

    #[test]
    fn impact_occurred_any_does_not_fire_on_entities_without_bound_effects() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        let entity_a = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        // entity_b has no BoundEffects
        let entity_b = app.world_mut().spawn_empty().id();

        app.insert_resource(TestImpactMessages {
            bolt_cell: vec![BoltImpactCell {
                bolt: bolt_entity,
                cell: cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack_a = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_a)
            .expect("entity_a (has BoundEffects) should have EffectStack");
        assert_eq!(stack_a.len(), 1);

        let stack_b = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_b);
        assert!(
            stack_b.is_none(),
            "entity_b (no BoundEffects) should not have EffectStack"
        );
    }

    // -- Behavior 6 edge case: empty BoundEffects still walked but no match --

    #[test]
    fn impact_occurred_any_empty_bound_effects_no_match() {
        let mut app = bridge_test_app();

        let bolt_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        let entity = app.world_mut().spawn(BoundEffects(vec![])).id();

        app.insert_resource(TestImpactMessages {
            bolt_cell: vec![BoltImpactCell {
                bolt: bolt_entity,
                cell: cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "empty BoundEffects should not produce EffectStack"
        );
    }

    // -- Behavior 7: Any dispatch does not false-match specific gates --

    #[test]
    fn impact_occurred_any_dispatch_does_not_false_match_specific_gates() {
        let mut app = bridge_test_app();

        let breaker_entity = app.world_mut().spawn_empty().id();
        let cell_entity = app.world_mut().spawn_empty().id();

        // entity_specific gates on Bolt — BreakerImpactCell has no Bolt
        let entity_specific = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_bolt",
                EntityKind::Bolt,
                1.5,
            )]))
            .id();

        // entity_any gates on Any — should always fire
        let entity_any = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_any",
                EntityKind::Any,
                2.0,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            breaker_cell: vec![BreakerImpactCell {
                breaker: breaker_entity,
                cell:    cell_entity,
            }],
            ..default()
        });

        tick(&mut app);

        // entity_any should fire
        let stack_any = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_any)
            .expect("entity_any should have EffectStack");
        assert_eq!(stack_any.len(), 1);

        // entity_specific should NOT fire — BreakerImpactCell dispatches
        // Breaker and Cell, not Bolt. ImpactOccurred(Any) should not match Bolt gate.
        let stack_specific = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity_specific);
        assert!(
            stack_specific.is_none(),
            "ImpactOccurred(Any) dispatch must not false-match ImpactOccurred(Bolt) gate"
        );
    }

    // -- Behavior 8: ImpactOccurred(Any) fires for BreakerImpactWall --

    #[test]
    fn impact_occurred_any_fires_for_breaker_impact_wall() {
        let mut app = bridge_test_app();

        let breaker_entity = app.world_mut().spawn_empty().id();
        let wall_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            breaker_wall: vec![BreakerImpactWall {
                breaker: breaker_entity,
                wall:    wall_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after BreakerImpactWall + Any gate");
        assert_eq!(stack.len(), 1);
    }

    // -- Behavior 9: ImpactOccurred(Any) fires for CellImpactWall --

    #[test]
    fn impact_occurred_any_fires_for_cell_impact_wall() {
        let mut app = bridge_test_app();

        let cell_entity = app.world_mut().spawn_empty().id();
        let wall_entity = app.world_mut().spawn_empty().id();

        let entity = app
            .world_mut()
            .spawn(BoundEffects(vec![impact_occurred_speed_tree(
                "chip_a",
                EntityKind::Any,
                1.5,
            )]))
            .id();

        app.insert_resource(TestImpactMessages {
            cell_wall: vec![CellImpactWall {
                cell: cell_entity,
                wall: wall_entity,
            }],
            ..default()
        });

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("entity should have EffectStack after CellImpactWall + Any gate");
        assert_eq!(stack.len(), 1);
    }
}
