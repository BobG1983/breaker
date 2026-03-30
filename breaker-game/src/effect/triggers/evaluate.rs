use bevy::prelude::*;

use crate::effect::{
    commands::{EffectCommandsExt, ResolveOnCommand},
    core::*,
};

/// Command that removes matching chains from an entity's `BoundEffects`.
pub(crate) struct RemoveChainsCommand {
    pub(crate) entity: Entity,
    pub(crate) chains: Vec<EffectNode>,
}

impl Command for RemoveChainsCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
            bound.0.retain(|(_, node)| !self.chains.contains(node));
        }
    }
}

/// Walk `BoundEffects` for a trigger. Entries are NEVER consumed.
pub(crate) fn evaluate_bound_effects(
    trigger: &Trigger,
    entity: Entity,
    bound: &BoundEffects,
    staged: &mut StagedEffects,
    commands: &mut Commands,
) {
    for (chip_name, node) in &bound.0 {
        walk_bound_node(trigger, entity, chip_name, node, staged, commands);
    }
}

/// Walk `StagedEffects` for a trigger. Matching entries ARE consumed.
pub(crate) fn evaluate_staged_effects(
    trigger: &Trigger,
    entity: Entity,
    staged: &mut StagedEffects,
    commands: &mut Commands,
) {
    let mut additions = Vec::new();
    staged.0.retain(|(chip_name, node)| {
        !walk_staged_node(trigger, entity, chip_name, node, &mut additions, commands)
    });
    staged.0.extend(additions);
}

fn walk_bound_node(
    trigger: &Trigger,
    entity: Entity,
    chip_name: &str,
    node: &EffectNode,
    staged: &mut StagedEffects,
    commands: &mut Commands,
) {
    if let EffectNode::When { trigger: t, then } = node
        && t == trigger
    {
        for child in then {
            match child {
                EffectNode::Do(effect) => {
                    commands.fire_effect(entity, effect.clone(), chip_name.to_string());
                }
                other => {
                    staged.0.push((chip_name.to_string(), other.clone()));
                }
            }
        }
    }
}

