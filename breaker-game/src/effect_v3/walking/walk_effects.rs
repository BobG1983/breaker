//! `walk_effects` — outer loop for evaluating effect trees against a trigger.
//!
//! This file contains both walker entry points (`walk_bound_effects` and
//! `walk_staged_effects`) plus the shared `evaluate_tree` dispatch. All
//! three are scoped to `pub(in crate::effect_v3)` so other domains must
//! go through `EffectCommandsExt` methods instead of bypassing the
//! command queue.

use bevy::prelude::*;

use super::{
    evaluate_during, evaluate_fire, evaluate_on, evaluate_once, evaluate_sequence, evaluate_until,
    evaluate_when,
};
use crate::effect_v3::{
    commands::EffectCommandsExt,
    types::{Tree, Trigger, TriggerContext},
};

/// Walk all effect trees in an entity's `BoundEffects`, evaluating nodes
/// against the given trigger and context.
///
/// This is the main entry point for trigger dispatch on persistent bound
/// trees. Bridge systems call this after `walk_staged_effects` for the
/// same `(entity, trigger, context)` tuple — see `walk_staged_effects`
/// for why ordering matters.
pub(in crate::effect_v3) fn walk_bound_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    trees: &[(String, Tree)],
    commands: &mut Commands,
) {
    for (source, tree) in trees {
        evaluate_tree(entity, tree, trigger, context, source, commands);
    }
}

/// Walk staged effect trees on an entity. On trigger match, the matching
/// entry is consumed **entry-specifically** (removed from `StagedEffects`
/// via first-match on the `(source, Tree)` tuple) using
/// `commands.remove_staged_effect`. Non-matching entries are left in
/// place. The bound outer tree (if any) is never touched by the consume —
/// only `BoundEffects` cleanup happens through `remove_effect` during
/// explicit chip disarm, never here.
///
/// Bridge systems must call this BEFORE `walk_bound_effects` so that
/// entries staged by a `When` / `Once` arming during the same trigger
/// event do not erroneously match the trigger that staged them (see
/// `evaluate_when` and `evaluate_once` arming logic).
///
/// Same-tick stage-then-consume interaction: if the inner staged tree is
/// `Tree::When(A, Tree::When(A, ...))`, evaluating it will call
/// `commands.stage_effect(entity, source, inner_when)` to arm its inner
/// gate. The subsequent `commands.remove_staged_effect(entity, source,
/// outer_tree)` removes the ORIGINAL outer staged entry by tuple match —
/// the freshly-armed inner (different `Tree` value) is preserved. Deeper
/// chains therefore peel layer by layer across ticks rather than wiping:
/// depth N takes N ticks to prime, after which every matching trigger
/// fires the inner-most `Fire` once. The bound outer persists across all
/// of this and continues to re-arm the top level every tick.
pub(in crate::effect_v3) fn walk_staged_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    trees: &[(String, Tree)],
    commands: &mut Commands,
) {
    for (source, tree) in trees {
        // A staged entry is always the inner tree of a `When`/`Once`/`Until`
        // that was armed by its outer gate. Only consume it when its own
        // top-level gate trigger matches — otherwise `evaluate_tree` would
        // be a no-op and we would burn a staged entry that never fired.
        if !tree_matches_trigger(tree, trigger) {
            continue;
        }
        // `evaluate_tree` may queue `Stage` commands for arm-push paths.
        // Those MUST be enqueued BEFORE the consume below so the
        // entry-specific remove hits the original staged entry
        // (first match on `(source, tree)`) and not the freshly-armed
        // inner.
        evaluate_tree(entity, tree, trigger, context, source, commands);
        // Consume THIS specific entry — entry-specific by
        // `(source, tree)` tuple. Fresh same-name stages are preserved.
        commands.remove_staged_effect(entity, source.clone(), tree.clone());
    }
}

/// Does this staged tree's top-level gate match the active trigger?
///
/// A staged entry on `StagedEffects` is always the inner tree of a
/// `When`/`Once`/`Until` armed by its outer gate, so its root variant is
/// one of those three. Non-gate roots (`Fire`, `Sequence`, `On`,
/// `During`) should never be staged in normal flow — return `false`
/// defensively so `walk_staged_effects` leaves them alone.
fn tree_matches_trigger(tree: &Tree, active: &Trigger) -> bool {
    match tree {
        Tree::When(gate, _) | Tree::Once(gate, _) | Tree::Until(gate, _) => gate == active,
        _ => false,
    }
}

