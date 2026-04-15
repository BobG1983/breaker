//! Until node evaluator — event-scoped effect application.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::effect_v3::{
    conditions::DuringActive,
    dispatch::{fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch},
    storage::BoundEffects,
    types::{ScopedTree, Tree, Trigger, TriggerContext},
};

/// Tracks which Until sources have already applied their inner effects
/// on this entity. Each entry in the `HashSet` is a source name string.
#[derive(Component, Default, Debug)]
pub struct UntilApplied(pub HashSet<String>);

/// Evaluate a `Tree::Until` node: apply inner effects immediately,
/// reverse them when the trigger fires.
pub fn evaluate_until(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &ScopedTree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    let _ = context;
    commands.queue(UntilEvaluateCommand {
        entity,
        gate_trigger: gate_trigger.clone(),
        active_trigger: active_trigger.clone(),
        inner: inner.clone(),
        source: source.to_owned(),
    });
}

/// Deferred command that performs the Until state-machine logic with world access.
struct UntilEvaluateCommand {
    entity:         Entity,
    gate_trigger:   Trigger,
    active_trigger: Trigger,
    inner:          ScopedTree,
    source:         String,
}

impl Command for UntilEvaluateCommand {
    fn apply(self, world: &mut World) {
        // Guard: entity must still exist
        if world.get_entity(self.entity).is_err() {
            return;
        }

        // Ensure UntilApplied component exists on entity
        if world.get::<UntilApplied>(self.entity).is_none() {
            world
                .entity_mut(self.entity)
                .insert(UntilApplied::default());
        }

        // Check if this source has already been applied
        let is_applied = world
            .get::<UntilApplied>(self.entity)
            .is_some_and(|ua| ua.0.contains(&self.source));

        match &self.inner {
            ScopedTree::During(condition, inner_scoped) => {
                // Shape B: Until wraps a During
                if !is_applied {
                    // Install the During into BoundEffects for the condition poller
                    let install_key = format!("{}#installed[0]", self.source);
                    if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity)
                        && !bound.0.iter().any(|(name, _)| name == &install_key)
                    {
                        bound.0.push((
                            install_key,
                            Tree::During(condition.clone(), Box::new((**inner_scoped).clone())),
                        ));
                    }
                    // Mark as applied
                    if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                        ua.0.insert(self.source.clone());
                    }

                    // If gate matches on first walk, immediately tear down
                    if self.gate_trigger == self.active_trigger {
                        teardown_installed_during(self.entity, &self.source, inner_scoped, world);
                        // Clean up Until tracking
                        if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                            ua.0.remove(&self.source);
                        }
                        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                            bound.0.retain(|(name, _)| name != &self.source);
                        }
                    }
                } else if self.gate_trigger == self.active_trigger {
                    // Applied and gate fires: tear down
                    teardown_installed_during(self.entity, &self.source, inner_scoped, world);
                    if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                        ua.0.remove(&self.source);
                    }
                    if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                        bound.0.retain(|(name, _)| name != &self.source);
                    }
                }
                // else: APPLIED but gate doesn't match — no-op (During polls normally)
            }
            _ => {
                // Existing logic: fire/reverse for Fire/Sequence/When/On
                if !is_applied {
                    fire_scoped_tree(&self.inner, self.entity, &self.source, world);
                    if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                        ua.0.insert(self.source.clone());
                    }

                    // If gate matches on first walk, immediately reverse and clean up
                    if self.gate_trigger == self.active_trigger {
                        reverse_scoped_tree(&self.inner, self.entity, &self.source, world);
                        if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                            ua.0.remove(&self.source);
                        }
                        if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                            bound.0.retain(|(name, _)| name != &self.source);
                        }
                    }
                } else if self.gate_trigger == self.active_trigger {
                    reverse_scoped_tree(&self.inner, self.entity, &self.source, world);
                    if let Some(mut ua) = world.get_mut::<UntilApplied>(self.entity) {
                        ua.0.remove(&self.source);
                    }
                    if let Some(mut bound) = world.get_mut::<BoundEffects>(self.entity) {
                        bound.0.retain(|(name, _)| name != &self.source);
                    }
                }
                // else: APPLIED but gate doesn't match — no-op
            }
        }
    }
}

/// Apply scoped tree effects (fire phase).
fn fire_scoped_tree(inner: &ScopedTree, entity: Entity, source: &str, world: &mut World) {
    match inner {
        ScopedTree::Fire(reversible) => {
            fire_reversible_dispatch(reversible, entity, source, world);
        }
        ScopedTree::Sequence(effects) => {
            for reversible in effects {
                fire_reversible_dispatch(reversible, entity, source, world);
            }
        }
        ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {
            // Nested When/On/During inside Until: conditional/redirected behavior
            // that fires during future walks, not during initial application.
        }
    }
}

/// Reverse scoped tree effects (reversal phase).
fn reverse_scoped_tree(inner: &ScopedTree, entity: Entity, source: &str, world: &mut World) {
    match inner {
        ScopedTree::Fire(reversible) => {
            reverse_dispatch(reversible, entity, source, world);
        }
        ScopedTree::Sequence(effects) => {
            for reversible in effects {
                reverse_dispatch(reversible, entity, source, world);
            }
        }
        ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {
            // Nested When/On/During inside Until: no explicit reversal needed.
        }
    }
}

/// Tear down an installed During from `BoundEffects` and reverse any
/// active effects it fired. Used by Shape B (Until wrapping During).
fn teardown_installed_during(
    entity: Entity,
    source: &str,
    inner_scoped: &ScopedTree,
    world: &mut World,
) {
    let install_key = format!("{source}#installed[0]");

    // Remove installed During from BoundEffects
    if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
        bound.0.retain(|(name, _)| name != &install_key);
    }

    // Check if the During was active and reverse its effects
    let was_active = world
        .get::<DuringActive>(entity)
        .is_some_and(|da| da.0.contains(&install_key));

    if was_active {
        reverse_scoped_tree_by_source(inner_scoped, entity, &install_key, world);
        if let Some(mut da) = world.get_mut::<DuringActive>(entity) {
            da.0.remove(&install_key);
        }
    }
}

/// Reverse scoped tree effects using `reverse_all_by_source_dispatch`,
/// which removes all instances fired from the given source. Used during
/// Shape B teardown where the install key is the source.
fn reverse_scoped_tree_by_source(
    tree: &ScopedTree,
    entity: Entity,
    source: &str,
    world: &mut World,
) {
    match tree {
        ScopedTree::Fire(effect) => {
            reverse_all_by_source_dispatch(effect, entity, source, world);
        }
        ScopedTree::Sequence(effects) => {
            for effect in effects {
                reverse_all_by_source_dispatch(effect, entity, source, world);
            }
        }
        ScopedTree::When(..) | ScopedTree::On(..) | ScopedTree::During(..) => {}
    }
}
