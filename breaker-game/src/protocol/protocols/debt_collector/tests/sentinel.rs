//! Group J — `DEBT_COLLECTOR_SENTINEL` drift guard (Behavior 54).
//!
//! Pins the exact literal value of the sentinel const. Any refactor that
//! renames or retypes the const must also update this test.

use super::super::system::DEBT_COLLECTOR_SENTINEL;

#[test]
fn debt_collector_sentinel_value_is_protocol_debt_collector() {
    assert_eq!(
        DEBT_COLLECTOR_SENTINEL, "protocol:debt_collector",
        "DEBT_COLLECTOR_SENTINEL drift guard — must be exactly \
         \"protocol:debt_collector\"; got {DEBT_COLLECTOR_SENTINEL:?}"
    );
}
