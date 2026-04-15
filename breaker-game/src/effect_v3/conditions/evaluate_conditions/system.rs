//! `evaluate_conditions` — per-frame condition polling system for During nodes.

use std::collections::HashSet;

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::{is_combo_active, is_node_active, is_shield_active};
use crate::effect_v3::{
    commands::EffectCommandsExt,
    dispatch::{fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch},
    storage::{ArmedFiredParticipants, BoundEffects},
    types::{Condition, ReversibleEffectType, ScopedTerminal, ScopedTree, Terminal, Tree},
};

/// Tracks which During sources have their effects currently applied
/// on this entity. Each entry in the `HashSet` is a source name string.
#[derive(Component, Default, Debug)]
pub struct DuringActive(pub HashSet<String>);

/// Poll all registered conditions each frame and fire/reverse During
/// entries on state transitions.
///
/// Runs in `EffectV3Systems::Conditions`.
pub fn evaluate_conditions(world: &mut World) {
    // Phase 1: Collect During entries (need immutable borrow first)
    let mut during_entries: Vec<(Entity, String, Condition, ScopedTree)> = Vec::new();
    let mut query = world.query::<(Entity, &BoundEffects)>();
    for (entity, bound) in query.iter(world) {
        for (source, tree) in &bound.0 {
            if let Tree::During(condition, inner) = tree {
                during_entries.push((entity, source.clone(), condition.clone(), (**inner).clone()));
            }
        }
    }

    // Phase 2: Evaluate conditions and manage transitions
    for (entity, source, condition, inner) in during_entries {
        if world.get_entity(entity).is_err() {
            continue;
        }

        let is_true = evaluate_condition(&condition, world);

        // Ensure DuringActive exists
        if world.get::<DuringActive>(entity).is_none() {
            world.entity_mut(entity).insert(DuringActive::default());
        }

        let was_active = world
            .get::<DuringActive>(entity)
            .is_some_and(|da| da.0.contains(&source));

        if !was_active && is_true {
            fire_scoped_tree(&inner, entity, &source, world);
            if let Some(mut da) = world.get_mut::<DuringActive>(entity) {
                da.0.insert(source);
            }
        } else if was_active && !is_true {
            reverse_scoped_tree(&inner, entity, &source, world);
            if let Some(mut da) = world.get_mut::<DuringActive>(entity) {
                da.0.remove(&source);
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
        ScopedTree::When(trigger, inner_tree) => {
            let armed_key = format!("{source}#armed[0]");
            let armed_tree = Tree::When(trigger.clone(), inner_tree.clone());
            install_armed_entry(entity, armed_key, armed_tree, world);
        }
        ScopedTree::On(participant, scoped_terminal) => {
            let armed_key = format!("{source}#armed[0]");
            let terminal = Terminal::from(scoped_terminal.clone());
            let armed_tree = Tree::On(*participant, terminal);
            install_armed_entry(entity, armed_key, armed_tree, world);
        }
        ScopedTree::During(..) => {
            // Nested During inside During: handled by Shape A (wave 7b install pattern).
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
        ScopedTree::When(_trigger, inner_tree) => {
            let armed_key = format!("{source}#armed[0]");
            // Remove armed entry from BoundEffects
            if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
                bound.0.retain(|(name, _)| name != &armed_key);
            }
            // Reverse any effects the armed When may have fired
            reverse_armed_tree(inner_tree, entity, &armed_key, world);
        }
        ScopedTree::On(_participant, scoped_terminal) => {
            let armed_key = format!("{source}#armed[0]");

            // Remove armed entry from BoundEffects.
            if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
                bound.0.retain(|(name, _)| name != &armed_key);
            }

            // Drain tracked fired participants on the owner for this
            // armed source. If the owner never fired (or the component
            // was never inserted), `drain` returns an empty `Vec` —
            // no panic, no reverse.
            let tracked: Vec<Entity> = world
                .get_mut::<ArmedFiredParticipants>(entity)
                .map(|mut comp| comp.drain(&armed_key))
                .unwrap_or_default();

            // For each fired participant, reverse one instance of the
            // effect via `commands.reverse_effect()`. Single-instance
            // semantics match the single-instance fire that produced
            // the tracking entry, so N fires → N reverses.
            if let ScopedTerminal::Fire(reversible) = scoped_terminal {
                let mut queue = CommandQueue::default();
                {
                    let mut commands = Commands::new(&mut queue, world);
                    for participant in tracked {
                        commands.reverse_effect(participant, reversible.clone(), armed_key.clone());
                    }
                }
                queue.apply(world);
            }
            // `ScopedTerminal::Route` in an armed `On` context has no
            // reversal semantics and is not produced by any current
            // authoring path; tracked entries for a Route variant are
            // dropped harmlessly by the drain above.
        }
        ScopedTree::During(..) => {
            // Nested During: handled by Shape A reversal.
        }
    }
}

/// Helper to evaluate a single condition against world state.
///
/// Used by the During state machine to check condition transitions.
#[must_use]
pub fn evaluate_condition(condition: &Condition, world: &World) -> bool {
    match condition {
        Condition::NodeActive => is_node_active(world),
        Condition::ShieldActive => is_shield_active(world),
        Condition::ComboActive(threshold) => is_combo_active(world, *threshold),
    }
}

/// Install a tree into `BoundEffects` with idempotency guard.
fn install_armed_entry(entity: Entity, armed_key: String, tree: Tree, world: &mut World) {
    if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
        if bound.0.iter().any(|(name, _)| name == &armed_key) {
            return; // Already installed — idempotent
        }
        bound.0.push((armed_key, tree));
    } else {
        world
            .entity_mut(entity)
            .insert(BoundEffects(vec![(armed_key, tree)]));
    }
}

/// Reverse all Fire effects in a `Tree` by armed source key.
///
/// Only handles `Fire` and `Sequence` — other node types are not expected
/// inside armed `ScopedTree::When` inner trees for current shapes.
fn reverse_armed_tree(tree: &Tree, entity: Entity, source: &str, world: &mut World) {
    match tree {
        Tree::Fire(et) => {
            if let Ok(reversible) = ReversibleEffectType::try_from(et.clone()) {
                reverse_all_by_source_dispatch(&reversible, entity, source, world);
            }
        }
        Tree::Sequence(terminals) => {
            for terminal in terminals {
                if let Terminal::Fire(et) = terminal
                    && let Ok(reversible) = ReversibleEffectType::try_from(et.clone())
                {
                    reverse_all_by_source_dispatch(&reversible, entity, source, world);
                }
            }
        }
        _ => {}
    }
}
