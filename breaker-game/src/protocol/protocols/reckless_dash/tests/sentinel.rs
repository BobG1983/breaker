//! Group G — `RECKLESS_DASH_SENTINEL` drift guard (Behavior 56).
//!
//! Pins the exact literal value of the sentinel const. Any refactor that
//! renames or retypes the const must also update this test.

use super::super::system::RECKLESS_DASH_SENTINEL;

#[test]
fn reckless_dash_sentinel_value_is_protocol_reckless_dash() {
    assert_eq!(
        RECKLESS_DASH_SENTINEL, "protocol:reckless_dash",
        "RECKLESS_DASH_SENTINEL drift guard — must be exactly \
         \"protocol:reckless_dash\"; got {RECKLESS_DASH_SENTINEL:?}"
    );
}
