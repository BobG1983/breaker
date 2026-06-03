//! Group F — `DENY_LIST` constant (#36)

use crate::state::run::generation::*;

// ── Behavior #36: DENY_LIST is an empty const slice at Wave 1 ────────────────

fn assert_type(_: &[(BehaviorKind, BehaviorKind)]) {}

#[test]
fn deny_list_is_empty_at_wave_1() {
    assert_type(DENY_LIST);
    assert!(DENY_LIST.is_empty(), "DENY_LIST must be empty at Wave 1");
    assert_eq!(DENY_LIST.len(), 0);
}