/// Returns true if the node was consumed (matched).
fn walk_staged_node(
    trigger: &Trigger,
    entity: Entity,
    chip_name: &str,
    node: &EffectNode,
    additions: &mut Vec<(String, EffectNode)>,
    commands: &mut Commands,
) -> bool {
    match node {
        EffectNode::When { trigger: t, then } if t == trigger => {
            for child in then {
                match child {
                    EffectNode::Do(effect) => {
                        commands.fire_effect(entity, effect.clone(), chip_name.to_string());
                    }
                    EffectNode::Reverse { effects, chains } => {
                        for effect in effects {
                            commands.reverse_effect(entity, effect.clone(), chip_name.to_string());
                        }
                        if !chains.is_empty() {
                            commands.queue(RemoveChainsCommand {
                                entity,
                                chains: chains.clone(),
                            });
                        }
                    }
                    other => {
                        additions.push((chip_name.to_string(), other.clone()));
                    }
                }
            }
            true // consumed
        }
        EffectNode::Once(children) => {
            let mut any_matched = false;
            for child in children {
                match child {
                    EffectNode::When { trigger: t, then } if t == trigger => {
                        any_matched = true;
                        for inner in then {
                            match inner {
                                EffectNode::Do(effect) => {
                                    commands.fire_effect(
                                        entity,
                                        effect.clone(),
                                        chip_name.to_string(),
                                    );
                                }
                                other => {
                                    additions.push((chip_name.to_string(), other.clone()));
                                }
                            }
                        }
                    }
                    EffectNode::Do(effect) => {
                        // Bare Do always matches — fire immediately
                        any_matched = true;
                        commands.fire_effect(entity, effect.clone(), chip_name.to_string());
                    }
                    _ => {}
                }
            }
            any_matched
        }
        EffectNode::On {
            target,
            permanent,
            then: on_children,
        } => {
            commands.queue(ResolveOnCommand {
                target: *target,
                chip_name: chip_name.to_string(),
                children: on_children.clone(),
                permanent: *permanent,
            });
            true // consumed — ResolveOnCommand resolves target asynchronously
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;

    /// Helper: build a `When(trigger, [Do(effect)])` node.
    fn when_do(trigger: Trigger, effect: EffectKind) -> EffectNode {
        EffectNode::When {
            trigger,
            then: vec![EffectNode::Do(effect)],
        }
    }

    /// Helper: build a `When(trigger, [child])` node with an arbitrary child.
    fn when_child(trigger: Trigger, child: EffectNode) -> EffectNode {
        EffectNode::When {
            trigger,
            then: vec![child],
        }
    }

    // -----------------------------------------------------------------------
    // Test systems: wrap evaluate_* so we can run them inside an App.
    // Each system evaluates for Trigger::Bump on all matching entities.
    // -----------------------------------------------------------------------

    fn sys_evaluate_bound_for_bump(
        mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::Bump;
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(&trigger, entity, bound, &mut staged, &mut commands);
        }
    }

    fn sys_evaluate_staged_for_bump(
        mut query: Query<(Entity, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::Bump;
        for (entity, mut staged) in &mut query {
            evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
        }
    }

    fn sys_evaluate_bound_for_bump_whiff(
        mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::BumpWhiff;
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(&trigger, entity, bound, &mut staged, &mut commands);
        }
    }

    fn sys_evaluate_staged_for_bump_whiff(
        mut query: Query<(Entity, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::BumpWhiff;
        for (entity, mut staged) in &mut query {
            evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
        }
    }

    fn sys_evaluate_staged_for_death(
        mut query: Query<(Entity, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::Death;
        for (entity, mut staged) in &mut query {
            evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
        }
    }

    /// Resource to snapshot component state BEFORE commands are applied.
    /// A system reads BoundEffects/StagedEffects and stores a copy here.
    #[derive(Resource, Default)]
    struct Snapshot {
        bound_len: usize,
        staged_len: usize,
        staged_entries: Vec<(String, EffectNode)>,
    }

    fn sys_snapshot(
        query: Query<(Option<&BoundEffects>, &StagedEffects)>,
        mut snap: ResMut<Snapshot>,
    ) {
        for (bound, staged) in &query {
            snap.bound_len = bound.map_or(0, |b| b.0.len());
            snap.staged_len = staged.0.len();
            snap.staged_entries = staged.0.clone();
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Snapshot>();
        app
    }

    // -----------------------------------------------------------------------
    // evaluate_bound_effects tests
    // -----------------------------------------------------------------------

    #[test]
    fn bound_entry_not_consumed_after_match() {
        // BoundEffects entries are NEVER consumed, even when the trigger matches.
        // We use a non-Do child to avoid queuing fire_effect commands.
        let mut app = test_app();
        app.add_systems(Update, (sys_evaluate_bound_for_bump, sys_snapshot).chain());

        let inner = when_do(Trigger::Death, EffectKind::DamageBoost(2.0));
        let bound_node = when_child(Trigger::Bump, inner);
        app.world_mut().spawn((
            BoundEffects(vec![("chip_a".into(), bound_node)]),
            StagedEffects::default(),
        ));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(snap.bound_len, 1, "BoundEffects entry must not be consumed");
    }

    #[test]
    fn non_matching_trigger_leaves_bound_and_staged_unchanged() {
        // When the trigger does not match, nothing changes.
        let mut app = test_app();
        app.add_systems(
            Update,
            (sys_evaluate_bound_for_bump_whiff, sys_snapshot).chain(),
        );

        let node = when_do(Trigger::Bump, EffectKind::DamageBoost(2.0));
        app.world_mut().spawn((
            BoundEffects(vec![("chip_a".into(), node)]),
            StagedEffects::default(),
        ));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(snap.bound_len, 1, "BoundEffects unchanged on non-match");
        assert_eq!(snap.staged_len, 0, "StagedEffects empty on non-match");
    }

    #[test]
    fn non_do_children_pushed_to_staged_effects() {
        // When(Bump, [When(Death, Do(X))]) — the inner When is non-Do,
        // so it gets pushed to StagedEffects instead of being fired.
        let mut app = test_app();
        app.add_systems(Update, (sys_evaluate_bound_for_bump, sys_snapshot).chain());

        let inner_when = when_do(Trigger::Death, EffectKind::DamageBoost(2.0));
        let outer = when_child(Trigger::Bump, inner_when.clone());

        app.world_mut().spawn((
            BoundEffects(vec![("chip_a".into(), outer)]),
            StagedEffects::default(),
        ));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(
            snap.staged_len, 1,
            "Non-Do child should be pushed to StagedEffects"
        );
        assert_eq!(
            snap.staged_entries[0].1, inner_when,
            "Pushed node should be the inner When(Death, Do(DamageBoost))"
        );
    }

    #[test]
    fn staged_entry_consumed_on_matching_trigger() {
        // When(Bump, Do(X)) in StagedEffects — after evaluating for Bump,
        // the entry should be consumed (removed). We use a non-Do child
        // to avoid command panics, but the When itself is still consumed.
        let mut app = test_app();
        app.add_systems(Update, (sys_evaluate_staged_for_bump, sys_snapshot).chain());

        let inner = when_do(Trigger::Death, EffectKind::DamageBoost(2.0));
        let node = when_child(Trigger::Bump, inner);

        let entity = app
            .world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), node)]))
            .id();
        // Also need BoundEffects for the query in other systems, but
        // sys_evaluate_staged_for_bump only queries StagedEffects.

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(
            snap.staged_len, 1,
            "When consumed, its non-Do child should be added to StagedEffects (net 1 entry)"
        );

        // The original When(Bump) entry should be gone; the remaining entry
        // should be the inner When(Death, Do(DamageBoost)).
        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        assert!(
            matches!(
                &staged.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Death,
                    ..
                }
            ),
            "Remaining entry should be the inner When(Death) pushed as addition"
        );
    }

    #[test]
    fn staged_non_matching_trigger_retained() {
        // When(Bump, Do(X)) in StagedEffects — evaluate for BumpWhiff.
        // Entry should be retained because the trigger does not match.
        let mut app = test_app();
        app.add_systems(
            Update,
            (sys_evaluate_staged_for_bump_whiff, sys_snapshot).chain(),
        );

        let node = when_do(Trigger::Bump, EffectKind::DamageBoost(2.0));
        app.world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), node)]));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(
            snap.staged_len, 1,
            "Non-matching staged entry must be retained"
        );
    }

    #[test]
    fn once_consumed_when_child_trigger_matches() {
        // Once([When(Bump, Do(X))]) in StagedEffects — evaluate for Bump.
        // The Once should be consumed because a child matched.
        // Use non-Do inner child to avoid fire_effect panics.
        let mut app = test_app();
        app.add_systems(Update, (sys_evaluate_staged_for_bump, sys_snapshot).chain());

        let inner_when = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![when_do(Trigger::Death, EffectKind::DamageBoost(2.0))],
        };
        let once_node = EffectNode::Once(vec![inner_when]);
        app.world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), once_node)]));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        // Once is consumed; its non-Do children are pushed as additions.
        // So net staged should have 1 entry (the inner When(Death, Do(X))).
        assert_eq!(
            snap.staged_len, 1,
            "Once consumed, addition from inner When should remain"
        );
        assert!(
            matches!(
                &snap.staged_entries[0].1,
                EffectNode::When {
                    trigger: Trigger::Death,
                    ..
                }
            ),
            "The addition should be the inner non-Do node"
        );
    }

    #[test]
    fn once_retained_when_no_child_matches() {
        // Once([When(Bump, Do(X))]) in StagedEffects — evaluate for Death.
        // No child matches, so Once should be retained.
        let mut app = test_app();
        app.add_systems(
            Update,
            (sys_evaluate_staged_for_death, sys_snapshot).chain(),
        );

        let inner_when = when_do(Trigger::Bump, EffectKind::DamageBoost(2.0));
        let once_node = EffectNode::Once(vec![inner_when]);
        app.world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), once_node)]));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(
            snap.staged_len, 1,
            "Once must be retained when no child matches"
        );
        assert!(
            matches!(&snap.staged_entries[0].1, EffectNode::Once(_)),
            "Retained entry should still be an Once node"
        );
    }

    #[test]
    fn bare_do_in_staged_not_consumed() {
        // A bare Do(X) in StagedEffects should not be consumed by any trigger
        // evaluation — walk_staged_node returns false for non-When/non-Once.
        let mut app = test_app();
        app.add_systems(Update, (sys_evaluate_staged_for_bump, sys_snapshot).chain());

        let do_node = EffectNode::Do(EffectKind::DamageBoost(2.0));
        app.world_mut()
            .spawn(StagedEffects(vec![("chip_a".into(), do_node)]));

        app.update();

        let snap = app.world().resource::<Snapshot>();
        assert_eq!(
            snap.staged_len, 1,
            "Bare Do in StagedEffects must not be consumed"
        );
        assert!(
            matches!(&snap.staged_entries[0].1, EffectNode::Do(_)),
            "Retained entry should still be a Do node"
        );
    }

    // -- Section J: EffectSourceChip threading through evaluate ───────────────────

    use crate::effect::{core::EffectSourceChip, effects::explode::ExplodeRequest};

    fn evaluate_source_chip_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sys_evaluate_bound_for_bump);
        app.add_systems(Update, sys_evaluate_staged_for_bump);
        app
    }

    #[test]
    fn evaluate_bound_effects_threads_chip_name_as_source_chip_to_fire_effect() {
        let mut app = evaluate_source_chip_test_app();

        let node = when_do(
            Trigger::Bump,
            EffectKind::Explode {
                range: 60.0,
                damage_mult: 2.0,
            },
        );

        app.world_mut().spawn((
            BoundEffects(vec![("resonance".into(), node)]),
            StagedEffects::default(),
            Transform::from_xyz(50.0, 50.0, 0.0),
        ));

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&EffectSourceChip, With<ExplodeRequest>>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected one entity with EffectSourceChip (on ExplodeRequest)"
        );
        assert_eq!(
            results[0].0,
            Some("resonance".to_string()),
            "evaluate_bound_effects should thread chip_name 'resonance' to fire_effect"
        );
    }

    #[test]
    fn evaluate_bound_effects_empty_chip_name_produces_none() {
        let mut app = evaluate_source_chip_test_app();

        let node = when_do(
            Trigger::Bump,
            EffectKind::Explode {
                range: 60.0,
                damage_mult: 2.0,
            },
        );

        app.world_mut().spawn((
            BoundEffects(vec![(String::new(), node)]),
            StagedEffects::default(),
            Transform::from_xyz(50.0, 50.0, 0.0),
        ));

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&EffectSourceChip, With<ExplodeRequest>>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected one entity with EffectSourceChip"
        );
        assert_eq!(
            results[0].0, None,
            "empty chip_name should produce EffectSourceChip(None)"
        );
    }

    #[test]
    fn evaluate_staged_effects_threads_chip_name_as_source_chip_to_fire_effect() {
        let mut app = evaluate_source_chip_test_app();

        let node = when_do(
            Trigger::Bump,
            EffectKind::Explode {
                range: 60.0,
                damage_mult: 2.0,
            },
        );

        app.world_mut().spawn((
            StagedEffects(vec![("zapper".into(), node)]),
            Transform::from_xyz(50.0, 50.0, 0.0),
        ));

        app.update();
        app.update();

        let mut query = app.world_mut().query::<&EffectSourceChip>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected one entity with EffectSourceChip (on ExplodeRequest)"
        );
        assert_eq!(
            results[0].0,
            Some("zapper".to_string()),
            "evaluate_staged_effects should thread chip_name 'zapper' to fire_effect"
        );
    }

    #[test]
    fn once_node_preserves_chip_name_through_to_fire_effect() {
        // Once([When(Bump, Do(Explode))]) in StagedEffects — chip_name should
        // be threaded through Once dispatch to the inner fire_effect call.
        let mut app = evaluate_source_chip_test_app();

        let inner_when = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::Explode {
                range: 60.0,
                damage_mult: 2.0,
            })],
        };
        let once_node = EffectNode::Once(vec![inner_when]);

        app.world_mut().spawn((
            StagedEffects(vec![("once_chip".into(), once_node)]),
            Transform::from_xyz(50.0, 50.0, 0.0),
        ));

        app.update();
        app.update();

        let mut query = app.world_mut().query::<&EffectSourceChip>();
        let results: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            results.len(),
            1,
            "expected one entity with EffectSourceChip"
        );
        assert_eq!(
            results[0].0,
            Some("once_chip".to_string()),
            "Once node should preserve chip_name through to fire_effect"
        );
    }
}

