//! Pure evaluation function — maps a runtime `Trigger` + `EffectNode` to matched children.
//!
//! Unified version covering all trigger kinds including bump grades
//! (`EarlyBump`, `LateBump`, `BumpWhiff`) and `Death`.

use crate::effect::definition::{EffectNode, Trigger};

/// Returns the matched children if the trigger matches the node's declared trigger,
/// or `None` if there is no match.
///
/// `TimeExpires` and `NodeTimerThreshold` have no runtime trigger mapping
/// and always return `None`.
pub(crate) fn evaluate_node(trigger: Trigger, node: &EffectNode) -> Option<&[EffectNode]> {
    match node {
        EffectNode::When {
            trigger: declared,
            then,
        } if trigger_matches(trigger, *declared) => Some(then.as_slice()),
        _ => None,
    }
}

/// Returns `true` if the runtime trigger matches the declared trigger on the node.
///
/// Uses `PartialEq` for matching — both simple and parameterized variants
/// (e.g., `Impact(ImpactTarget)`) are handled by the derived equality.
/// `TimeExpires` and `NodeTimerThreshold` are explicitly excluded because
/// they have no runtime trigger mapping. New variants automatically work
/// without code changes.
fn trigger_matches(runtime: Trigger, declared: Trigger) -> bool {
    match declared {
        Trigger::TimeExpires(_) | Trigger::NodeTimerThreshold(_) => false,
        _ => runtime == declared,
    }
}
