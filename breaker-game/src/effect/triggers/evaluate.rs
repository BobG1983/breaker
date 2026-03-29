use bevy::prelude::*;

use crate::effect::{commands::EffectCommandsExt, core::*};

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