#[cfg(test)]
mod on_resolution_tests {
    //! Tests for On-node handling in `walk_bound_node`, `walk_staged_node`,
    //! and `ResolveOnCommand` resolution (behaviors 14-24).

    use bevy::prelude::*;

    use super::*;
    use crate::{
        bolt::components::Bolt,
        breaker::components::Breaker,
        cells::components::Cell,
        effect::{commands::ResolveOnCommand, effects::speed_boost::ActiveSpeedBoosts},
        wall::components::Wall,
    };

    // -----------------------------------------------------------------------
    // Test helper systems
    // -----------------------------------------------------------------------

    fn sys_evaluate_bound_for_node_start(
        mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::NodeStart;
        for (entity, bound, mut staged) in &mut query {
            evaluate_bound_effects(&trigger, entity, bound, &mut staged, &mut commands);
        }
    }

    fn sys_evaluate_staged_for_node_start(
        mut query: Query<(Entity, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::NodeStart;
        for (entity, mut staged) in &mut query {
            evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
        }
    }

    fn sys_evaluate_staged_for_bump(
        mut query: Query<(Entity, &mut StagedEffects)>,
        mut commands: Commands,
    ) {
        let trigger = Trigger::Bump;
        for (entity, mut staged) in &mut query {
            evaluate_staged_effects(&trigger, entity, &mut staged, &mut commands);
        }
    }

    // -----------------------------------------------------------------------
    // Section K: walk_bound_node pushes On children to StagedEffects
    // -----------------------------------------------------------------------

    // ── Behavior 14: walk_bound_node pushes On children to StagedEffects ──

    #[test]
    fn walk_bound_node_pushes_on_child_to_staged_effects_when_trigger_matches() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let bound_node = EffectNode::When {
            trigger: Trigger::NodeStart,
            then: vec![EffectNode::On {
                target: Target::AllCells,
                permanent: true,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            }],
        };

        let entity = app
            .world_mut()
            .spawn((
                BoundEffects(vec![("cell_fortify".into(), bound_node)]),
                StagedEffects::default(),
            ))
            .id();

        app.add_systems(Update, sys_evaluate_bound_for_node_start);
        app.update();

        // After evaluation, StagedEffects should have 1 entry (the On node)
        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "StagedEffects should have 1 entry (the On node pushed from walk_bound_node)"
        );
        assert_eq!(staged.0[0].0, "cell_fortify", "chip_name preserved");
        assert!(
            matches!(
                &staged.0[0].1,
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1
            ),
            "Pushed entry should be the On(AllCells, permanent: true, ...) node, got {:?}",
            staged.0[0].1
        );

