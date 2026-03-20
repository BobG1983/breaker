//! Pure evaluation function — maps a trigger kind + chain to an `EvalResult`.

use crate::chips::definition::TriggerChain;

/// The kind of trigger event that occurred at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverclockTriggerKind {
    /// A perfect bump was performed.
    PerfectBump,
    /// A bolt hit a cell.
    Impact,
    /// A cell was destroyed.
    CellDestroyed,
    /// A bolt was lost.
    BoltLost,
}

/// Result of evaluating a trigger kind against a trigger chain.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EvalResult {
    /// The trigger kind does not match the chain's outermost trigger.
    NoMatch,
    /// The trigger matched, and the inner chain should be armed on the bolt.
    Arm(TriggerChain),
    /// The trigger matched and the inner chain is a leaf — fire immediately.
    Fire(TriggerChain),
}

/// Evaluates whether a runtime trigger event matches the outermost trigger
/// of a `TriggerChain`.
///
/// Returns `NoMatch` if the chain is a leaf (no trigger wrapper) or if the
/// trigger kind doesn't match the chain's outermost trigger wrapper.
///
/// Returns `Fire(inner)` if the trigger matches and the inner chain is a leaf.
///
/// Returns `Arm(inner)` if the trigger matches but the inner chain is another
/// trigger wrapper (needs further resolution).
pub(crate) fn evaluate(trigger: OverclockTriggerKind, chain: &TriggerChain) -> EvalResult {
    let ((OverclockTriggerKind::PerfectBump, TriggerChain::OnPerfectBump(inner))
    | (OverclockTriggerKind::Impact, TriggerChain::OnImpact(inner))
    | (OverclockTriggerKind::CellDestroyed, TriggerChain::OnCellDestroyed(inner))
    | (OverclockTriggerKind::BoltLost, TriggerChain::OnBoltLost(inner))) = (trigger, chain)
    else {
        return EvalResult::NoMatch;
    };
    if inner.is_leaf() {
        EvalResult::Fire((**inner).clone())
    } else {
        EvalResult::Arm((**inner).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_returns_no_match_for_mismatched_trigger() {
        // PerfectBump vs OnImpact(leaf) -- wrong trigger kind
        let chain = TriggerChain::OnImpact(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        let result = evaluate(OverclockTriggerKind::PerfectBump, &chain);
        assert_eq!(result, EvalResult::NoMatch);
    }

    #[test]
    fn evaluate_returns_fire_for_matching_trigger_with_leaf() {
        // PerfectBump vs OnPerfectBump(Shockwave{64}) -- match, inner is leaf
        let chain = TriggerChain::OnPerfectBump(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        let result = evaluate(OverclockTriggerKind::PerfectBump, &chain);
        assert_eq!(
            result,
            EvalResult::Fire(TriggerChain::Shockwave { range: 64.0 })
        );
    }

    #[test]
    fn evaluate_returns_arm_for_matching_trigger_with_non_leaf() {
        // PerfectBump vs OnPerfectBump(OnImpact(Shockwave{64})) -- match, inner is non-leaf
        let inner_chain = TriggerChain::OnImpact(Box::new(TriggerChain::Shockwave { range: 64.0 }));
        let chain = TriggerChain::OnPerfectBump(Box::new(inner_chain.clone()));
        let result = evaluate(OverclockTriggerKind::PerfectBump, &chain);
        assert_eq!(result, EvalResult::Arm(inner_chain));
    }

    #[test]
    fn evaluate_impact_trigger_fires() {
        // Impact vs OnImpact(MultiBolt{3}) -- match, inner is leaf
        let chain = TriggerChain::OnImpact(Box::new(TriggerChain::MultiBolt { count: 3 }));
        let result = evaluate(OverclockTriggerKind::Impact, &chain);
        assert_eq!(
            result,
            EvalResult::Fire(TriggerChain::MultiBolt { count: 3 })
        );
    }

    #[test]
    fn evaluate_cell_destroyed_trigger_fires() {
        // CellDestroyed vs OnCellDestroyed(Shield{5.0}) -- match, inner is leaf
        let chain = TriggerChain::OnCellDestroyed(Box::new(TriggerChain::Shield { duration: 5.0 }));
        let result = evaluate(OverclockTriggerKind::CellDestroyed, &chain);
        assert_eq!(
            result,
            EvalResult::Fire(TriggerChain::Shield { duration: 5.0 })
        );
    }

    #[test]
    fn evaluate_bolt_lost_trigger_fires() {
        // BoltLost vs OnBoltLost(Shockwave{32}) -- match, inner is leaf
        let chain = TriggerChain::OnBoltLost(Box::new(TriggerChain::Shockwave { range: 32.0 }));
        let result = evaluate(OverclockTriggerKind::BoltLost, &chain);
        assert_eq!(
            result,
            EvalResult::Fire(TriggerChain::Shockwave { range: 32.0 })
        );
    }

    #[test]
    fn evaluate_leaf_chain_returns_no_match() {
        // PerfectBump vs Shockwave{64} -- leaf, not wrapped in a trigger
        let chain = TriggerChain::Shockwave { range: 64.0 };
        let result = evaluate(OverclockTriggerKind::PerfectBump, &chain);
        assert_eq!(result, EvalResult::NoMatch);
    }
}
