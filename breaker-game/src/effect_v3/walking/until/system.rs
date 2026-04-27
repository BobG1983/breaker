//! Until node evaluator — event-scoped effect application.

use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    effect_v3::{
        conditions::DuringActive,
        dispatch::{fire_reversible_dispatch, reverse_all_by_source_dispatch, reverse_dispatch},
        storage::BoundEffects,
        triggers::time::components::EffectTimers,
        types::{Condition, ScopedTree, Tree, Trigger, TriggerContext},
    },
    prelude::SourceId,
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

        // Derive the timer duration once: only `Trigger::TimeExpires(d)`
        // gates need a backing `EffectTimers` entry. Other gates leave
        // this `None` and the arm/cancel calls below short-circuit.
        let time_expires_duration: Option<OrderedFloat<f32>> = match &self.gate_trigger {
            Trigger::TimeExpires(d) => Some(*d),
            _ => None,
        };

        match &self.inner {
            ScopedTree::During(condition, inner_scoped) => {
                apply_until_during_branch(
                    &self,
                    is_applied,
                    time_expires_duration,
                    condition,
                    inner_scoped,
                    world,
                );
            }
            _ => {
                apply_until_fire_branch(&self, is_applied, time_expires_duration, world);
            }
        }
    }
}

/// Shape B handler (`Until` wraps `During`): see `docs/architecture/effects/until.md`
/// "Shape 4 — `Until(gate, ScopedTree::During(condition, inner_scoped))`".
/// Installs the During into `BoundEffects` for the condition poller on first
/// walk, and tears down both the installed During and the Until's tracking
/// when the gate fires.
fn apply_until_during_branch(
    cmd: &UntilEvaluateCommand,
    is_applied: bool,
    time_expires_duration: Option<OrderedFloat<f32>>,
    condition: &Condition,
    inner_scoped: &ScopedTree,
    world: &mut World,
) {
    if !is_applied {
        // Install the During into BoundEffects for the condition poller
        let install_key = format!("{}#installed[0]", cmd.source);
        if let Some(mut bound) = world.get_mut::<BoundEffects>(cmd.entity)
            && !bound.0.iter().any(|(name, _)| name == &install_key)
        {
            bound.0.push((
                install_key,
                Tree::During(condition.clone(), Box::new(inner_scoped.clone())),
            ));
        }
        arm_until_first_walk(
            cmd.entity,
            &cmd.source,
            &cmd.gate_trigger,
            &cmd.inner,
            time_expires_duration,
            world,
        );

        // If gate matches on first walk, immediately tear down
        if cmd.gate_trigger == cmd.active_trigger {
            teardown_installed_during(cmd.entity, &cmd.source, inner_scoped, world);
            teardown_until_entry(
                cmd.entity,
                &cmd.source,
                &cmd.gate_trigger,
                time_expires_duration,
                world,
            );
        }
    } else if cmd.gate_trigger == cmd.active_trigger {
        // Applied and gate fires: tear down
        teardown_installed_during(cmd.entity, &cmd.source, inner_scoped, world);
        teardown_until_entry(
            cmd.entity,
            &cmd.source,
            &cmd.gate_trigger,
            time_expires_duration,
            world,
        );
    }
    // else: APPLIED but gate doesn't match — no-op (During polls normally)
}

/// Shapes 1–3 handler (non-`During` inner — Fire / Sequence / nested
/// When / On): see `docs/architecture/effects/until.md` "Shape 1", "Shape 2",
/// and "Shape 3". Fires the inner tree on first walk and reverses it when
/// the gate fires.
fn apply_until_fire_branch(
    cmd: &UntilEvaluateCommand,
    is_applied: bool,
    time_expires_duration: Option<OrderedFloat<f32>>,
    world: &mut World,
) {
    if !is_applied {
        fire_scoped_tree(&cmd.inner, cmd.entity, &cmd.source, world);
        arm_until_first_walk(
            cmd.entity,
            &cmd.source,
            &cmd.gate_trigger,
            &cmd.inner,
            time_expires_duration,
            world,
        );

        // If gate matches on first walk, immediately reverse and clean up
        if cmd.gate_trigger == cmd.active_trigger {
            reverse_scoped_tree(&cmd.inner, cmd.entity, &cmd.source, world);
            teardown_until_entry(
                cmd.entity,
                &cmd.source,
                &cmd.gate_trigger,
                time_expires_duration,
                world,
            );
        }
    } else if cmd.gate_trigger == cmd.active_trigger {
        reverse_scoped_tree(&cmd.inner, cmd.entity, &cmd.source, world);
        teardown_until_entry(
            cmd.entity,
            &cmd.source,
            &cmd.gate_trigger,
            time_expires_duration,
            world,
        );
    }
    // else: APPLIED but gate doesn't match — no-op
}

/// First-walk arming sequence shared by both branches of
/// `UntilEvaluateCommand::apply`: insert the source into `UntilApplied`,
/// arm the `EffectTimers` entry when the gate is `TimeExpires`, and
/// self-bind the Until into `BoundEffects` so subsequent walks find it.
///
/// Behavior matches the inlined sequences exactly.
fn arm_until_first_walk(
    entity: Entity,
    source: &str,
    gate: &Trigger,
    inner: &ScopedTree,
    time_expires_duration: Option<OrderedFloat<f32>>,
    world: &mut World,
) {
    if let Some(mut ua) = world.get_mut::<UntilApplied>(entity) {
        ua.0.insert(source.to_owned());
    }
    if let Some(d) = time_expires_duration {
        arm_time_expires_timer(entity, d, SourceId::from(source.to_owned()), world);
    }
    ensure_until_bound(entity, source, gate, inner, world);
}

