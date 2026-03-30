//! Dispatches cell-defined effects to target entities when spawned.
//!
//! Reads each cell's `CellTypeDefinition.effects` (optional) and pushes
//! children to the appropriate target entity's `BoundEffects`.
//! Bare `Do` children are fired immediately via `commands.fire_effect()`.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{
        components::{Cell, CellEffectsDispatched, CellTypeAlias},
        resources::CellTypeRegistry,
    },
    effect::{EffectCommandsExt, EffectNode, RootEffect, Target},
    wall::components::Wall,
};

/// Query for cells that have not yet had their effects dispatched.
type CellDispatchQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static CellTypeAlias), (With<Cell>, Without<CellEffectsDispatched>)>;

/// Dispatches effects from cell type definitions to target entities.
///
/// For each cell entity without `CellEffectsDispatched`, looks up the cell's
/// definition in `CellTypeRegistry` and processes `RootEffect::On { target, then }`:
/// - `Do` children are fired immediately
/// - Non-`Do` children are pushed to the target entity's `BoundEffects`
///
/// Inserts `CellEffectsDispatched` marker after processing to prevent double-dispatch.
pub(crate) fn dispatch_cell_effects(
    mut commands: Commands,
    cell_query: CellDispatchQuery,
    registry: Option<Res<CellTypeRegistry>>,
    bolt_query: Query<Entity, With<Bolt>>,
    breaker_query: Query<Entity, With<Breaker>>,
    wall_query: Query<Entity, With<Wall>>,
    all_cells_query: Query<Entity, With<Cell>>,
) {
    let Some(registry) = registry else {
        return;
    };
    for (entity, alias) in &cell_query {
        let Some(def) = registry.get(alias.0) else {
            continue;
        };

        let effects = match &def.effects {
            None => continue,
            Some(effects) if effects.is_empty() => continue,
            Some(effects) => effects,
        };

        for root_effect in effects {
            let RootEffect::On { target, then } = root_effect;

            let mut non_do_children: Vec<(String, EffectNode)> = Vec::new();
            let mut do_children = Vec::new();
            for child in then {
                match child {
                    EffectNode::Do(effect) => do_children.push(effect.clone()),
                    // Cell-sourced effects use empty source_chip — they come from
                    // the cell type definition, not from an evolution chip.
                    other => non_do_children.push((String::new(), other.clone())),
                }
            }

            // Resolve target entities
            let target_entities: Vec<Entity> = match target {
                Target::Cell => vec![entity],
                Target::Bolt | Target::AllBolts => bolt_query.iter().collect(),
                Target::Breaker => breaker_query
                    .single()
                    .map_or_else(|_| Vec::new(), |breaker| vec![breaker]),
                Target::AllCells => all_cells_query.iter().collect(),
                Target::Wall | Target::AllWalls => wall_query.iter().collect(),
            };

            for target_entity in &target_entities {
                // Fire Do children immediately. Empty source_chip because
                // cell-sourced effects are not attributed to any evolution chip.
                for effect in &do_children {
                    commands.fire_effect(*target_entity, effect.clone(), String::new());
                }

                // Push non-Do children to BoundEffects
                if !non_do_children.is_empty() {
                    commands.push_bound_effects(*target_entity, non_do_children.clone());
                }
            }
        }

        commands.entity(entity).insert(CellEffectsDispatched);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::dispatch_cell_effects;
    use crate::{
        bolt::components::Bolt,
        breaker::components::Breaker,
        cells::{
            components::{Cell, CellEffectsDispatched, CellTypeAlias},
            definition::{CellBehavior, CellTypeDefinition},
            resources::CellTypeRegistry,
        },
        effect::{
            BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
            Trigger,
        },
        wall::components::Wall,
    };

    // ── Test helpers ────────────────────────────────────────────────

    /// Builds a minimal cell type definition with the given id, alias, hp, and effects.
    fn make_cell_def(
        id: &str,
        alias: char,
        hp: f32,
        effects: Option<Vec<RootEffect>>,
    ) -> CellTypeDefinition {
        CellTypeDefinition {
            id: id.to_owned(),
            alias,
            hp,
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 1.0,
            damage_green_min: 0.3,
            damage_blue_range: 0.5,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
            effects,
        }
    }

    /// Creates a test app with the dispatch system and the given registry.
    fn test_app(registry: CellTypeRegistry) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(registry);
        app.add_systems(Update, dispatch_cell_effects);
        app
    }

    // ── Behavior 1: Cell with effects gets children pushed to BoundEffects (Target::Cell) ──

    #[test]
    fn cell_with_target_cell_effect_gets_bound_effects_populated() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        app.update();

        // Cell should have BoundEffects with exactly 1 entry
        let bound = app
            .world()
            .get::<BoundEffects>(cell_entity)
            .expect("cell should have BoundEffects after dispatch");
        assert_eq!(
            bound.0.len(),
            1,
            "cell should have exactly 1 BoundEffects entry, got {}",
            bound.0.len()
        );
        let (chip_name, node) = &bound.0[0];
        assert_eq!(
            chip_name, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node,
                EffectNode::When {
                    trigger: Trigger::Died,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::Explode { range, damage_mult }) if (range - 48.0).abs() < f32::EPSILON && (damage_mult - 1.0).abs() < f32::EPSILON)
            ),
            "expected When {{ Died, [Do(Explode {{ range: 48.0, damage_mult: 1.0 }})] }}, got {node:?}"
        );

        // Cell should have StagedEffects (default-inserted)
        assert!(
            app.world().get::<StagedEffects>(cell_entity).is_some(),
            "cell should have StagedEffects after dispatch"
        );

        // Cell should have CellEffectsDispatched marker
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched marker after dispatch"
        );
    }

    // ── Behavior 1 edge case: Cell with existing BoundEffects but no marker ──

    #[test]
    fn cell_with_existing_bound_effects_but_no_marker_still_gets_dispatched() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias('E'),
                BoundEffects(vec![(
                    "existing_chip".to_owned(),
                    EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                    },
                )]),
            ))
            .id();
        app.update();

        let bound = app
            .world()
            .get::<BoundEffects>(cell_entity)
            .expect("cell should have BoundEffects after dispatch");
        assert_eq!(
            bound.0.len(),
            2,
            "cell should have 2 BoundEffects entries (1 existing + 1 dispatched), got {}",
            bound.0.len()
        );
        // Existing entry at index 0 is preserved
        assert_eq!(
            bound.0[0].0, "existing_chip",
            "existing entry at index 0 should be preserved"
        );
        // Dispatched entry at index 1
        assert_eq!(
            bound.0[1].0, "",
            "dispatched entry at index 1 should have empty chip_name"
        );

        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched marker"
        );
    }

    // ── Behavior 2: Cell with no effects is unchanged ──

    #[test]
    fn cell_with_no_effects_is_unchanged() {
        let mut registry = CellTypeRegistry::default();
        registry.insert('S', make_cell_def("standard", 'S', 10.0, None));
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_s = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
        let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        app.update();

        // Positive: 'E' cell with effects SHOULD get BoundEffects
        let bound_e = app
            .world()
            .get::<BoundEffects>(cell_e)
            .expect("cell 'E' with effects should have BoundEffects after dispatch");
        assert_eq!(
            bound_e.0.len(),
            1,
            "cell 'E' should have exactly 1 BoundEffects entry"
        );

        // Negative: 'S' cell with no effects should NOT get BoundEffects
        assert!(
            app.world().get::<BoundEffects>(cell_s).is_none(),
            "cell 'S' with no effects should NOT have BoundEffects"
        );
        assert!(
            app.world().get::<StagedEffects>(cell_s).is_none(),
            "cell 'S' with no effects should NOT have StagedEffects"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_s).is_none(),
            "cell 'S' with no effects should NOT have CellEffectsDispatched"
        );
    }

    // ── Behavior 2 edge case: effects is Some(vec![]) (empty vec) ──

    #[test]
    fn cell_with_empty_effects_vec_is_unchanged() {
        let mut registry = CellTypeRegistry::default();
        registry.insert('S', make_cell_def("standard", 'S', 10.0, Some(vec![])));
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_s = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
        let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        app.update();

        // Positive: 'E' cell with non-empty effects SHOULD get BoundEffects
        let bound_e = app
            .world()
            .get::<BoundEffects>(cell_e)
            .expect("cell 'E' with non-empty effects should have BoundEffects after dispatch");
        assert_eq!(
            bound_e.0.len(),
            1,
            "cell 'E' should have exactly 1 BoundEffects entry"
        );

        // Negative: 'S' cell with empty effects vec should NOT get BoundEffects
        assert!(
            app.world().get::<BoundEffects>(cell_s).is_none(),
            "cell 'S' with empty effects vec should NOT have BoundEffects"
        );
        assert!(
            app.world().get::<StagedEffects>(cell_s).is_none(),
            "cell 'S' with empty effects vec should NOT have StagedEffects"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_s).is_none(),
            "cell 'S' with empty effects vec should NOT have CellEffectsDispatched"
        );
    }

    // ── Behavior 3: Cell with alias not found in registry is skipped ──

    #[test]
    fn cell_with_unknown_alias_is_skipped_no_panic() {
        let mut registry = CellTypeRegistry::default();
        registry.insert('S', make_cell_def("standard", 'S', 10.0, None));
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_x = app.world_mut().spawn((Cell, CellTypeAlias('X'))).id();
        let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        app.update();

        // Positive: 'E' cell with known alias and effects SHOULD get BoundEffects
        let bound_e = app
            .world()
            .get::<BoundEffects>(cell_e)
            .expect("cell 'E' with known alias should have BoundEffects after dispatch");
        assert_eq!(
            bound_e.0.len(),
            1,
            "cell 'E' should have exactly 1 BoundEffects entry"
        );

        // Negative: 'X' cell with unknown alias should NOT get BoundEffects
        assert!(
            app.world().get::<BoundEffects>(cell_x).is_none(),
            "cell with unknown alias should NOT have BoundEffects"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_x).is_none(),
            "cell with unknown alias should NOT have CellEffectsDispatched"
        );
    }

    // ── Behavior 3 edge case: Known alias dispatched, missing alias skipped ──

    #[test]
    fn cell_with_alias_not_in_registry_skipped_while_known_alias_dispatched() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        let cell_x = app.world_mut().spawn((Cell, CellTypeAlias('X'))).id();
        app.update();

        // Positive: 'E' cell with known alias SHOULD get BoundEffects
        let bound_e = app
            .world()
            .get::<BoundEffects>(cell_e)
            .expect("cell 'E' with known alias should have BoundEffects after dispatch");
        assert_eq!(
            bound_e.0.len(),
            1,
            "cell 'E' should have exactly 1 BoundEffects entry"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_e).is_some(),
            "cell 'E' should have CellEffectsDispatched marker"
        );

        // Negative: 'X' cell with alias not in registry should NOT get BoundEffects
        assert!(
            app.world().get::<BoundEffects>(cell_x).is_none(),
            "cell 'X' with alias not in registry should NOT have BoundEffects"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_x).is_none(),
            "cell 'X' with alias not in registry should NOT have CellEffectsDispatched"
        );
    }

    // ── Behavior 4: Multiple cells each get their own effects dispatched independently ──

    #[test]
    fn multiple_cells_dispatched_independently() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );
        registry.insert('S', make_cell_def("standard", 'S', 10.0, None));

        let mut app = test_app(registry);
        let cell_a = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        let cell_b = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        let cell_c = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
        app.update();

        // Cell A: has BoundEffects with 1 entry
        let bound_a = app
            .world()
            .get::<BoundEffects>(cell_a)
            .expect("Cell A should have BoundEffects");
        assert_eq!(
            bound_a.0.len(),
            1,
            "Cell A should have 1 BoundEffects entry"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_a).is_some(),
            "Cell A should have CellEffectsDispatched"
        );

        // Cell B: has BoundEffects with 1 entry
        let bound_b = app
            .world()
            .get::<BoundEffects>(cell_b)
            .expect("Cell B should have BoundEffects");
        assert_eq!(
            bound_b.0.len(),
            1,
            "Cell B should have 1 BoundEffects entry"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_b).is_some(),
            "Cell B should have CellEffectsDispatched"
        );

        // Cell C: no effects
        assert!(
            app.world().get::<BoundEffects>(cell_c).is_none(),
            "Cell C should NOT have BoundEffects"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_c).is_none(),
            "Cell C should NOT have CellEffectsDispatched"
        );
    }

    // ── Behavior 5: Cell with Target::Bolt effect dispatches to bolt entity ──

    #[test]
    fn cell_with_target_bolt_dispatches_to_bolt_entity() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'B',
            make_cell_def(
                "bolt_boost_cell",
                'B',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
        let bolt_entity = app.world_mut().spawn(Bolt).id();
        app.update();

        // Cell should have CellEffectsDispatched marker
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched marker"
        );

        // Bolt should have BoundEffects with 1 entry
        let bolt_bound = app
            .world()
            .get::<BoundEffects>(bolt_entity)
            .expect("bolt should have BoundEffects after dispatch");
        assert_eq!(
            bolt_bound.0.len(),
            1,
            "bolt should have 1 BoundEffects entry"
        );
        let (chip_name, node) = &bolt_bound.0[0];
        assert_eq!(chip_name, "", "chip_name should be empty string");
        assert!(
            matches!(
                node,
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.2).abs() < f32::EPSILON)
            ),
            "expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.2 }})] }}, got {node:?}"
        );

        // Bolt should have StagedEffects
        assert!(
            app.world().get::<StagedEffects>(bolt_entity).is_some(),
            "bolt should have StagedEffects after dispatch"
        );

        // Cell itself should NOT get BoundEffects from bolt-targeted effect
        assert!(
            app.world().get::<BoundEffects>(cell_entity).is_none(),
            "cell should NOT have BoundEffects from bolt-targeted effect"
        );
    }

    // ── Behavior 5 edge case: No bolt present ──

    #[test]
    fn cell_with_target_bolt_no_bolt_present_no_panic() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'B',
            make_cell_def(
                "bolt_boost_cell",
                'B',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
        // No bolt entity spawned
        app.update();

        // Cell should still get CellEffectsDispatched marker
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched marker even with no bolt present"
        );

        // Cell should NOT get BoundEffects (effect targets bolt, not cell)
        assert!(
            app.world().get::<BoundEffects>(cell_entity).is_none(),
            "cell should NOT have BoundEffects when bolt-targeted and no bolt present"
        );
    }

    // ── Behavior 6: Cell with Target::Breaker dispatches to breaker entity ──

    #[test]
    fn cell_with_target_breaker_dispatches_to_breaker_entity() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'R',
            make_cell_def(
                "breaker_buff_cell",
                'R',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bump,
                        then: vec![EffectNode::Do(EffectKind::QuickStop { multiplier: 2.0 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('R'))).id();
        let breaker_entity = app
            .world_mut()
            .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
            .id();
        app.update();

        // Cell should have CellEffectsDispatched
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched marker"
        );

        // Breaker should have 1 entry in BoundEffects
        let breaker_bound = app
            .world()
            .get::<BoundEffects>(breaker_entity)
            .expect("breaker should have BoundEffects");
        assert_eq!(
            breaker_bound.0.len(),
            1,
            "breaker should have 1 BoundEffects entry"
        );
        let (chip_name, node) = &breaker_bound.0[0];
        assert_eq!(chip_name, "", "chip_name should be empty string");
        assert!(
            matches!(
                node,
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::QuickStop { multiplier }) if (multiplier - 2.0).abs() < f32::EPSILON)
            ),
            "expected When {{ Bump, [Do(QuickStop {{ multiplier: 2.0 }})] }}, got {node:?}"
        );

        // Cell itself should NOT get these effects
        assert!(
            app.world().get::<BoundEffects>(cell_entity).is_none(),
            "cell should NOT have BoundEffects from breaker-targeted effect"
        );
    }

    // ── Behavior 6 edge case: No breaker present ──

    #[test]
    fn cell_with_target_breaker_no_breaker_present_no_panic() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'R',
            make_cell_def(
                "breaker_buff_cell",
                'R',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bump,
                        then: vec![EffectNode::Do(EffectKind::QuickStop { multiplier: 2.0 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('R'))).id();
        // No breaker entity spawned
        app.update();

        // Cell should still get CellEffectsDispatched marker
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched even with no breaker present"
        );
    }

    // ── Target::AllBolts dispatches to all bolt entities ──

    #[test]
    fn cell_with_target_all_bolts_dispatches_to_all_bolt_entities() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'B',
            make_cell_def(
                "all_bolts_cell",
                'B',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::AllBolts,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
        let bolt_a = app.world_mut().spawn(Bolt).id();
        let bolt_b = app.world_mut().spawn(Bolt).id();
        app.update();

        // Both bolts should have BoundEffects with 1 entry each
        let bound_a = app
            .world()
            .get::<BoundEffects>(bolt_a)
            .expect("bolt A should have BoundEffects from AllBolts dispatch");
        assert_eq!(
            bound_a.0.len(),
            1,
            "bolt A should have exactly 1 BoundEffects entry"
        );
        let (chip_name_a, node_a) = &bound_a.0[0];
        assert_eq!(
            chip_name_a, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node_a,
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.5).abs() < f32::EPSILON)
            ),
            "bolt A expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.5 }})] }}, got {node_a:?}"
        );

        let bound_b = app
            .world()
            .get::<BoundEffects>(bolt_b)
            .expect("bolt B should have BoundEffects from AllBolts dispatch");
        assert_eq!(
            bound_b.0.len(),
            1,
            "bolt B should have exactly 1 BoundEffects entry"
        );
        let (chip_name_b, node_b) = &bound_b.0[0];
        assert_eq!(
            chip_name_b, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node_b,
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.5).abs() < f32::EPSILON)
            ),
            "bolt B expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.5 }})] }}, got {node_b:?}"
        );

        // Cell should have CellEffectsDispatched marker
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched marker"
        );

        // Cell itself should NOT get BoundEffects (effect targets bolts, not cell)
        assert!(
            app.world().get::<BoundEffects>(cell_entity).is_none(),
            "cell should NOT have BoundEffects from AllBolts-targeted effect"
        );
    }

    // ── Behavior 7: Cell with Target::AllCells dispatches to ALL cell entities ──

    #[test]
    fn cell_with_target_all_cells_dispatches_to_all_cells() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'A',
            make_cell_def(
                "all_cells_buff",
                'A',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::AllCells,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 32.0,
                            damage_mult: 0.5,
                        })],
                    }],
                }]),
            ),
        );
        registry.insert('S', make_cell_def("standard", 'S', 10.0, None));

        let mut app = test_app(registry);
        let cell_a = app.world_mut().spawn((Cell, CellTypeAlias('A'))).id();
        let cell_b = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
        app.update();

        // Cell A (source) has BoundEffects with 1 entry (AllCells includes self)
        let bound_a = app
            .world()
            .get::<BoundEffects>(cell_a)
            .expect("Cell A should have BoundEffects (AllCells includes source)");
        assert_eq!(
            bound_a.0.len(),
            1,
            "Cell A should have 1 BoundEffects entry from AllCells"
        );

        // Cell B (other cell) also has BoundEffects with 1 entry
        let bound_b = app
            .world()
            .get::<BoundEffects>(cell_b)
            .expect("Cell B should have BoundEffects from AllCells dispatch");
        assert_eq!(
            bound_b.0.len(),
            1,
            "Cell B should have 1 BoundEffects entry from AllCells"
        );

        // Cell A has CellEffectsDispatched marker (it was the source)
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_a).is_some(),
            "Cell A (source) should have CellEffectsDispatched"
        );

        // Cell B does NOT have CellEffectsDispatched (it has no effects of its own)
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_b).is_none(),
            "Cell B should NOT have CellEffectsDispatched (marker is for source cell only)"
        );
    }

    // ── Behavior 7 edge case: Only 1 cell entity (source gets its own AllCells effect) ──

    #[test]
    fn single_cell_with_all_cells_targets_itself() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'A',
            make_cell_def(
                "all_cells_buff",
                'A',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::AllCells,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 32.0,
                            damage_mult: 0.5,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_a = app.world_mut().spawn((Cell, CellTypeAlias('A'))).id();
        app.update();

        let bound = app
            .world()
            .get::<BoundEffects>(cell_a)
            .expect("single cell should receive its own AllCells effect");
        assert_eq!(
            bound.0.len(),
            1,
            "single cell should have 1 BoundEffects entry from AllCells"
        );
    }

    // ── Behavior 8: Do children are fired immediately, not stored in BoundEffects ──

    #[test]
    fn do_children_are_not_stored_in_bound_effects() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'D',
            make_cell_def(
                "immediate_effect_cell",
                'D',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![
                        EffectNode::Do(EffectKind::DamageBoost(1.5)),
                        EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::Explode {
                                range: 48.0,
                                damage_mult: 1.0,
                            })],
                        },
                    ],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('D'))).id();
        app.update();

        // BoundEffects should have exactly 1 entry (the When node), NOT the Do node
        let bound = app
            .world()
            .get::<BoundEffects>(cell_entity)
            .expect("cell should have BoundEffects");
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should have 1 entry (When), not the Do child; got {}",
            bound.0.len()
        );
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Died,
                    ..
                }
            ),
            "the single BoundEffects entry should be the When {{ Died }} node, got {:?}",
            bound.0[0].1
        );
    }

    // ── Behavior 8 edge case: All children are Do nodes ──

    #[test]
    fn all_do_children_results_in_no_bound_effects_entries() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'D',
            make_cell_def(
                "all_do_cell",
                'D',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![
                        EffectNode::Do(EffectKind::DamageBoost(1.5)),
                        EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 }),
                    ],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('D'))).id();
        app.update();

        // If BoundEffects was inserted, it should be empty (all children were Do nodes)
        // Or BoundEffects might not be inserted at all -- both are acceptable
        if let Some(bound) = app.world().get::<BoundEffects>(cell_entity) {
            assert_eq!(
                bound.0.len(),
                0,
                "BoundEffects should be empty when all children are Do nodes, got {}",
                bound.0.len()
            );
        }

        // Cell should still have marker since it had effects
        assert!(
            app.world()
                .get::<CellEffectsDispatched>(cell_entity)
                .is_some(),
            "cell should have CellEffectsDispatched even when all children are Do nodes"
        );
    }

    // ── Behavior 9: Cell with multiple RootEffects gets all dispatched ──

    #[test]
    fn cell_with_multiple_root_effects_gets_all_dispatched() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'M',
            make_cell_def(
                "multi_effect_cell",
                'M',
                20.0,
                Some(vec![
                    RootEffect::On {
                        target: Target::Cell,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::Explode {
                                range: 48.0,
                                damage_mult: 1.0,
                            })],
                        }],
                    },
                    RootEffect::On {
                        target: Target::Cell,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Impacted(ImpactTarget::Bolt),
                            then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
                        }],
                    },
                ]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('M'))).id();
        app.update();

        let bound = app
            .world()
            .get::<BoundEffects>(cell_entity)
            .expect("cell should have BoundEffects");
        assert_eq!(
            bound.0.len(),
            2,
            "cell should have 2 BoundEffects entries, got {}",
            bound.0.len()
        );

        // Both entries should have empty chip_name
        assert_eq!(bound.0[0].0, "", "first entry chip_name should be empty");
        assert_eq!(bound.0[1].0, "", "second entry chip_name should be empty");

        // First: Died->Explode
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Died,
                    ..
                }
            ),
            "first entry should be When {{ Died }}, got {:?}",
            bound.0[0].1
        );

        // Second: Impacted(Bolt)->DamageBoost
        assert!(
            matches!(
                &bound.0[1].1,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    ..
                }
            ),
            "second entry should be When {{ Impacted(Bolt) }}, got {:?}",
            bound.0[1].1
        );
    }

    // ── Behavior 9 edge case: Mix of Target::Cell and Target::Bolt ──

    #[test]
    fn cell_with_mixed_targets_dispatches_to_correct_entities() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'M',
            make_cell_def(
                "mixed_target_cell",
                'M',
                10.0,
                Some(vec![
                    RootEffect::On {
                        target: Target::Cell,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Died,
                            then: vec![EffectNode::Do(EffectKind::Explode {
                                range: 48.0,
                                damage_mult: 1.0,
                            })],
                        }],
                    },
                    RootEffect::On {
                        target: Target::Bolt,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Bumped,
                            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                        }],
                    },
                ]),
            ),
        );

        let mut app = test_app(registry);
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('M'))).id();
        let bolt_entity = app.world_mut().spawn(Bolt).id();
        app.update();

        // Cell gets the Cell-targeted effect
        let cell_bound = app
            .world()
            .get::<BoundEffects>(cell_entity)
            .expect("cell should have BoundEffects for Cell-targeted effect");
        assert_eq!(
            cell_bound.0.len(),
            1,
            "cell should have 1 BoundEffects entry (Cell-targeted only)"
        );

        // Bolt gets the Bolt-targeted effect
        let bolt_bound = app
            .world()
            .get::<BoundEffects>(bolt_entity)
            .expect("bolt should have BoundEffects for Bolt-targeted effect");
        assert_eq!(
            bolt_bound.0.len(),
            1,
            "bolt should have 1 BoundEffects entry (Bolt-targeted only)"
        );
    }

    // ── Behavior 10: CellEffectsDispatched prevents double-dispatch ──

    #[test]
    fn cell_effects_dispatched_marker_prevents_double_dispatch() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        // Spawn cell that already has the marker and 1 existing entry
        let cell_entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias('E'),
                CellEffectsDispatched,
                BoundEffects(vec![(
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    },
                )]),
            ))
            .id();
        app.update();

        let bound = app
            .world()
            .get::<BoundEffects>(cell_entity)
            .expect("cell should still have BoundEffects");
        assert_eq!(
            bound.0.len(),
            1,
            "BoundEffects should still have 1 entry (no double-dispatch), got {}",
            bound.0.len()
        );
    }

    // ── Behavior 10 edge case: Marker on A (skipped), no marker on B (dispatched) ──

    #[test]
    fn marker_on_one_cell_skips_it_while_other_is_dispatched() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        // Cell A: already dispatched (has marker)
        let cell_a = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias('E'),
                CellEffectsDispatched,
                BoundEffects(vec![(
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    },
                )]),
            ))
            .id();
        // Cell B: not dispatched yet
        let cell_b = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        app.update();

        // Cell A unchanged (still 1 entry)
        let bound_a = app
            .world()
            .get::<BoundEffects>(cell_a)
            .expect("Cell A should have BoundEffects");
        assert_eq!(
            bound_a.0.len(),
            1,
            "Cell A should be unchanged (skipped by marker)"
        );

        // Cell B dispatched (now has 1 entry)
        let bound_b = app
            .world()
            .get::<BoundEffects>(cell_b)
            .expect("Cell B should have BoundEffects after dispatch");
        assert_eq!(
            bound_b.0.len(),
            1,
            "Cell B should have 1 BoundEffects entry"
        );
        assert!(
            app.world().get::<CellEffectsDispatched>(cell_b).is_some(),
            "Cell B should have CellEffectsDispatched marker"
        );
    }

    // ── Behavior 11: BoundEffects and StagedEffects inserted if absent on self-targeted cell ──

    #[test]
    fn bound_effects_and_staged_effects_inserted_on_cell_if_absent() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        // Spawn cell with NO BoundEffects and NO StagedEffects
        let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
        app.update();

        assert!(
            app.world().get::<BoundEffects>(cell_entity).is_some(),
            "BoundEffects should be inserted on cell when absent"
        );
        assert!(
            app.world().get::<StagedEffects>(cell_entity).is_some(),
            "StagedEffects should be inserted on cell when absent"
        );
    }

    // ── Behavior 11 edge case: Cell has BoundEffects but no StagedEffects ──

    #[test]
    fn staged_effects_inserted_when_bound_effects_already_exists() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'E',
            make_cell_def(
                "effect_cell",
                'E',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        // Spawn cell WITH BoundEffects but WITHOUT StagedEffects
        let cell_entity = app
            .world_mut()
            .spawn((Cell, CellTypeAlias('E'), BoundEffects::default()))
            .id();
        app.update();

        assert!(
            app.world().get::<StagedEffects>(cell_entity).is_some(),
            "StagedEffects should be inserted even when BoundEffects already existed"
        );
    }

    // ── Behavior 12: Non-Cell target entities get BoundEffects/StagedEffects pre-inserted ──

    #[test]
    fn bolt_gets_bound_effects_and_staged_effects_pre_inserted() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'B',
            make_cell_def(
                "bolt_boost_cell",
                'B',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        app.world_mut().spawn((Cell, CellTypeAlias('B')));
        // Bolt spawned with NO BoundEffects, NO StagedEffects
        let bolt_entity = app.world_mut().spawn(Bolt).id();
        app.update();

        assert!(
            app.world().get::<BoundEffects>(bolt_entity).is_some(),
            "bolt should have BoundEffects pre-inserted by dispatch"
        );
        assert!(
            app.world().get::<StagedEffects>(bolt_entity).is_some(),
            "bolt should have StagedEffects pre-inserted by dispatch"
        );
    }

    // ── Behavior 12 edge case: Bolt has BoundEffects but not StagedEffects ──

    #[test]
    fn bolt_with_bound_effects_but_no_staged_effects_gets_staged_effects_inserted() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'B',
            make_cell_def(
                "bolt_boost_cell",
                'B',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        app.world_mut().spawn((Cell, CellTypeAlias('B')));
        // Bolt has BoundEffects but NOT StagedEffects
        let bolt_entity = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
        app.update();

        assert!(
            app.world().get::<StagedEffects>(bolt_entity).is_some(),
            "StagedEffects should be inserted on bolt even when BoundEffects already existed"
        );
        // Existing BoundEffects should be preserved (with dispatched entry appended)
        let bound = app
            .world()
            .get::<BoundEffects>(bolt_entity)
            .expect("bolt should still have BoundEffects");
        assert_eq!(
            bound.0.len(),
            1,
            "bolt BoundEffects should have 1 dispatched entry"
        );
    }

    // ── Regression: Target::Bolt dispatches to ALL bolt entities, not just first ──

    #[test]
    fn cell_with_target_bolt_dispatches_to_all_bolt_entities() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'B',
            make_cell_def(
                "bolt_boost_cell",
                'B',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.4 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let _cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
        let bolt_a = app.world_mut().spawn(Bolt).id();
        let bolt_b = app.world_mut().spawn(Bolt).id();
        app.update();

        // BOTH bolts should have BoundEffects with 1 entry each
        let bound_a = app
            .world()
            .get::<BoundEffects>(bolt_a)
            .expect("bolt A should have BoundEffects from Target::Bolt dispatch");
        assert_eq!(
            bound_a.0.len(),
            1,
            "bolt A should have exactly 1 BoundEffects entry"
        );
        let (chip_name_a, node_a) = &bound_a.0[0];
        assert_eq!(
            chip_name_a, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node_a,
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.4).abs() < f32::EPSILON)
            ),
            "bolt A expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.4 }})] }}, got {node_a:?}"
        );

        let bound_b = app
            .world()
            .get::<BoundEffects>(bolt_b)
            .expect("bolt B should have BoundEffects from Target::Bolt dispatch");
        assert_eq!(
            bound_b.0.len(),
            1,
            "bolt B should have exactly 1 BoundEffects entry"
        );
        let (chip_name_b, node_b) = &bound_b.0[0];
        assert_eq!(
            chip_name_b, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node_b,
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.4).abs() < f32::EPSILON)
            ),
            "bolt B expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.4 }})] }}, got {node_b:?}"
        );
    }

    // ── Regression: Target::Wall dispatches to ALL wall entities, not just first ──

    #[test]
    fn cell_with_target_wall_dispatches_to_all_wall_entities() {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'W',
            make_cell_def(
                "wall_buff_cell",
                'W',
                10.0,
                Some(vec![RootEffect::On {
                    target: Target::Wall,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 0.8 })],
                    }],
                }]),
            ),
        );

        let mut app = test_app(registry);
        let _cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('W'))).id();
        let wall_a = app.world_mut().spawn(Wall).id();
        let wall_b = app.world_mut().spawn(Wall).id();
        app.update();

        // BOTH walls should have BoundEffects with 1 entry each
        let bound_a = app
            .world()
            .get::<BoundEffects>(wall_a)
            .expect("wall A should have BoundEffects from Target::Wall dispatch");
        assert_eq!(
            bound_a.0.len(),
            1,
            "wall A should have exactly 1 BoundEffects entry"
        );
        let (chip_name_a, node_a) = &bound_a.0[0];
        assert_eq!(
            chip_name_a, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node_a,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 0.8).abs() < f32::EPSILON)
            ),
            "wall A expected When {{ Impacted(Bolt), [Do(SpeedBoost {{ multiplier: 0.8 }})] }}, got {node_a:?}"
        );

        let bound_b = app
            .world()
            .get::<BoundEffects>(wall_b)
            .expect("wall B should have BoundEffects from Target::Wall dispatch");
        assert_eq!(
            bound_b.0.len(),
            1,
            "wall B should have exactly 1 BoundEffects entry"
        );
        let (chip_name_b, node_b) = &bound_b.0[0];
        assert_eq!(
            chip_name_b, "",
            "chip_name should be empty string for cell-defined effects"
        );
        assert!(
            matches!(
                node_b,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then,
                } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 0.8).abs() < f32::EPSILON)
            ),
            "wall B expected When {{ Impacted(Bolt), [Do(SpeedBoost {{ multiplier: 0.8 }})] }}, got {node_b:?}"
        );
    }
}