/// Recursively evaluate a single tree node against the active trigger.
pub(in crate::effect_v3) fn evaluate_tree(
    entity: Entity,
    tree: &Tree,
    trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    match tree {
        Tree::Fire(effect_type) => {
            evaluate_fire(entity, effect_type, source, context, commands);
        }
        Tree::When(gate_trigger, inner) => {
            evaluate_when(
                entity,
                gate_trigger,
                inner,
                trigger,
                context,
                source,
                commands,
            );
        }
        Tree::Once(gate_trigger, inner) => {
            evaluate_once(
                entity,
                gate_trigger,
                inner,
                trigger,
                context,
                source,
                commands,
            );
        }
        Tree::During(condition, inner) => {
            evaluate_during(entity, condition, inner, context, source, commands);
        }
        Tree::Until(gate_trigger, inner) => {
            evaluate_until(
                entity,
                gate_trigger,
                inner,
                trigger,
                context,
                source,
                commands,
            );
        }
        Tree::Sequence(terminals) => {
            evaluate_sequence(entity, terminals, context, source, commands);
        }
        Tree::On(target, terminal) => {
            evaluate_on(entity, *target, terminal, context, source, commands);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        types::EffectType,
    };

    #[test]
    fn walk_effects_fire_tree_queues_fire_command() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let trees = vec![(
            "test_chip".to_string(),
            Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        )];

        let trigger = Trigger::Bumped;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_bound_effects(entity, &trigger, &context, &trees, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after walk_bound_effects fires a SpeedBoost");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn walk_effects_when_matching_trigger_fires_inner() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let trees = vec![(
            "test_chip".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )];

        let trigger = Trigger::Bumped;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_bound_effects(entity, &trigger, &context, &trees, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("EffectStack should exist after matching When trigger");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn walk_effects_when_non_matching_trigger_does_nothing() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let trees = vec![(
            "test_chip".to_string(),
            Tree::When(
                Trigger::Bumped,
                Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.5),
                }))),
            ),
        )];

        // Use a different trigger that doesn't match
        let trigger = Trigger::BoltLostOccurred;
        let context = TriggerContext::None;

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_bound_effects(entity, &trigger, &context, &trees, &mut commands);
        }
        queue.apply(&mut world);

        let stack = world.get::<EffectStack<SpeedBoostConfig>>(entity);
        assert!(
            stack.is_none(),
            "No EffectStack should exist when trigger doesn't match"
        );
    }

    // ================================================================
    // Wave C — walk_staged_effects
    // ================================================================

    // ----------------------------------------------------------------
    // Behavior 15: walk_staged_effects with a matching trigger consumes
    //              the entry via remove_effect
    // ----------------------------------------------------------------
    #[test]
    fn walk_staged_effects_matching_trigger_consumes_entry() {
        let mut world = World::new();
        let entity = world
            .spawn((
                StagedEffects(vec![(
                    "chip_a".to_string(),
                    Tree::When(
                        Trigger::Bumped,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        }))),
                    ),
                )]),
                BoundEffects(vec![]),
            ))
            .id();

        let staged_snapshot = world.get::<StagedEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_staged_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &staged_snapshot,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        let stack = world
            .get::<EffectStack<SpeedBoostConfig>>(entity)
            .expect("inner Fire should have run");
        assert_eq!(stack.len(), 1);

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert!(
            staged.0.iter().all(|(name, _)| name != "chip_a"),
            "staged entry must be consumed via RemoveEffectCommand"
        );

        let bound = world.get::<BoundEffects>(entity).unwrap();
        assert!(
            bound.0.is_empty(),
            "BoundEffects should remain empty — only the staged copy was removed"
        );
    }

    // ----------------------------------------------------------------
    // Behavior 16: walk_staged_effects with a non-matching trigger
    //              leaves the entry alone
    // ----------------------------------------------------------------
    #[test]
    fn walk_staged_effects_non_matching_trigger_leaves_entry_alone() {
        let mut world = World::new();
        let entity = world
            .spawn((
                StagedEffects(vec![(
                    "chip_a".to_string(),
                    Tree::When(
                        Trigger::BoltLostOccurred,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.5),
                        }))),
                    ),
                )]),
                BoundEffects(vec![]),
            ))
            .id();

        let staged_snapshot = world.get::<StagedEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_staged_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &staged_snapshot,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        assert!(
            world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
            "inner must not fire when inner gate trigger does not match"
        );

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(
            staged.0[0],
            (
                "chip_a".to_string(),
                Tree::When(
                    Trigger::BoltLostOccurred,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    }))),
                ),
            )
        );
    }

    // ----------------------------------------------------------------
    // Behavior 17: walk_staged_effects defensively handles a staged
    //              Tree::Fire (non-gate root) as a no-op
    // ----------------------------------------------------------------
    #[test]
    fn walk_staged_effects_defensively_ignores_non_gate_root() {
        let mut world = World::new();
        let entity = world
            .spawn((
                StagedEffects(vec![(
                    "chip_a".to_string(),
                    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    })),
                )]),
                BoundEffects(vec![]),
            ))
            .id();

        let staged_snapshot = world.get::<StagedEffects>(entity).unwrap().0.clone();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            walk_staged_effects(
                entity,
                &Trigger::Bumped,
                &TriggerContext::None,
                &staged_snapshot,
                &mut commands,
            );
        }
        queue.apply(&mut world);

        assert!(
            world.get::<EffectStack<SpeedBoostConfig>>(entity).is_none(),
            "non-gate staged Fire must not fire"
        );

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(staged.0.len(), 1);
        assert_eq!(staged.0[0].0, "chip_a");
    }
}
