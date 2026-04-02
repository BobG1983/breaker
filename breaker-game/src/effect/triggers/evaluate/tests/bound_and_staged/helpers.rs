//! Shared test helpers for `evaluate_bound_effects` and `evaluate_staged_effects`.

use bevy::prelude::*;

use super::super::super::system::*;
pub(super) use crate::effect::core::*;

/// Helper: build a `When(trigger, [Do(effect)])` node.
pub(super) fn when_do(trigger: Trigger, effect: EffectKind) -> EffectNode {
    EffectNode::When {
        trigger,
        then: vec![EffectNode::Do(effect)],
    }
}

/// Helper: build a `When(trigger, [child])` node with an arbitrary child.
pub(super) fn when_child(trigger: Trigger, child: EffectNode) -> EffectNode {
    EffectNode::When {
        trigger,
        then: vec![child],
    }
}

// -----------------------------------------------------------------------
// Test systems: wrap evaluate_* so we can run them inside an App.
// Each system evaluates for Trigger::Bump on all matching entities.
// -----------------------------------------------------------------------

pub(super) fn sys_evaluate_bound_for_bump(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::Bump;
    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(
            &trigger,
            entity,
            bound,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

pub(super) fn sys_evaluate_staged_for_bump(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::Bump;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(
            &trigger,
            entity,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

pub(super) fn sys_evaluate_bound_for_bump_whiff(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::BumpWhiff;
    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(
            &trigger,
            entity,
            bound,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

pub(super) fn sys_evaluate_staged_for_bump_whiff(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::BumpWhiff;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(
            &trigger,
            entity,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

pub(super) fn sys_evaluate_staged_for_death(
    mut query: Query<(Entity, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let trigger = Trigger::Death;
    for (entity, mut staged) in &mut query {
        evaluate_staged_effects(
            &trigger,
            entity,
            &mut staged,
            &mut commands,
            TriggerContext::default(),
        );
    }
}

/// Resource to snapshot component state BEFORE commands are applied.
/// A system reads `BoundEffects`/`StagedEffects` and stores a copy here.
#[derive(Resource, Default)]
pub(super) struct Snapshot {
    pub(super) bound_len: usize,
    pub(super) staged_len: usize,
    pub(super) staged_entries: Vec<(String, EffectNode)>,
}

pub(super) fn sys_snapshot(
    query: Query<(Option<&BoundEffects>, &StagedEffects)>,
    mut snap: ResMut<Snapshot>,
) {
    for (bound, staged) in &query {
        snap.bound_len = bound.map_or(0, |b| b.0.len());
        snap.staged_len = staged.0.len();
        snap.staged_entries = staged.0.clone();
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Snapshot>();
    app
}