        // BoundEffects should be unchanged (entries are never consumed)
        let bound = app.world().get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "BoundEffects entry must not be consumed");
    }

    // ── Behavior 14 edge case: On node with multiple children ──

    #[test]
    fn walk_bound_node_pushes_on_with_multiple_children_as_single_entry() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let bound_node = EffectNode::When {
            trigger: Trigger::NodeStart,
            then: vec![EffectNode::On {
                target: Target::AllBolts,
                permanent: true,
                then: vec![
                    EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                    },
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(EffectKind::Shockwave {
                            base_range: 64.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 500.0,
                        })],
                    },
                ],
            }],
        };

        let entity = app
            .world_mut()
            .spawn((
                BoundEffects(vec![("bolt_buff".into(), bound_node)]),
                StagedEffects::default(),
            ))
            .id();

        app.add_systems(Update, sys_evaluate_bound_for_node_start);
        app.update();

        let staged = app.world().get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "Entire On node (with both children) should be pushed as a single entry"
        );

        if let EffectNode::On { then: inner, .. } = &staged.0[0].1 {
            assert_eq!(
                inner.len(),
                2,
                "On node should have 2 children (both When nodes)"
            );
        } else {
            panic!("Expected On(...) in StagedEffects, got {:?}", staged.0[0].1);
        }
    }

    // -----------------------------------------------------------------------
    // Section L: walk_staged_node handles On nodes via ResolveOnCommand
    // -----------------------------------------------------------------------

    // ── Behavior 15: On in StagedEffects consumed and children transferred to targets ──

    #[test]
    fn on_node_in_staged_effects_consumed_and_resolved_to_target_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Source entity with the On node in StagedEffects
        let source = app
            .world_mut()
            .spawn(StagedEffects(vec![(
                "cell_fortify".into(),
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                    }],
                },
            )]))
            .id();

        // Target Cell entities
        let cell_a = app
            .world_mut()
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();
        let cell_b = app
            .world_mut()
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();

        app.add_systems(Update, sys_evaluate_staged_for_node_start);
        // First update: evaluate_staged_effects runs, queues ResolveOnCommand
        app.update();

        // The On entry should be consumed from source's StagedEffects
        let staged = app.world().get::<StagedEffects>(source).unwrap();
        assert_eq!(
            staged.0.len(),
            0,
            "On node should be consumed from StagedEffects after evaluation"
        );

        // After command application, each Cell should have BoundEffects updated
        for (label, cell) in [("cell_a", cell_a), ("cell_b", cell_b)] {
            let bound = app.world().get::<BoundEffects>(cell).unwrap();
            assert_eq!(
                bound.0.len(),
                1,
                "{label} should have 1 BoundEffects entry after ResolveOnCommand"
            );
            assert_eq!(bound.0[0].0, "cell_fortify", "{label} chip_name preserved");
            assert!(
                matches!(
                    &bound.0[0].1,
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        ..
                    }
                ),
                "{label} should have When(Impacted(Bolt), ...) in BoundEffects, got {:?}",
                bound.0[0].1
            );
        }
    }

    // ── Behavior 15 edge case: permanent: false sends to StagedEffects ──

    #[test]
    fn on_node_with_permanent_false_sends_children_to_staged_effects_on_targets() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let source = app
            .world_mut()
            .spawn(StagedEffects(vec![(
                "cell_fortify".into(),
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: false,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                    }],
                },
            )]))
            .id();

        let cell = app
            .world_mut()
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();

        app.add_systems(Update, sys_evaluate_staged_for_node_start);
        app.update();

        // On node consumed from source
        let staged = app.world().get::<StagedEffects>(source).unwrap();
        assert_eq!(staged.0.len(), 0, "On node should be consumed");

        // With permanent: false, children go to StagedEffects (not BoundEffects)
        let cell_staged = app.world().get::<StagedEffects>(cell).unwrap();
        assert_eq!(
            cell_staged.0.len(),
            1,
            "Cell should have 1 StagedEffects entry (permanent: false)"
        );

        let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert!(
            cell_bound.0.is_empty(),
            "Cell BoundEffects should remain empty when permanent: false"
        );
    }

    // -----------------------------------------------------------------------
    // ResolveOnCommand unit tests (behaviors 16-23)
    // -----------------------------------------------------------------------

    // ── Behavior 16: ResolveOnCommand resolves AllCells ──

    #[test]
    fn resolve_on_command_resolves_all_cells_to_cell_entities() {
        let mut world = World::new();
        let cell_a = world
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();
        let cell_b = world
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();
        let cell_c = world
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();

        // Also spawn non-target entities to ensure they are not affected
        let breaker = world
            .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
            .id();
        let bolt = world
            .spawn((
                Bolt,
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts::default(),
            ))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllCells,
            chip_name: "cell_fortify".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        // Each Cell should have 1 BoundEffects entry
        for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b), ("cell_c", cell_c)] {
            let bound = world.get::<BoundEffects>(entity).unwrap();
            assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
            assert_eq!(bound.0[0].0, "cell_fortify", "{label} chip_name");
            assert!(
                matches!(
                    &bound.0[0].1,
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        ..
                    }
                ),
                "{label} node should be When(Impacted(Bolt), ...)"
            );
        }

        // Non-target entities should be unaffected
        let breaker_bound = world.get::<BoundEffects>(breaker).unwrap();
        assert!(
            breaker_bound.0.is_empty(),
            "Breaker BoundEffects should be unchanged"
        );
        let bolt_bound = world.get::<BoundEffects>(bolt).unwrap();
        assert!(
            bolt_bound.0.is_empty(),
            "Bolt BoundEffects should be unchanged"
        );
    }

    // ── Behavior 16 edge case: AllCells with Do children fires immediately ──

    #[test]
    fn resolve_on_command_all_cells_with_do_children_fires_immediately() {
        use crate::effect::effects::damage_boost::ActiveDamageBoosts;

        let mut world = World::new();
        let cell = world
            .spawn((
                Cell,
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveDamageBoosts::default(),
            ))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllCells,
            chip_name: "cell_damage".to_string(),
            children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            permanent: true,
        };
        cmd.apply(&mut world);

        // Do child should fire immediately
        let boosts = world.get::<ActiveDamageBoosts>(cell).unwrap();
        assert_eq!(
            boosts.0,
            vec![2.0],
            "Do child should fire immediately on Cell entity"
        );

        // BoundEffects should remain empty (Do fires, not pushed)
        let bound = world.get::<BoundEffects>(cell).unwrap();
        assert!(
            bound.0.is_empty(),
            "BoundEffects should remain empty when only Do children"
        );
    }

    // ── Behavior 17: ResolveOnCommand resolves AllBolts ──

    #[test]
    fn resolve_on_command_resolves_all_bolts_to_bolt_entities() {
        let mut world = World::new();
        let bolt_a = world
            .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
            .id();
        let bolt_b = world
            .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllBolts,
            chip_name: "bolt_chain".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 500.0,
                })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
            let bound = world.get::<BoundEffects>(entity).unwrap();
            assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
            assert_eq!(bound.0[0].0, "bolt_chain", "{label} chip_name");
        }
    }

    // ── Behavior 17 edge case: AllBolts with multiple children ──

    #[test]
    fn resolve_on_command_all_bolts_with_multiple_children() {
        let mut world = World::new();
        let bolt = world
            .spawn((Bolt, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllBolts,
            chip_name: "bolt_chain".to_string(),
            children: vec![
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
                },
            ],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world.get::<BoundEffects>(bolt).unwrap();
        assert_eq!(
            bound.0.len(),
            2,
            "Bolt should have 2 BoundEffects entries (one per child)"
        );
        assert_eq!(bound.0[0].0, "bolt_chain");
        assert_eq!(bound.0[1].0, "bolt_chain");
    }

    // ── Behavior 18: ResolveOnCommand resolves AllWalls ──

    #[test]
    fn resolve_on_command_resolves_all_walls_to_wall_entities() {
        let mut world = World::new();
        let wall_a = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();
        let wall_b = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllWalls,
            chip_name: "wall_boost".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        for (label, entity) in [("wall_a", wall_a), ("wall_b", wall_b)] {
            let bound = world.get::<BoundEffects>(entity).unwrap();
            assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
            assert_eq!(bound.0[0].0, "wall_boost", "{label} chip_name");
        }
    }

    // ── Behavior 18 edge case: Single Wall entity ──

    #[test]
    fn resolve_on_command_all_walls_with_single_wall() {
        let mut world = World::new();
        let wall = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllWalls,
            chip_name: "wall_boost".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world.get::<BoundEffects>(wall).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "Single Wall should have 1 BoundEffects entry"
        );
    }

    // ── Behavior 19: Bolt target resolves to all Bolt entities ──

    #[test]
    fn resolve_on_command_bolt_target_fires_do_on_all_bolt_entities() {
        let mut world = World::new();
        let bolt_a = world
            .spawn((
                Bolt,
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts::default(),
            ))
            .id();
        let bolt_b = world
            .spawn((
                Bolt,
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts::default(),
            ))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::Bolt,
            chip_name: "bolt_speed".to_string(),
            children: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
            permanent: true,
        };
        cmd.apply(&mut world);

        for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
            let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
            assert_eq!(
                speed.0,
                vec![1.2],
                "{label} should have ActiveSpeedBoosts [1.2] from fired Do"
            );
        }
    }

    // ── Behavior 19 edge case: Single Bolt entity ──

    #[test]
    fn resolve_on_command_bolt_target_with_single_bolt() {
        let mut world = World::new();
        let bolt = world
            .spawn((
                Bolt,
                BoundEffects::default(),
                StagedEffects::default(),
                ActiveSpeedBoosts::default(),
            ))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::Bolt,
            chip_name: "bolt_speed".to_string(),
            children: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
            permanent: true,
        };
        cmd.apply(&mut world);

        let speed = world.get::<ActiveSpeedBoosts>(bolt).unwrap();
        assert_eq!(
            speed.0,
            vec![1.2],
            "Single Bolt should have ActiveSpeedBoosts [1.2]"
        );
    }

    // ── Behavior 20: Cell target resolves to all Cell entities ──

    #[test]
    fn resolve_on_command_cell_target_resolves_to_all_cell_entities() {
        let mut world = World::new();
        let cell_a = world
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();
        let cell_b = world
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::Cell,
            chip_name: "cell_armor".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 2 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b)] {
            let bound = world.get::<BoundEffects>(entity).unwrap();
            assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
            assert_eq!(bound.0[0].0, "cell_armor", "{label} chip_name");
        }
    }

    // ── Behavior 21: Wall target resolves to all Wall entities ──

    #[test]
    fn resolve_on_command_wall_target_resolves_to_all_wall_entities() {
        let mut world = World::new();
        let wall = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::Wall,
            chip_name: "wall_reflect".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world.get::<BoundEffects>(wall).unwrap();
        assert_eq!(bound.0.len(), 1);
        assert_eq!(bound.0[0].0, "wall_reflect");
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    ..
                }
            ),
            "Wall entry should be When(Impacted(Bolt), ...)"
        );
    }

    // ── Behavior 21 edge case: Three Wall entities ──

    #[test]
    fn resolve_on_command_wall_target_with_three_walls() {
        let mut world = World::new();
        let wall_a = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();
        let wall_b = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();
        let wall_c = world
            .spawn((Wall, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::Wall,
            chip_name: "wall_reflect".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        for (label, entity) in [("wall_a", wall_a), ("wall_b", wall_b), ("wall_c", wall_c)] {
            let bound = world.get::<BoundEffects>(entity).unwrap();
            assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
        }
    }

    // ── Behavior 22: No matching entities — no-op ──

    #[test]
    fn resolve_on_command_with_no_matching_entities_is_noop() {
        let mut world = World::new();

        // Spawn a Breaker but target AllCells — no Cell entities exist
        let breaker = world
            .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::AllCells,
            chip_name: "cell_fortify".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
            permanent: true,
        };
        // Should not panic
        cmd.apply(&mut world);

        // Breaker's BoundEffects should remain empty
        let breaker_bound = world.get::<BoundEffects>(breaker).unwrap();
        assert!(
            breaker_bound.0.is_empty(),
            "Breaker BoundEffects should remain empty (not an AllCells target)"
        );
    }

    // ── Behavior 22 edge case: AllBolts with no bolts ──

    #[test]
    fn resolve_on_command_all_bolts_with_no_bolts_is_noop() {
        let mut world = World::new();

        let cmd = ResolveOnCommand {
            target: Target::AllBolts,
            chip_name: "bolt_chain".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 500.0,
                })],
            }],
            permanent: true,
        };
        // Should not panic
        cmd.apply(&mut world);
    }

    // ── Behavior 22 edge case: AllWalls with no walls ──

    #[test]
    fn resolve_on_command_all_walls_with_no_walls_is_noop() {
        let mut world = World::new();

        let cmd = ResolveOnCommand {
            target: Target::AllWalls,
            chip_name: "wall_boost".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
            permanent: true,
        };
        // Should not panic
        cmd.apply(&mut world);
    }

    // ── Behavior 23: Breaker target resolves to Breaker entity ──

    #[test]
    fn resolve_on_command_breaker_target_resolves_to_breaker_entity() {
        let mut world = World::new();
        let breaker = world
            .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
            .id();

        let cmd = ResolveOnCommand {
            target: Target::Breaker,
            chip_name: "breaker_buff".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
            permanent: true,
        };
        cmd.apply(&mut world);

        let bound = world.get::<BoundEffects>(breaker).unwrap();
        assert_eq!(bound.0.len(), 1, "Breaker should have 1 BoundEffects entry");
        assert_eq!(bound.0[0].0, "breaker_buff");
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    ..
                }
            ),
            "Breaker entry should be When(PerfectBump, ...)"
        );
    }

    // ── Behavior 23 edge case: Breaker target with no Breaker entity ──

    #[test]
    fn resolve_on_command_breaker_target_with_no_breaker_is_noop() {
        let mut world = World::new();

        let cmd = ResolveOnCommand {
            target: Target::Breaker,
            chip_name: "breaker_buff".to_string(),
            children: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
            permanent: true,
        };
        // Should not panic
        cmd.apply(&mut world);
    }

    // ── Behavior 24: On node in StagedEffects consumed regardless of trigger ──

    #[test]
    fn on_node_in_staged_effects_consumed_regardless_of_trigger() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let source = app
            .world_mut()
            .spawn(StagedEffects(vec![(
                "cell_fortify".into(),
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                    }],
                },
            )]))
            .id();

        let cell = app
            .world_mut()
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();

        // Evaluate for Bump (NOT NodeStart) — On node should still be consumed
        app.add_systems(Update, sys_evaluate_staged_for_bump);
        app.update();

        let staged = app.world().get::<StagedEffects>(source).unwrap();
        assert_eq!(
            staged.0.len(),
            0,
            "On node should be consumed regardless of which trigger is being evaluated"
        );

        // Cell should still get the resolved entry
        let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert_eq!(
            cell_bound.0.len(),
            1,
            "Cell should have 1 BoundEffects entry from the resolved On node"
        );
    }

    // ── Behavior 24 edge case: Mixed On and When in StagedEffects ──

    #[test]
    fn mixed_on_and_when_in_staged_effects_both_consumed_when_trigger_matches() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let source = app
            .world_mut()
            .spawn(StagedEffects(vec![
                (
                    "chip_a".into(),
                    EffectNode::On {
                        target: Target::AllCells,
                        permanent: true,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Impacted(ImpactTarget::Bolt),
                            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                        }],
                    },
                ),
                (
                    "chip_b".into(),
                    EffectNode::When {
                        trigger: Trigger::Bump,
                        then: vec![EffectNode::When {
                            trigger: Trigger::Death,
                            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                        }],
                    },
                ),
            ]))
            .id();

        let _cell = app
            .world_mut()
            .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
            .id();

        // Evaluate for Bump: On consumed (trigger-independent), When(Bump) also consumed
        app.add_systems(Update, sys_evaluate_staged_for_bump);
        app.update();

        let staged = app.world().get::<StagedEffects>(source).unwrap();
        // The On node is consumed, the When(Bump) is consumed (its non-Do child When(Death) is added),
        // so we expect 1 addition from the When(Bump) match
        assert_eq!(
            staged.0.len(),
            1,
            "After evaluation: On consumed, When(Bump) consumed, When(Death) added as addition. Net: 1"
        );
        assert!(
            matches!(
                &staged.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Death,
                    ..
                }
            ),
            "Remaining entry should be the When(Death, ...) addition from the consumed When(Bump)"
        );
    }
}
