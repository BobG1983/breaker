//! Group I — `SIPHON_SENTINEL` drift guard (Behavior 45).
//!
//! Pins the exact literal value of the sentinel const. Reserved for future
//! heal/debug plumbing; any refactor that renames or retypes the const must
//! also update this test.

use super::super::system::SIPHON_SENTINEL;

#[test]
fn siphon_sentinel_value_is_protocol_siphon() {
    assert_eq!(
        SIPHON_SENTINEL, "protocol:siphon",
        "SIPHON_SENTINEL drift guard — must be exactly \"protocol:siphon\"; got {SIPHON_SENTINEL:?}"
    );
}
