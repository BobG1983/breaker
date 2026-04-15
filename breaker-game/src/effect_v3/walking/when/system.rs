//! When node evaluator — repeating trigger gate.

use bevy::prelude::*;

use crate::effect_v3::{
    commands::EffectCommandsExt,
    types::{Tree, Trigger, TriggerContext},
    walking::walk_effects::evaluate_tree,
};

/// Evaluate a `Tree::When` node: if the trigger matches, evaluate the inner tree.
/// Repeats on every match.
///
/// When the inner tree's root is itself a trigger gate (`When`, `Once`,
/// `Until`), the outer `When` ARMS the inner by staging it under the
/// same source name instead of recursing. The staged entry is evaluated
/// on the *next* matching trigger — not the one that armed it. This
/// preserves the documented "new trigger event required" semantics of
/// nested gate types.
///
/// For non-gate inner trees (`Fire`, `Sequence`, `On`, `During`), the
/// previous recursion path is preserved.
pub fn evaluate_when(
    entity: Entity,
    gate_trigger: &Trigger,
    inner: &Tree,
    active_trigger: &Trigger,
    context: &TriggerContext,
    source: &str,
    commands: &mut Commands,
) {
    if gate_trigger != active_trigger {
        return;
    }
    match inner {
        Tree::When(..) | Tree::Once(..) | Tree::Until(..) => {
            // Arm the inner gate by staging it under the same source.
            // The staged tree carries no context binding — when the
            // staged entry is later walked, it runs against the next
            // trigger's context, not the arming frame's context. This is
            // intentional.
            commands.stage_effect(entity, source.to_owned(), inner.clone());
        }
        _ => {
            evaluate_tree(entity, inner, active_trigger, context, source, commands);
        }
    }
}
