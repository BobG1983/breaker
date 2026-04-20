//! Group H — `ECHO_STRIKE_SENTINEL` drift guard (Behavior 61).
//!
//! Pins the exact literal value of the sentinel const. Any refactor that
//! renames or retypes the const must also update this test.

use super::super::system::ECHO_STRIKE_SENTINEL;

#[test]
fn echo_strike_sentinel_value_is_protocol_echo_strike() {
    assert_eq!(
        ECHO_STRIKE_SENTINEL, "protocol:echo_strike",
        "ECHO_STRIKE_SENTINEL drift guard — must be exactly \
         \"protocol:echo_strike\"; got {ECHO_STRIKE_SENTINEL:?}"
    );
}
