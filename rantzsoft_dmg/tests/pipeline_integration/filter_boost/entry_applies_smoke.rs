//! Smoke check that `entry_applies` is reachable from the crate root re-export.

use rantzsoft_dmg::{SourceId, entry_applies};

// ── Behavior 26: `entry_applies` is reachable from the crate root re-export ──

#[test]
fn entry_applies_is_reachable_from_crate_root() {
    // Smoke-check that the `lib.rs` re-export
    // (`pub use source_id::{SourceId, entry_applies};`) actually
    // compiled. If the re-export is missing or visibility is wrong,
    // this test will fail to build at the RED gate.
    assert!(entry_applies(None, None));
    let filter = SourceId::from("protocol:burnout");
    let emission = SourceId::from("protocol:burnout");
    assert!(entry_applies(Some(&filter), Some(&emission)));
}
