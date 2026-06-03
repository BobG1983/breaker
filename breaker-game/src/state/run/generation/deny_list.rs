use super::constraint::BehaviorKind;

/// Pairs of `BehaviorKind` that must not appear together on the same cell.
/// Populated in Wave 5 as conflicts are surfaced via playtesting.
pub(crate) const DENY_LIST: &[(BehaviorKind, BehaviorKind)] = &[];