/// Common teardown for an Until entry: cancel its timer (if any),
/// remove the source from `UntilApplied`, and retain `BoundEffects`
/// so the matching `Until(gate, _)` entry under `source` is dropped.
///
/// Used by both branches of `UntilEvaluateCommand::apply` (the `During`
/// shape and the fire/reverse shape) on every gate-match teardown path.
/// Behavior matches the inlined sequences exactly.
fn teardown_until_entry(
    entity: Entity,
    source: &str,
    gate: &Trigger,
    time_expires_duration: Option<OrderedFloat<f32>>,
    world: &mut World,
) {
    if let Some(d) = time_expires_duration {
        cancel_time_expires_timer(entity, d, &SourceId::from(source.to_owned()), world);
    }
    if let Some(mut ua) = world.get_mut::<UntilApplied>(entity) {
        ua.0.remove(source);
    }
    if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
        bound.0.retain(|(name, tree)| {
            !(name == source && matches!(tree, Tree::Until(g, _) if g == gate))
        });
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

/// Insert or refresh an `EffectTimers` entry for an Until's `TimeExpires(d)` gate.
///
/// Idempotent against re-arms with the same `(duration, source)`: if an entry
/// matching both already exists, the function returns without inserting a
/// duplicate. The component is created on demand if absent.
///
/// The idempotency key is `(original, source)`, never `(remaining, source)` —
/// `remaining` decrements each tick and would not match after any tick has
/// passed.
///
/// Two-step borrow shape: the `world.get` existence check drops its borrow
/// before `world.entity_mut(...).insert(...)` runs, then `world.get_mut(...)`
/// opens a fresh `Mut<EffectTimers>` to push the new entry. Mirrors the
/// pattern in `cancel_time_expires_timer` and avoids nested mutable borrows.
fn arm_time_expires_timer(
    entity: Entity,
    duration: OrderedFloat<f32>,
    source: SourceId,
    world: &mut World,
) {
    if world.get::<EffectTimers>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(EffectTimers { timers: vec![] });
    }
    if let Some(mut timers) = world.get_mut::<EffectTimers>(entity) {
        let already_armed = timers
            .timers
            .iter()
            .any(|(_, original, src)| *original == duration && *src == source);
        if already_armed {
            return;
        }
        timers.timers.push((duration, duration, source));
    }
}

/// Remove an `EffectTimers` entry matching `(duration, source)`. No-op when
/// the entry is absent (timer already fired or was never armed).
///
/// Two-step borrow shape: hold `Mut<EffectTimers>` only long enough to mutate
/// `timers.timers` and observe `is_empty()`, then drop the borrow before
/// calling `world.entity_mut(entity).remove::<EffectTimers>()`. This matches
/// the established pattern elsewhere in this file and avoids a nested
/// mutable borrow on `World`.
///
/// Rationale for the explicit `remove`: `tick_effect_timers` already auto-
/// removes the component when `timers.is_empty()` post-tick, so for the
/// natural-expiry path this branch is technically redundant. It IS needed
/// for the early-cancel path — when the Until reverses BEFORE its timer
/// naturally expires — otherwise the entity keeps an orphaned, empty
/// `EffectTimers` component until the next `FixedUpdate` tick.
fn cancel_time_expires_timer(
    entity: Entity,
    duration: OrderedFloat<f32>,
    source: &SourceId,
    world: &mut World,
) {
    let became_empty = world
        .get_mut::<EffectTimers>(entity)
        .is_some_and(|mut timers| {
            if let Some(pos) = timers
                .timers
                .iter()
                .position(|(_, original, src)| *original == duration && src == source)
            {
                timers.timers.swap_remove(pos);
            }
            timers.timers.is_empty()
        });
    // Close the `Mut<EffectTimers>` borrow before calling `world.entity_mut`.
    if became_empty {
        world.entity_mut(entity).remove::<EffectTimers>();
    }
}

/// Ensure this entity's `BoundEffects` contains an `Until(gate, inner)` entry
/// under `source`. Idempotent: returns early when an entry with matching name
/// AND tree shape is already present.
///
/// Required for Until entries that arrive via `evaluate_when` /
/// `evaluate_once` direct-call (where there is no outer arming stage) — it
/// self-binds the Until into `BoundEffects` so the bridge's filtered walk
/// can find it on expiry. For Untils that are already bound (the most
/// common case — a chip's top-level Until), the helper's idempotent guard
/// makes the call a no-op.
fn ensure_until_bound(
    entity: Entity,
    source: &str,
    gate: &Trigger,
    inner: &ScopedTree,
    world: &mut World,
) {
    let expected_tree = Tree::Until(gate.clone(), Box::new(inner.clone()));
    if world.get::<BoundEffects>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(BoundEffects(vec![(source.to_owned(), expected_tree)]));
        return;
    }
    if let Some(mut bound) = world.get_mut::<BoundEffects>(entity) {
        let already = bound
            .0
            .iter()
            .any(|(name, tree)| name == source && *tree == expected_tree);
        if already {
            return;
        }
        bound.0.push((source.to_owned(), expected_tree));
    }
}
